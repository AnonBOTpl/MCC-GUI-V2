//! Zarządzanie procesami MCC: jedno **połączenie** (konto + serwer) = jeden proces.
//!
//! Wszystkie połączenia jednego konta uruchamiamy z **tym samym katalogiem
//! roboczym** (`accounts/<id>/`), więc dzielą jedną sesję MCC - jedno logowanie
//! Microsoft obsługuje wszystkie serwery konta. Każde połączenie ma osobny plik
//! ustawień (`settings-<serwer>.ini`) przekazany jako pierwszy argument, więc
//! każde łączy się z własnym serwerem.
//!
//! Moduł nie zależy od Tauri (patrz `EventSink`), więc da się go testować bez
//! aplikacji i bez prawdziwego MCC.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde_json::json;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, Command};
use tokio::sync::{Mutex, oneshot};
use tokio::time::{Instant, sleep};

use crate::account::{Account, AccountStatus, MAX_SERVERS_PER_ACCOUNT, ServerEntry};
use crate::connection::{self, Connection};
use crate::device_code;
use crate::error::AppError;
use crate::events::{EventSink, LogStream};
use crate::input;
use crate::job_object::JobObject;
use crate::launch;

/// Flaga `CreateProcessW`: brak okna konsoli dla procesu potomnego.
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// Komenda MCC zamykająca klienta po połączeniu (potwierdzone w źródle MCC).
const QUIT_COMMAND: &str = "/quit";

/// Ile czekamy na łagodne zamknięcie po `/quit`, zanim wymusimy kill.
const STOP_TIMEOUT: Duration = Duration::from_secs(5);

/// Co ile sprawdzamy, czy proces już się zakończył.
const STATUS_POLL: Duration = Duration::from_millis(50);

/// Uruchomiony proces połączenia.
struct Running {
    stdin: Option<ChildStdin>,
    status: AccountStatus,
    /// Kanał wymuszający kill, gdy łagodne `/quit` nie zadziała.
    kill_tx: Option<oneshot::Sender<()>>,
    /// Czy zgłosiliśmy już prompt logowania (żeby nie duplikować bannera).
    login_prompted: bool,
    /// Czy zgłosiliśmy już połączenie z serwerem.
    joined: bool,
}

/// Znacznik czasu w ms od epoki Unix.
fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// Zamienia błąd wejścia/wyjścia na stabilny kod z numerem błędu systemowego.
fn io_error(code: &str, error: &std::io::Error) -> AppError {
    AppError::with_params(
        code,
        json!({ "osError": error.raw_os_error().unwrap_or(-1) }),
    )
}

pub struct ProcessManager<S: EventSink> {
    /// Ścieżka do `MinecraftClient.exe`. Trzymamy ją za `RwLock`, bo użytkownik
    /// może ją zmienić w ustawieniach już po starcie aplikacji (Etap 4).
    mcc_exe: RwLock<Option<PathBuf>>,
    accounts_root: PathBuf,
    sink: Arc<S>,
    job: JobObject,
    accounts: Mutex<HashMap<String, Account>>,
    /// Uruchomione połączenia; klucz to `connection::connection_id`.
    running: Arc<Mutex<HashMap<String, Running>>>,
}

impl<S: EventSink> ProcessManager<S> {
    /// Tworzy menedżer. `mcc_exe = None` oznacza, że ścieżka nie jest jeszcze
    /// znana (Etap 4 doda jej wybór w UI).
    pub fn new(
        mcc_exe: Option<PathBuf>,
        accounts_root: PathBuf,
        sink: Arc<S>,
    ) -> Result<Self, AppError> {
        let job = JobObject::new_kill_on_close().map_err(|error| {
            tracing::error!(%error, "nie udało się utworzyć Job Object");
            io_error("process.jobCreateFailed", &error)
        })?;

        Ok(Self {
            mcc_exe: RwLock::new(mcc_exe),
            accounts_root,
            sink,
            job,
            accounts: Mutex::new(HashMap::new()),
            running: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Zmienia ścieżkę do MCC (np. po wyborze w ustawieniach).
    pub fn set_mcc_exe(&self, exe: Option<PathBuf>) {
        let mut guard = self
            .mcc_exe
            .write()
            .unwrap_or_else(|error| error.into_inner());
        *guard = exe;
    }

    /// Zwraca bieżącą ścieżkę do MCC. Odzyskujemy wartość także po panice
    /// innego wątku (`into_inner`), żeby zatrucie blokady nie wysypało aplikacji.
    fn current_mcc_exe(&self) -> Option<PathBuf> {
        self.mcc_exe
            .read()
            .unwrap_or_else(|error| error.into_inner())
            .clone()
    }

    /// Katalog roboczy konta - **wspólny** dla wszystkich jego połączeń, więc
    /// wspólna jest też sesja MCC (jedno logowanie na wszystkie serwery konta).
    fn account_dir(&self, account_id: &str) -> PathBuf {
        self.accounts_root.join(account_id)
    }

    /// Zwraca kopię konta albo `account.notFound`.
    async fn account(&self, id: &str) -> Result<Account, AppError> {
        self.accounts
            .lock()
            .await
            .get(id)
            .cloned()
            .ok_or_else(|| AppError::new("account.notFound"))
    }

    /// Przywraca zapisane konta przy starcie aplikacji (Etap 4).
    ///
    /// Znane konta tylko aktualizuje (bez restartu procesów), a nowe rejestruje
    /// (tworzy katalog konta). **Nie uruchamia** żadnego procesu i nie usuwa kont
    /// nieobecnych na liście - to robi jawnie `remove_account`.
    pub async fn sync_accounts(&self, accounts: &[Account]) {
        for account in accounts {
            let known = {
                let mut map = self.accounts.lock().await;
                match map.get_mut(&account.id) {
                    Some(existing) => {
                        *existing = account.clone();
                        true
                    }
                    None => false,
                }
            };
            if known {
                continue;
            }
            match self.register_account(account).await {
                Ok(()) => {
                    tracing::info!(account = %account.id, "przywrócono zapisane konto");
                }
                Err(error) => {
                    tracing::warn!(%error, account = %account.id, "nie udało się przywrócić konta");
                }
            }
        }
    }

    /// Rejestruje konto: tworzy jego katalog i dodaje do menedżera. Bez procesów.
    async fn register_account(&self, account: &Account) -> Result<(), AppError> {
        let dir = self.account_dir(&account.id);
        if let Err(error) = tokio::fs::create_dir_all(&dir).await {
            tracing::error!(%error, account = %account.id, "nie udało się utworzyć katalogu konta");
            return Err(io_error("account.dirCreateFailed", &error));
        }
        self.accounts
            .lock()
            .await
            .insert(account.id.clone(), account.clone());
        Ok(())
    }

    /// Dodaje konto (bez uruchamiania czegokolwiek - użytkownik startuje
    /// połączenia sam). Zapisywanie konta na dysk należy do warstwy komend.
    pub async fn add_account(&self, account: Account) -> Result<(), AppError> {
        if self.accounts.lock().await.contains_key(&account.id) {
            return Err(AppError::new("account.alreadyExists"));
        }
        self.register_account(&account).await
    }

    /// Usuwa konto i zatrzymuje wszystkie jego połączenia.
    ///
    /// Katalogu konta **nie kasujemy**: trzyma sesję MCC, a pliki sesji są dla
    /// aplikacji nietykalne. Dzięki temu ponowne dodanie tego
    /// samego konta nie wymaga logowania od nowa.
    pub async fn remove_account(&self, id: &str) -> Result<(), AppError> {
        let account = self.accounts.lock().await.remove(id);
        let Some(account) = account else {
            return Err(AppError::new("account.notFound"));
        };
        for entry in &account.servers {
            let conn = connection::connection_id(id, &entry.address);
            let _ = self.stop_connection(&conn).await;
        }
        Ok(())
    }

    /// Dopisuje serwer do konta (bez uruchamiania procesu).
    ///
    /// Ten sam adres to **aktualizacja wersji**, a nie błąd - dzięki temu można
    /// dopisać wersję do już zapisanego serwera (np. gdy MCC zapytał o nią
    /// w konsoli). Zmiana wersji działa od następnego uruchomienia połączenia.
    pub async fn add_server(
        &self,
        account_id: &str,
        address: &str,
        version: Option<&str>,
    ) -> Result<(), AppError> {
        let address = connection::validate_server(address)?;
        let version = crate::account::normalize_version(version)?;
        let mut accounts = self.accounts.lock().await;
        let account = accounts
            .get_mut(account_id)
            .ok_or_else(|| AppError::new("account.notFound"))?;

        if let Some(existing) = account
            .servers
            .iter_mut()
            .find(|entry| entry.address.eq_ignore_ascii_case(&address))
        {
            existing.version = version;
            return Ok(());
        }
        if account.servers.len() >= MAX_SERVERS_PER_ACCOUNT {
            return Err(AppError::new("account.tooManyServers"));
        }
        account.servers.push(ServerEntry { address, version });
        Ok(())
    }

    /// Usuwa serwer z konta, zatrzymując jego połączenie, jeśli działa.
    pub async fn remove_server(&self, account_id: &str, server: &str) -> Result<(), AppError> {
        let conn = connection::connection_id(account_id, server);
        let _ = self.stop_connection(&conn).await;

        let mut accounts = self.accounts.lock().await;
        let account = accounts
            .get_mut(account_id)
            .ok_or_else(|| AppError::new("account.notFound"))?;
        let before = account.servers.len();
        account
            .servers
            .retain(|entry| !entry.address.eq_ignore_ascii_case(server));
        if account.servers.len() == before {
            return Err(AppError::new("connection.notFound"));
        }
        Ok(())
    }

    /// Uruchamia proces dla pary (konto, serwer).
    pub async fn start_connection(&self, account_id: &str, server: &str) -> Result<(), AppError> {
        let mcc_exe = self
            .current_mcc_exe()
            .ok_or_else(|| AppError::new("mcc.executableNotSet"))?;

        let account = self.account(account_id).await?;
        // Serwer musi być zapisany na koncie - bierzemy jego zapisaną postać
        // (z wersją), żeby klucz połączenia zgadzał się z listą w UI.
        let entry = account
            .servers
            .iter()
            .find(|saved| saved.address.eq_ignore_ascii_case(server))
            .cloned()
            .ok_or_else(|| AppError::new("connection.notFound"))?;

        let conn = Connection::new(account_id, &entry.address);
        if self.running.lock().await.contains_key(&conn.id) {
            return Err(AppError::new("connection.alreadyRunning"));
        }

        self.spawn_process(&account, &entry, &mcc_exe).await
    }

    /// Ponownie uruchamia połączenie (stop, jeśli działa, potem start).
    pub async fn restart_connection(&self, account_id: &str, server: &str) -> Result<(), AppError> {
        let conn = connection::connection_id(account_id, server);
        if self.running.lock().await.contains_key(&conn) {
            self.stop_connection(&conn).await?;
        }
        self.start_connection(account_id, server).await
    }

    /// Wysyła wpis do jednego połączenia.
    pub async fn send_input(&self, connection_id: &str, text: &str) -> Result<(), AppError> {
        let line = input::validate_input(text)?;
        // `/quit` wpisane przez użytkownika zamyka proces **na jego życzenie**.
        // Bez tej wiedzy strażnik uznałby samodzielne zakończenie za awarię
        // i pokazał fałszywy błąd w UI.
        let quitting = input::is_quit_command(&line);

        let mut running = self.running.lock().await;
        let entry = running
            .get_mut(connection_id)
            .ok_or_else(|| AppError::new("connection.notRunning"))?;
        // Do `stdin` piszemy także w `awaiting_login` - MCC oczekuje tam
        // ewentualnej odpowiedzi użytkownika (np. wyboru serwera lub wersji).
        write_line(entry, &line).await?;
        if quitting {
            // Blokadę trzymamy do końca, więc strażnik nie zdejmie wpisu przed
            // ustawieniem statusu - nie ma wyścigu.
            entry.status = AccountStatus::Stopping;
        }
        Ok(())
    }

    /// Wysyła wpis do wszystkich działających połączeń, z opóźnieniem między nimi.
    /// Zwraca identyfikatory połączeń, do których faktycznie wysłano.
    pub async fn send_input_all(&self, text: &str, delay_ms: u64) -> Result<Vec<String>, AppError> {
        let line = input::validate_input(text)?;
        // Tak samo jak w `send_input`: `/quit` to zamknięcie na życzenie
        // użytkownika, więc status `Stopping` zamiast fałszywego `Crashed`.
        let quitting = input::is_quit_command(&line);

        let ids: Vec<String> = {
            let running = self.running.lock().await;
            running
                .iter()
                .filter(|(_, entry)| is_input_ready(entry.status) && entry.stdin.is_some())
                .map(|(id, _)| id.clone())
                .collect()
        };

        let mut sent = Vec::new();
        for (index, id) in ids.iter().enumerate() {
            if index > 0 && delay_ms > 0 {
                sleep(Duration::from_millis(delay_ms)).await;
            }
            let mut running = self.running.lock().await;
            if let Some(entry) = running.get_mut(id)
                && is_input_ready(entry.status)
                && write_line(entry, &line).await.is_ok()
            {
                if quitting {
                    entry.status = AccountStatus::Stopping;
                }
                sent.push(id.clone());
            }
        }
        Ok(sent)
    }

    /// Łagodnie zatrzymuje połączenie (`/quit`), a po timeoucie wymusza zamknięcie.
    ///
    /// Wywołanie dla nieuruchomionego połączenia jest bezpiecznym „nic do zrobienia".
    ///
    /// Powtórne zatrzymanie połączenia, które **już** jest w stanie `Stopping`, nie jest
    /// ignorowane - od razu wymuszamy kill. Powód: stan `Stopping` ustawia też `/quit`
    /// wpisane przez użytkownika (`send_input`), a MCC potrafi taką linię zjeść jako
    /// odpowiedź na swoje pytanie w konsoli i **nie zakończyć się** (patrz
    /// `docs/MCC_NOTES.md` §3.2). Wcześniejsze ciche wyjście czyniło z przycisku
    /// „Disconnect" martwy przycisk: proces żył dalej, a UI pokazywało `running`.
    pub async fn stop_connection(&self, connection_id: &str) -> Result<(), AppError> {
        if !self.running.lock().await.contains_key(connection_id) {
            return Ok(());
        }

        // `true` = `/quit` poszło właśnie teraz, `false` = ta komenda była już wysłana
        // wcześniej, więc nie wysyłamy jej drugi raz.
        let quit_sent = {
            let mut running = self.running.lock().await;
            let Some(entry) = running.get_mut(connection_id) else {
                return Ok(());
            };
            if entry.status == AccountStatus::Stopping {
                false
            } else {
                entry.status = AccountStatus::Stopping;
                if let Some(stdin) = entry.stdin.as_mut() {
                    let _ = stdin.write_all(QUIT_COMMAND.as_bytes()).await;
                    let _ = stdin.write_all(b"\n").await;
                    let _ = stdin.flush().await;
                }
                true
            }
        };

        self.sink
            .emit_status(connection_id, AccountStatus::Stopping);
        if quit_sent {
            tracing::info!(connection = %connection_id, "wysłano /quit, czekam na zamknięcie");
            if self.wait_until_stopped(connection_id, STOP_TIMEOUT).await {
                return Ok(());
            }
        } else {
            tracing::warn!(
                connection = %connection_id,
                "ponowne zatrzymanie - nie czekam dłużej na /quit"
            );
        }

        // Łagodne zamknięcie nie zadziałało - wymuszamy.
        tracing::warn!(connection = %connection_id, "brak zamknięcia po /quit, wymuszam kill");
        let kill_tx = {
            let mut running = self.running.lock().await;
            running
                .get_mut(connection_id)
                .and_then(|entry| entry.kill_tx.take())
        };
        if let Some(tx) = kill_tx {
            let _ = tx.send(());
        }
        let _ = self.wait_until_stopped(connection_id, STOP_TIMEOUT).await;
        Ok(())
    }

    /// Ile połączeń jest teraz w toku (`starting`, `awaiting_login`, `running`,
    /// `stopping`). Używane przy zamykaniu okna: pytamy o potwierdzenie tylko
    /// wtedy, gdy jest co przerywać. Wpis znika z mapy dopiero wtedy, gdy proces
    /// się kończy, więc mapa trzyma wyłącznie połączenia w toku.
    pub async fn active_count(&self) -> usize {
        self.running.lock().await.len()
    }

    /// Zatrzymuje wszystkie połączenia (używane przy zamykaniu aplikacji).
    pub async fn stop_all(&self) {
        let ids: Vec<String> = self.running.lock().await.keys().cloned().collect();
        for id in ids {
            let _ = self.stop_connection(&id).await;
        }
    }

    /// Czeka, aż wpis zniknie z mapy uruchomionych (proces się zakończył).
    async fn wait_until_stopped(&self, connection_id: &str, timeout: Duration) -> bool {
        let deadline = Instant::now() + timeout;
        loop {
            if !self.running.lock().await.contains_key(connection_id) {
                return true;
            }
            if Instant::now() >= deadline {
                return false;
            }
            sleep(STATUS_POLL).await;
        }
    }

    /// Uruchamia proces połączenia, podłącza czytniki i strażnika zakończenia.
    async fn spawn_process(
        &self,
        account: &Account,
        server: &ServerEntry,
        mcc_exe: &Path,
    ) -> Result<(), AppError> {
        let conn = Connection::new(&account.id, &server.address);
        let dir = self.account_dir(&account.id);
        if let Err(error) = tokio::fs::create_dir_all(&dir).await {
            tracing::error!(%error, account = %account.id, "nie udało się utworzyć katalogu konta");
            return Err(io_error("account.dirCreateFailed", &error));
        }

        // Ustawienia są per serwer, ale leżą w katalogu konta (wspólny CWD =
        // wspólna sesja MCC).
        let settings_file = connection::settings_file_name(&server.address);
        if let Err(error) = tokio::fs::write(
            dir.join(&settings_file),
            launch::build_settings_ini(account, server),
        )
        .await
        {
            tracing::error!(%error, account = %account.id, "nie udało się zapisać ustawień połączenia");
            return Err(io_error("account.settingsWriteFailed", &error));
        }

        let mut command = Command::new(mcc_exe);
        command.args(launch::build_args(&settings_file));
        command.current_dir(&dir);
        command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        command.creation_flags(CREATE_NO_WINDOW);

        let mut child = command.spawn().map_err(|error| {
            tracing::error!(%error, connection = %conn.id, "nie udało się uruchomić MCC");
            io_error("process.spawnFailed", &error)
        })?;

        // Przypisanie do Job Object gwarantuje sprzątanie po zamknięciu aplikacji.
        {
            let handle = child
                .raw_handle()
                .ok_or_else(|| AppError::new("process.handleUnavailable"))?;
            if let Err(error) = self.job.assign(windows::Win32::Foundation::HANDLE(handle)) {
                tracing::error!(%error, connection = %conn.id, "nie udało się przypisać do Job Object");
                let _ = child.start_kill();
                return Err(io_error("process.jobAssignFailed", &error));
            }
        }

        let stdin = child.stdin.take();
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        // Wpis musi powstać zanim wystartują czytniki i strażnik, inaczej wyścig.
        let (kill_tx, kill_rx) = oneshot::channel();
        self.running.lock().await.insert(
            conn.id.clone(),
            Running {
                stdin,
                status: AccountStatus::Starting,
                kill_tx: Some(kill_tx),
                login_prompted: false,
                joined: false,
            },
        );
        self.sink.emit_status(&conn.id, AccountStatus::Starting);

        // Czytniki `stdout`/`stderr` - po jednym zadaniu na strumień. Czytnik
        // `stdout` dodatkowo wykrywa prompt logowania i moment połączenia.
        spawn_reader(
            self.running.clone(),
            self.sink.clone(),
            conn.id.clone(),
            stdout,
            LogStream::Stdout,
        );
        spawn_reader(
            self.running.clone(),
            self.sink.clone(),
            conn.id.clone(),
            stderr,
            LogStream::Stderr,
        );

        // Strażnik: czeka na naturalne zakończenie albo sygnał kill.
        spawn_waiter(
            self.running.clone(),
            self.sink.clone(),
            conn.id.clone(),
            server.address.clone(),
            child,
            kill_rx,
        );

        // Status początkowy: proces żyje i przyjmuje wpis, więc `running`. Gdy MCC
        // wypisze prompt logowania Microsoft, czytnik przełączy połączenie na
        // `awaiting_login`, a po zdaniu `Server was successfully joined.` z powrotem
        // na `running` (Etap 3).
        //
        // Czytnik `stdout` startuje wcześniej, więc mógł już wykryć prompt - wtedy
        // nie nadpisujemy `awaiting_login`.
        {
            let mut running = self.running.lock().await;
            if let Some(entry) = running.get_mut(&conn.id)
                && !entry.login_prompted
            {
                entry.status = AccountStatus::Running;
                self.sink.emit_status(&conn.id, AccountStatus::Running);
            }
        }

        tracing::info!(connection = %conn.id, "uruchomiono MCC");
        Ok(())
    }
}

/// Statusy, w których proces przyjmuje wpis (działa albo czeka na logowanie).
fn is_input_ready(status: AccountStatus) -> bool {
    matches!(
        status,
        AccountStatus::Running | AccountStatus::AwaitingLogin
    )
}

/// Zapisuje jedną linię do `stdin` procesu (z terminatora `\n`).
async fn write_line(entry: &mut Running, line: &str) -> Result<(), AppError> {
    let stdin = entry
        .stdin
        .as_mut()
        .ok_or_else(|| AppError::new("process.stdinClosed"))?;
    if let Err(error) = stdin.write_all(line.as_bytes()).await {
        return Err(io_error("process.writeFailed", &error));
    }
    if let Err(error) = stdin.write_all(b"\n").await {
        return Err(io_error("process.writeFailed", &error));
    }
    if let Err(error) = stdin.flush().await {
        return Err(io_error("process.writeFailed", &error));
    }
    Ok(())
}

/// Uruchamia zadanie czytające strumień linia po linii i emitujące `mcc://log`.
///
/// Czytamy bajty i dekodujemy je stratnie (`from_utf8_lossy`), a nie przez
/// `Lines::next_line`, bo MCC potrafi wypisać bajty niebędące poprawnym UTF-8
/// (np. tekst serwera w innym kodowaniu). `next_line` zwróciłby wtedy błąd
/// i **uciszył resztę strumienia** konta - niedopuszczalne przy priorytecie
/// "UI nigdy się nie zawiesza". Stratna dekodacja zamienia złe bajty na `\u{FFFD}`
/// i pozwala czytać dalej.
async fn read_lines<S, R>(
    running: Arc<Mutex<HashMap<String, Running>>>,
    sink: Arc<S>,
    connection_id: String,
    reader: R,
    stream: LogStream,
) where
    S: EventSink,
    R: AsyncRead + Unpin + Send + 'static,
{
    let mut reader = BufReader::new(reader);
    let mut buffer: Vec<u8> = Vec::with_capacity(256);
    loop {
        buffer.clear();
        match reader.read_until(b'\n', &mut buffer).await {
            Ok(0) => break,
            Ok(_) => {
                // Zdejmujemy terminator linii (`\n` albo `\r\n`).
                if buffer.last() == Some(&b'\n') {
                    buffer.pop();
                }
                if buffer.last() == Some(&b'\r') {
                    buffer.pop();
                }
                let line = String::from_utf8_lossy(&buffer).into_owned();
                // Licznik linii odebranych z MCC, tylko przy diagnostyce
                // (`MCC_GUI_DIAG=1` - patrz `src-tauri/src/diag.rs`).
                crate::diag::count_read();

                // Wykrywanie promptu logowania i połączenia robimy **przed**
                // wysłaniem linii, żeby banner i status były już aktualne, gdy UI
                // dostanie tę linię logu.
                if stream == LogStream::Stdout {
                    handle_stdout_signals(&running, &sink, &connection_id, &line).await;
                }

                sink.emit_log(&connection_id, stream, line, now_ms());
                // Licznik linii wysłanych do UI (patrz `src-tauri/src/diag.rs`).
                crate::diag::count_emitted();
            }
            // Błąd odczytu pojedynczej linii nie przerywa całego strumienia.
            Err(error) => {
                tracing::warn!(connection = %connection_id, %error, "błąd odczytu strumienia MCC");
                break;
            }
        }
    }
}

/// Rozpoznaje w linii `stdout` prompt logowania i moment połączenia.
/// Każde zdarzenie wysyłamy co najwyżej raz na uruchomienie procesu.
async fn handle_stdout_signals<S>(
    running: &Arc<Mutex<HashMap<String, Running>>>,
    sink: &Arc<S>,
    connection_id: &str,
    line: &str,
) where
    S: EventSink,
{
    let mut map = running.lock().await;
    let Some(entry) = map.get_mut(connection_id) else {
        return;
    };

    if !entry.login_prompted
        && let Some(prompt) = device_code::detect_device_code(line)
    {
        entry.login_prompted = true;
        entry.status = AccountStatus::AwaitingLogin;
        sink.emit_device_code(connection_id, prompt.url, prompt.code);
        sink.emit_status(connection_id, AccountStatus::AwaitingLogin);
        tracing::info!(connection = %connection_id, "wykryto prompt logowania Microsoft");
        return;
    }

    if !entry.joined && device_code::is_joined(line) {
        entry.joined = true;
        // Powrót do `running` tylko z `awaiting_login`; przy ponownym połączeniu
        // (reconnect MCC) status jest już `running` i nie ma czego zmieniać.
        if entry.status == AccountStatus::AwaitingLogin {
            entry.status = AccountStatus::Running;
            sink.emit_status(connection_id, AccountStatus::Running);
        }
        sink.emit_joined(connection_id);
        tracing::info!(connection = %connection_id, "klient połączył się z serwerem");
    }
}

/// Podłącza czytnik strumienia, jeśli strumień istnieje.
fn spawn_reader<S, R>(
    running: Arc<Mutex<HashMap<String, Running>>>,
    sink: Arc<S>,
    connection_id: String,
    reader: Option<R>,
    stream: LogStream,
) where
    S: EventSink,
    R: AsyncRead + Unpin + Send + 'static,
{
    let Some(reader) = reader else {
        return;
    };
    tokio::spawn(read_lines(running, sink, connection_id, reader, stream));
}

/// Uruchamia strażnika, który czeka na zakończenie procesu i emituje status.
fn spawn_waiter<S>(
    running: Arc<Mutex<HashMap<String, Running>>>,
    sink: Arc<S>,
    connection_id: String,
    server: String,
    mut child: Child,
    kill_rx: oneshot::Receiver<()>,
) where
    S: EventSink,
{
    tokio::spawn(async move {
        let natural = tokio::select! {
            result = child.wait() => Some(result),
            _ = kill_rx => None,
        };

        if natural.is_none() {
            let _ = child.start_kill();
            let _ = child.wait().await;
        }

        let status = {
            let mut map = running.lock().await;
            map.remove(&connection_id).map(|entry| {
                if entry.status == AccountStatus::Stopping {
                    AccountStatus::Stopped
                } else {
                    AccountStatus::Crashed
                }
            })
        };

        if let Some(status) = status {
            sink.emit_status(&connection_id, status);
            if status == AccountStatus::Crashed {
                // Do UI idzie **adres serwera**, nie surowy identyfikator
                // połączenia - komunikat ma być czytelny dla człowieka.
                sink.emit_error(
                    "process.exitedUnexpectedly",
                    json!({ "server": server, "accountId": connection_id.clone() }),
                );
            }
            tracing::info!(connection = %connection_id, ?status, "proces MCC zakończony");
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::account::AccountType;
    use crate::events::LogStream;
    use std::sync::Mutex;

    /// Zbiera linie w kolejności odbioru.
    #[derive(Default)]
    struct Collector {
        lines: Mutex<Vec<String>>,
    }

    impl EventSink for Collector {
        fn emit_log(&self, _account_id: &str, _stream: LogStream, line: String, _ts: i64) {
            self.lines.lock().expect("mutex").push(line);
        }

        fn emit_status(&self, _account_id: &str, _status: AccountStatus) {}

        fn emit_error(&self, _code: &str, _params: serde_json::Value) {}

        fn emit_device_code(&self, _account_id: &str, _url: String, _code: String) {}

        fn emit_joined(&self, _account_id: &str) {}
    }

    /// Pusty stan procesów - wykrywanie sygnałów nie ma nic do zrobienia.
    fn no_running() -> Arc<tokio::sync::Mutex<HashMap<String, Running>>> {
        Arc::new(tokio::sync::Mutex::new(HashMap::new()))
    }

    fn sample_account(id: &str, servers: &[&str]) -> Account {
        Account {
            id: id.into(),
            label: id.into(),
            account_type: AccountType::Offline,
            login: "Bot".into(),
            servers: servers.iter().map(|s| ServerEntry::auto(s)).collect(),
            tab_order: 0,
        }
    }

    fn temp_root(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("mcc-pm-{}-{tag}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("katalog testowy");
        dir
    }

    /// Niepoprawne bajty UTF-8 nie mogą uciąć strumienia - MCC bywa nieprzewidywalny.
    #[tokio::test]
    async fn reader_keeps_going_after_invalid_utf8() {
        let sink = Arc::new(Collector::default());
        // "abc" + bajt 0xFF (niepoprawny w UTF-8) + CRLF + poprawna linia.
        let raw: &[u8] = b"abc\xffdef\r\nnext\n";
        read_lines(
            no_running(),
            sink.clone(),
            "conn".to_string(),
            raw,
            LogStream::Stdout,
        )
        .await;

        let lines = sink.lines.lock().expect("mutex").clone();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].starts_with("abc"));
        assert!(lines[0].ends_with("def"));
        assert_eq!(lines[1], "next");
    }

    /// Terminator CRLF jest zdejmowany, a ostatnia linia bez `\n` też jest emitowana.
    #[tokio::test]
    async fn reader_strips_line_endings() {
        let sink = Arc::new(Collector::default());
        read_lines(
            no_running(),
            sink.clone(),
            "conn".to_string(),
            b"one\r\ntwo\nthree".as_slice(),
            LogStream::Stdout,
        )
        .await;

        let lines = sink.lines.lock().expect("mutex").clone();
        assert_eq!(
            lines,
            vec!["one".to_string(), "two".to_string(), "three".to_string()]
        );
    }

    /// Przywracanie kont nie uruchamia procesów, ale tworzy katalogi kont.
    #[tokio::test]
    async fn sync_accounts_registers_without_spawning() {
        let root = temp_root("sync");
        let sink = Arc::new(Collector::default());
        let manager = ProcessManager::new(None, root.clone(), sink).expect("menedżer");

        let account = sample_account("acc-restore", &["localhost:25565"]);
        manager.sync_accounts(std::slice::from_ref(&account)).await;

        assert!(manager.accounts.lock().await.contains_key("acc-restore"));
        assert!(manager.running.lock().await.is_empty());
        assert!(
            root.join("acc-restore").is_dir(),
            "katalog konta musi istnieć"
        );

        // Powtórna synchronizacja tylko aktualizuje konto, nie duplikuje wpisu.
        let mut updated = account.clone();
        updated.servers = vec![ServerEntry::auto("other.example.com:25565")];
        manager.sync_accounts(std::slice::from_ref(&updated)).await;
        let accounts = manager.accounts.lock().await;
        assert_eq!(accounts.len(), 1);
        assert_eq!(
            accounts.get("acc-restore").map(|a| a.servers.len()),
            Some(1)
        );
        drop(accounts);

        let _ = std::fs::remove_dir_all(&root);
    }

    /// Bez ścieżki MCC nie da się wystartować połączenia.
    #[tokio::test]
    async fn start_connection_requires_mcc_path() {
        let root = temp_root("noexe");
        let sink = Arc::new(Collector::default());
        let manager = ProcessManager::new(None, root.clone(), sink).expect("menedżer");
        manager
            .add_account(sample_account("a", &["host:1"]))
            .await
            .expect("dodanie konta");

        assert_eq!(
            manager
                .start_connection("a", "host:1")
                .await
                .unwrap_err()
                .code,
            "mcc.executableNotSet"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    /// Serwer musi być zapisany na koncie, inaczej start kończy się błędem.
    #[tokio::test]
    async fn start_connection_rejects_unknown_server() {
        let root = temp_root("unknown");
        let sink = Arc::new(Collector::default());
        let manager = ProcessManager::new(Some(PathBuf::from("nope.exe")), root.clone(), sink)
            .expect("menedżer");
        manager
            .add_account(sample_account("a", &["host:1"]))
            .await
            .expect("dodanie konta");

        assert_eq!(
            manager
                .start_connection("a", "other:2")
                .await
                .unwrap_err()
                .code,
            "connection.notFound"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    /// Duplikat serwera i zły adres są odrzucane.
    #[tokio::test]
    async fn add_server_upserts_version_and_validates() {
        let root = temp_root("addserver");
        let sink = Arc::new(Collector::default());
        let manager = ProcessManager::new(None, root.clone(), sink).expect("menedżer");
        manager
            .add_account(sample_account("a", &["Host:25565"]))
            .await
            .expect("dodanie konta");

        // Ten sam adres (inna wielkość liter) to aktualizacja wersji, nie duplikat.
        manager
            .add_server("a", "host:25565", Some("1.21"))
            .await
            .expect("aktualizacja wersji");
        let stored = manager.account("a").await.expect("konto");
        assert_eq!(stored.servers.len(), 1, "adres nie może się zdublować");
        assert_eq!(stored.servers[0].version.as_deref(), Some("1.21"));

        assert_eq!(
            manager.add_server("a", "   ", None).await.unwrap_err().code,
            "server.invalid"
        );
        assert_eq!(
            manager
                .add_server("a", "host:1", Some("1\n2"))
                .await
                .unwrap_err()
                .code,
            "server.invalidVersion"
        );
        assert_eq!(
            manager
                .add_server("nope", "x:1", None)
                .await
                .unwrap_err()
                .code,
            "account.notFound"
        );

        manager
            .add_server("a", "other:2", None)
            .await
            .expect("nowy serwer");
        let stored = manager.account("a").await.expect("konto");
        assert_eq!(stored.servers.len(), 2);

        manager
            .remove_server("a", "Host:25565")
            .await
            .expect("usunięcie serwera");
        let stored = manager.account("a").await.expect("konto");
        assert_eq!(stored.servers, vec![ServerEntry::auto("other:2")]);

        let _ = std::fs::remove_dir_all(&root);
    }

    /// Zatrzymanie nieuruchomionego połączenia jest bezpieczne.
    #[tokio::test]
    async fn stopping_a_idle_connection_is_ok() {
        let root = temp_root("idle");
        let sink = Arc::new(Collector::default());
        let manager = ProcessManager::new(None, root.clone(), sink).expect("menedżer");
        manager
            .stop_connection("nope::host")
            .await
            .expect("stop nieuruchomionego połączenia ma być bezpieczny");
        let _ = std::fs::remove_dir_all(&root);
    }
}

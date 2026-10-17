//! Zapisywanie logów czatu do plików (Paczka 4, opis w `PLAN.md` §5).
//!
//! Układ plików: `<folder logów>/<serwer>/<konto>/<RRRR-MM-DD>.log`. Wszystkie
//! sesje tego samego dnia dopisują się do jednego pliku, a każdą sesję otwiera
//! i zamyka wiersz-znacznik. Po północy (czas lokalny) zapis przechodzi do pliku
//! z nową datą, także w działającej sesji.
//!
//! Czego tu **nigdzie** nie zapisujemy: promptu logowania Microsoft (kod
//! urządzenia i link). Filtrujemy go przed wrzuceniem linii do kolejki, korzystając
//! z wykrywania w `device_code` - jedynego miejsca, które zna ten wzorzec.
//!
//! Priorytet nr 1 z `AGENTS.md` („UI nigdy się nie zawiesza”) wymusza kształt
//! tego modułu: czytanie wyjścia MCC tylko **wrzuca** linię do kolejki o ograniczonej
//! pojemności (`try_send`, bez czekania), a całe I/O robi osobne zadanie `tokio`
//! jedno na połączenie. Gdy kolejka się przepełni, linię gubimy, liczymy
//! i zwracamy w pliku jednym wierszem „N linii pominięto”.
//!
//! `LocalDateTime` bierzemy z systemu (`GetLocalTime`) z istniejącej zależności
//! `windows` - nie potrzebujemy do tego `chrono`/`time`.

use std::collections::HashMap;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, RwLock};
use std::time::Duration;

use serde_json::json;
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use crate::account::Account;
use crate::device_code;
use crate::error::AppError;
use crate::events::LogStream;
use crate::settings::{AppSettings, DEFAULT_CHAT_LOG_RETENTION_DAYS};

/// Podkatalog w katalogu danych aplikacji, w którym domyślnie leżą logi czatu.
pub const LOGS_DIR: &str = "logs";

/// Pojemność kolejki zapisu na jedno połączenie (linie). Ograniczona z założenia:
/// wolny dysk nie może zamienić się w rosnącą w nieskończoność pamięć.
pub const QUEUE_CAPACITY: usize = 4096;

/// Co ile najwyżej zrzucamy bufor na dysk.
const FLUSH_INTERVAL: Duration = Duration::from_secs(1);

/// Ile jeszcze zbieramy linie po zamknięciu procesu. Status `stopped` wysyłamy,
/// gdy proces się zakończy, a czytniki strumieni mogą mieć wtedy jeszcze bufor
/// do odebrania - bez tego okresu końcówka logu traciłaby ostatnie linie.
const CLOSE_GRACE: Duration = Duration::from_millis(400);

/// Ile czekamy na dopisanie plików przy zamykaniu aplikacji.
const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(3);

/// Najdłuższa nazwa katalogu/pliku, jaką tworzymy (ochrona przed `MAX_PATH`).
const MAX_SEGMENT_LEN: usize = 64;

/// Znacznik początku sesji w pliku logu.
const SESSION_START: &str = "session start";

/// Znacznik końca sesji w pliku logu.
const SESSION_END: &str = "session end";

/// Data i czas lokalny - to, co trafia do wiersza `[HH:mm:ss]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalDateTime {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

impl LocalDateTime {
    pub fn new(year: u16, month: u8, day: u8, hour: u8, minute: u8, second: u8) -> Self {
        Self {
            year,
            month,
            day,
            hour,
            minute,
            second,
        }
    }

    /// Klucz daty `RRRR-MM-DD` (jednocześnie nazwa pliku bez rozszerzenia).
    pub fn date(&self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }

    /// Godzina w formacie `HH:MM:SS` (prefiks wiersza logu).
    pub fn time(&self) -> String {
        format!("{:02}:{:02}:{:02}", self.hour, self.minute, self.second)
    }

    /// Pełny znacznik czasu `RRRR-MM-DD HH:MM:SS` (wiersze-znaczniki sesji).
    pub fn stamp(&self) -> String {
        format!("{} {}", self.date(), self.time())
    }
}

/// Bieżący czas lokalny.
///
/// Na Windows bierzemy go wprost z systemu (`GetLocalTime`) - dzięki temu nie
/// potrzebujemy zależności od `chrono`/`time`, a strefa czasowa jest uwzględniona
/// razem ze zmianą czasu letniego. Poza Windows (buildy testowe) zwracamy UTC.
pub fn local_now() -> LocalDateTime {
    #[cfg(windows)]
    {
        use windows::Win32::System::SystemInformation::GetLocalTime;

        // Bezpieczne: `GetLocalTime` wypełnia przekazaną strukturę i nie zwraca błędu.
        let time = unsafe { GetLocalTime() };
        LocalDateTime::new(
            time.wYear,
            time.wMonth as u8,
            time.wDay as u8,
            time.wHour as u8,
            time.wMinute as u8,
            time.wSecond as u8,
        )
    }
    #[cfg(not(windows))]
    {
        let seconds = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or(0);
        let (year, month, day) = civil_from_days(seconds.div_euclid(86_400));
        let rest = seconds.rem_euclid(86_400);
        LocalDateTime::new(
            year as u16,
            month as u8,
            day as u8,
            (rest / 3600) as u8,
            ((rest % 3600) / 60) as u8,
            (rest % 60) as u8,
        )
    }
}

/// Dziś jako `(rok, miesiąc, dzień)` w czasie lokalnym.
pub fn today() -> (i64, u32, u32) {
    let now = local_now();
    (
        i64::from(now.year),
        u32::from(now.month),
        u32::from(now.day),
    )
}

/// Liczba dni od 1970-01-01 dla daty gregoriańskiej (algorytm cywilny Hinnanta).
fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = year - era * 400;
    let month_prime = (month + 9) % 12;
    let day_of_year = (153 * month_prime + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

/// Data dla liczby dni od 1970-01-01 (odwrotność `days_from_civil`).
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let shifted = days + 719_468;
    let era = if shifted >= 0 {
        shifted
    } else {
        shifted - 146_096
    } / 146_097;
    let day_of_era = shifted - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = if month_prime < 10 {
        month_prime + 3
    } else {
        month_prime - 9
    };
    (
        if month <= 2 { year + 1 } else { year },
        month as u32,
        day as u32,
    )
}

/// Klucz `RRRR-MM-DD` daty „dziś minus `days` dni” (granica retencji).
pub fn date_key_minus_days(today: (i64, u32, u32), days: u32) -> String {
    let since_epoch =
        days_from_civil(today.0, i64::from(today.1), i64::from(today.2)) - i64::from(days);
    let (year, month, day) = civil_from_days(since_epoch);
    format!("{year:04}-{month:02}-{day:02}")
}

/// Czy tekst jest poprawnym kluczem daty (`RRRR-MM-DD`) i czy taka data istnieje.
pub fn is_date_key(value: &str) -> bool {
    let parts: Vec<&str> = value.split('-').collect();
    if parts.len() != 3 || parts[0].len() != 4 || parts[1].len() != 2 || parts[2].len() != 2 {
        return false;
    }
    let (Ok(year), Ok(month), Ok(day)) = (
        parts[0].parse::<i64>(),
        parts[1].parse::<u32>(),
        parts[2].parse::<u32>(),
    ) else {
        return false;
    };
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return false;
    }
    // Data musi naprawdę istnieć (np. 31 lutego jest odrzucane).
    civil_from_days(days_from_civil(year, i64::from(month), i64::from(day))) == (year, month, day)
}

/// Czy nazwa jest zarezerwowana przez Windows (nie da się jej użyć jako nazwy
/// pliku ani katalogu, także z rozszerzeniem).
fn is_reserved_name(name: &str) -> bool {
    let stem = name
        .split('.')
        .next()
        .unwrap_or(name)
        .trim_end_matches([' ', '.'])
        .to_ascii_uppercase();
    matches!(
        stem.as_str(),
        "CON"
            | "PRN"
            | "AUX"
            | "NUL"
            | "COM1"
            | "COM2"
            | "COM3"
            | "COM4"
            | "COM5"
            | "COM6"
            | "COM7"
            | "COM8"
            | "COM9"
            | "LPT1"
            | "LPT2"
            | "LPT3"
            | "LPT4"
            | "LPT5"
            | "LPT6"
            | "LPT7"
            | "LPT8"
            | "LPT9"
    )
}

/// Oczyszcza nazwę katalogu albo pliku, żeby dała się utworzyć na Windows
/// i żeby nie dało się nią wyjść poza folder logów.
///
/// Wymieniamy znaki niedozwolone w nazwach (`< > : " / \ | ? *` i znaki sterujące)
/// na `_`, obcinamy końcowe kropki i spacje, skracamy bardzo długie nazwy,
/// a nazwy zarezerwowane (np. `NUL`) i „puste” (`.`, `..`) zamieniamy na bezpieczne.
pub fn sanitize_segment(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len().min(MAX_SEGMENT_LEN));
    for (length, ch) in raw.trim().chars().enumerate() {
        if length >= MAX_SEGMENT_LEN {
            break;
        }
        let forbidden =
            ch.is_control() || matches!(ch, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*');
        out.push(if forbidden { '_' } else { ch });
    }

    // Windows nie pozwala na nazwy kończące się kropką ani spacją.
    while out.ends_with(['.', ' ']) {
        out.pop();
    }

    if out.is_empty() || out == "." || out == ".." {
        return "_".to_string();
    }
    if is_reserved_name(&out) {
        out.insert(0, '_');
    }
    out
}

/// Ścieżka pliku logu: `<root>/<serwer>/<konto>/<RRRR-MM-DD>.log`.
pub fn log_file_path(root: &Path, server: &str, account: &str, date: &str) -> PathBuf {
    root.join(sanitize_segment(server))
        .join(sanitize_segment(account))
        .join(format!("{date}.log"))
}

/// Czy ścieżka leży wewnątrz folderu logów (żadnego `..`, dysku ani korzenia).
///
/// Ścieżkę budujemy z nazw po `sanitize_segment`, więc to jest tylko
/// zabezpieczenie „na wszelki wypadek”.
pub fn is_inside(root: &Path, path: &Path) -> bool {
    let Ok(relative) = path.strip_prefix(root) else {
        return false;
    };
    relative
        .components()
        .all(|component| matches!(component, Component::Normal(_)))
}

/// Wiersz logu: `[HH:mm:ss] tekst`, bez kodów `§` i ANSI.
pub fn format_line(stamp: &LocalDateTime, stream: LogStream, text: &str) -> String {
    let plain = device_code::strip_formatting(text);
    // W pliku jedna linia ma być jedną linią - żaden znak sterujący z wyjścia
    // MCC nie może go rozbić na dwa wiersze.
    let single: String = plain
        .chars()
        .map(|ch| if ch.is_control() { ' ' } else { ch })
        .collect();
    let prefix = match stream {
        LogStream::Stderr => "[stderr] ",
        LogStream::Stdout => "",
    };
    format!("[{}] {prefix}{single}", stamp.time())
}

/// Wiersz-znacznik generowany przez aplikację (`=== session start ... ===`).
fn marker(stamp: &LocalDateTime, kind: &str) -> String {
    format!("=== {kind} {} ===\n", stamp.stamp())
}

/// Wiersz informujący o liniach zgubionych przy przepełnionej kolejce.
fn omitted_line(stamp: &LocalDateTime, count: u64) -> String {
    format!(
        "[{}] === {count} lines omitted (queue full) ===\n",
        stamp.time()
    )
}

/// Pliki logów starsze niż `retention_days` (`0` = bez limitu), które wolno usunąć.
///
/// Reguła jest celowo wąska: patrzymy **wyłącznie** na pliki o nazwie
/// `RRRR-MM-DD.log` leżące dokładnie na głębokości `<serwer>/<konto>/`. Wszystko
/// inne w folderze logów (obce pliki, katalogi, głębsze poziomy) zostaje nietknięte.
pub fn expired_logs(root: &Path, today: (i64, u32, u32), retention_days: u32) -> Vec<PathBuf> {
    if retention_days == 0 {
        return Vec::new();
    }
    let cutoff = date_key_minus_days(today, retention_days);
    let mut expired = Vec::new();

    let Ok(servers) = std::fs::read_dir(root) else {
        return expired;
    };
    for server in servers.flatten() {
        if !is_dir(&server.path()) {
            continue;
        }
        let Ok(accounts) = std::fs::read_dir(server.path()) else {
            continue;
        };
        for account in accounts.flatten() {
            if !is_dir(&account.path()) {
                continue;
            }
            let Ok(files) = std::fs::read_dir(account.path()) else {
                continue;
            };
            for file in files.flatten() {
                if !file.file_type().map(|kind| kind.is_file()).unwrap_or(false) {
                    continue;
                }
                let name = file.file_name();
                let Some(name) = name.to_str() else {
                    continue;
                };
                let Some(date) = name.strip_suffix(".log") else {
                    continue;
                };
                // Klucze daty mają stałą szerokość, więc porównanie tekstów
                // jest porównaniem chronologicznym.
                if !is_date_key(date) || date >= cutoff.as_str() {
                    continue;
                }
                expired.push(file.path());
            }
        }
    }
    expired
}

/// Czy pod ścieżką jest **zwykły** katalog (błąd odczytu traktujemy jak „nie”).
///
/// Idziemy przez `symlink_metadata`, więc nie podążamy za dowiązaniami ani
/// junctionami: inaczej sprzątanie wyszłoby poza katalog logów i usunęło pliki
/// `RRRR-MM-DD.log` zupełnie gdzie indziej (R-11). Reparse point to dla nas
/// „nie katalog” - sprzątanie jest wtedy konserwatywne, nigdy agresywne.
fn is_dir(path: &Path) -> bool {
    let Ok(meta) = std::fs::symlink_metadata(path) else {
        return false;
    };
    meta.is_dir() && !is_reparse_point(&meta)
}

/// Czy metadane wskazują na reparse point (dowiązanie symboliczne albo junction).
///
/// Na Windows każdy reparse point ma odpowiedni atrybut; poza Windows sprawdzamy
/// zwykłe dowiązanie symboliczne.
#[cfg(windows)]
fn is_reparse_point(meta: &std::fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;

    /// `FILE_ATTRIBUTE_REPARSE_POINT` - wspólny znacznik dowiązań i junctionów.
    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;

    meta.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
}

#[cfg(not(windows))]
fn is_reparse_point(meta: &std::fs::Metadata) -> bool {
    meta.file_type().is_symlink()
}

/// Usuwa przedawnione pliki logów. Zwraca liczbę usuniętych plików.
pub fn purge_expired(root: &Path, today: (i64, u32, u32), retention_days: u32) -> usize {
    let mut removed = 0;
    for path in expired_logs(root, today, retention_days) {
        match std::fs::remove_file(&path) {
            Ok(()) => {
                removed += 1;
                tracing::debug!(file = %path.display(), "logi czatu: usunięto przedawniony plik");
            }
            Err(error) => {
                tracing::warn!(%error, file = %path.display(), "logi czatu: nie udało się usunąć przedawnionego pliku");
            }
        }
    }
    if removed > 0 {
        tracing::info!(removed, "logi czatu: czyszczenie zakończone");
    }
    removed
}

/// Konfiguracja zapisu logów czatu (odzwierciedla ustawienia aplikacji).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatLogConfig {
    pub enabled: bool,
    pub root: PathBuf,
    pub retention_days: u32,
}

impl ChatLogConfig {
    /// Buduje konfigurację z ustawień; brak własnego folderu oznacza folder domyślny.
    pub fn from_settings(settings: &AppSettings, default_root: PathBuf) -> Self {
        Self {
            enabled: settings.chat_log_enabled,
            root: settings
                .chat_log_dir
                .as_deref()
                .map(PathBuf::from)
                .unwrap_or(default_root),
            retention_days: settings.chat_log_retention_days,
        }
    }
}

/// Dokąd trafiają logi jednego połączenia.
#[derive(Debug, Clone)]
struct Target {
    root: PathBuf,
    server: String,
    account_id: String,
    account_label: String,
}

/// Zdarzenie odebrane przez zadanie zapisujące jedno połączenie.
enum Entry {
    /// Linia wyjścia MCC, jeszcze z kodami `§` (formatowanie robimy dopiero
    /// przy zapisie, żeby kolejka niosła jak najmniej danych).
    Line {
        stamp: LocalDateTime,
        stream: LogStream,
        text: String,
    },
    /// Prośba o zamknięcie pliku znacznikiem końca sesji.
    End(LocalDateTime),
}

/// Co przyniósł jeden obieg pętli zadania zapisującego.
enum Next {
    Entry(Entry),
    Tick,
    Closed,
    GraceExpired,
}

/// Jedna sesja zapisu (połączenie). `done` bierzemy tylko przy zamykaniu aplikacji.
struct Session {
    generation: u64,
    tx: mpsc::Sender<Entry>,
    dropped: Arc<AtomicU64>,
    done: Option<JoinHandle<()>>,
}

/// Blokada odporna na zatrucie - panika innego wątku nie może wyłączyć logów.
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|error| error.into_inner())
}

/// Serwis logów czatu.
///
/// Metody wywoływane w ścieżce czytania wyjścia MCC (`begin_session`,
/// `write_line`, `end_session`) są **synchroniczne i nieblokujące** - nigdy nie
/// czekają na dysk.
pub struct ChatLog {
    /// Folder domyślny (`<katalog danych>/logs`), używany bez własnego wyboru.
    default_root: PathBuf,
    /// Konfiguracja z ustawień: folder i retencja.
    config: RwLock<ChatLogConfig>,
    /// Szybki warunek dla każdej linii, żeby nie brać blokady w pętli odczytu.
    enabled: AtomicBool,
    /// Etykiety kont: to z nich powstaje nazwa katalogu (`<serwer>/<etykieta>`).
    labels: RwLock<HashMap<String, String>>,
    /// Aktywne sesje, klucz to identyfikator połączenia.
    sessions: Mutex<HashMap<String, Session>>,
    next_generation: AtomicU64,
    /// Kanał błędów zapisu; w aplikacji trafiają stąd jako `app://error` (toast).
    errors: mpsc::UnboundedSender<AppError>,
}

impl ChatLog {
    /// Tworzy serwis. `default_root` to `<katalog danych aplikacji>/logs`,
    /// a `errors` to kanał, którym błędy zapisu dochodzą do interfejsu.
    pub fn new(default_root: PathBuf, errors: mpsc::UnboundedSender<AppError>) -> Arc<Self> {
        Arc::new(Self {
            config: RwLock::new(ChatLogConfig {
                enabled: false,
                root: default_root.clone(),
                retention_days: DEFAULT_CHAT_LOG_RETENTION_DAYS,
            }),
            default_root,
            enabled: AtomicBool::new(false),
            labels: RwLock::new(HashMap::new()),
            sessions: Mutex::new(HashMap::new()),
            next_generation: AtomicU64::new(1),
            errors,
        })
    }

    /// Ustawia konfigurację z ustawień aplikacji.
    ///
    /// Zmiana folderu albo wyłączenie zapisu zamyka bieżące sesje znacznikiem
    /// końca - żaden plik nie zostaje urwany w połowie.
    pub fn apply_settings(&self, settings: &AppSettings) {
        let next = ChatLogConfig::from_settings(settings, self.default_root.clone());
        let changed = {
            let mut config = self
                .config
                .write()
                .unwrap_or_else(|error| error.into_inner());
            let changed = config.enabled != next.enabled || config.root != next.root;
            *config = next.clone();
            changed
        };
        self.enabled.store(next.enabled, Ordering::Relaxed);
        if changed {
            self.close_sessions();
        }
    }

    /// Bieżąca konfiguracja (folder, retencja, włączenie).
    pub fn config(&self) -> ChatLogConfig {
        self.config
            .read()
            .unwrap_or_else(|error| error.into_inner())
            .clone()
    }

    /// Efektywny folder logów (własny użytkownika albo domyślny).
    pub fn root(&self) -> PathBuf {
        self.config().root
    }

    /// Ile dni trzymamy logi (`0` = bez limitu).
    pub fn retention_days(&self) -> u32 {
        self.config().retention_days
    }

    /// Czy zapis logów czatu jest włączony.
    pub fn enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    /// Podstawia etykiety kont (pełna lista z `accounts.json`).
    pub fn set_account_labels(&self, accounts: &[Account]) {
        let mut labels = self
            .labels
            .write()
            .unwrap_or_else(|error| error.into_inner());
        labels.clear();
        for account in accounts {
            labels.insert(account.id.clone(), account.label.clone());
        }
    }

    /// Zaczyna sesję zapisu dla połączenia.
    ///
    /// Wołane raz przy starcie procesu (`status = starting`). Ponowne wołanie dla
    /// tego samego połączenia zastępuje poprzednią sesję (jej zadanie kończy się samo).
    pub fn begin_session(self: &Arc<Self>, connection_id: &str, server: &str, account_id: &str) {
        if !self.enabled() {
            return;
        }
        // Bez środowiska asynchronicznego nie ma gdzie uruchomić zapisu; lepiej
        // nie zapisywać niż panikować.
        let Ok(runtime) = tokio::runtime::Handle::try_current() else {
            tracing::warn!("logi czatu: brak środowiska asynchronicznego, zapis pominięty");
            return;
        };

        let target = Target {
            root: self.root(),
            server: server.to_string(),
            account_id: account_id.to_string(),
            account_label: self.account_label(account_id),
        };
        let generation = self.next_generation.fetch_add(1, Ordering::Relaxed);
        let (tx, rx) = mpsc::channel(QUEUE_CAPACITY);
        let dropped = Arc::new(AtomicU64::new(0));

        {
            let mut sessions = lock(&self.sessions);
            sessions.insert(
                connection_id.to_string(),
                Session {
                    generation,
                    tx,
                    dropped: Arc::clone(&dropped),
                    done: None,
                },
            );
        }

        let handle = runtime.spawn(run_session(
            Arc::clone(self),
            connection_id.to_string(),
            generation,
            target,
            rx,
            dropped,
        ));

        let mut sessions = lock(&self.sessions);
        if let Some(session) = sessions.get_mut(connection_id)
            && session.generation == generation
        {
            session.done = Some(handle);
        }
    }

    /// Dopisuje linię wyjścia MCC do pliku (przez kolejkę).
    ///
    /// Nigdy nie blokuje: przy przepełnionej kolejce linię gubimy i liczymy.
    pub fn write_line(&self, connection_id: &str, stream: LogStream, line: &str) {
        if !self.enabled() {
            return;
        }
        // Prompt logowania (kod urządzenia i link) nigdy nie trafia do pliku.
        if device_code::is_login_prompt(line) {
            return;
        }

        let entry = Entry::Line {
            stamp: local_now(),
            stream,
            text: line.to_string(),
        };
        let sessions = lock(&self.sessions);
        let Some(session) = sessions.get(connection_id) else {
            return;
        };
        if let Err(mpsc::error::TrySendError::Full(_)) = session.tx.try_send(entry) {
            session.dropped.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Prosi o zamknięcie sesji (znacznik końca i flush).
    ///
    /// Wpisu nie usuwamy od razu: dopiero po krótkim okresie łaski, w którym
    /// zapisujemy jeszcze linie dochodzące po zakończeniu procesu.
    pub fn end_session(&self, connection_id: &str) {
        let Some(tx) = ({
            let sessions = lock(&self.sessions);
            sessions
                .get(connection_id)
                .map(|session| session.tx.clone())
        }) else {
            return;
        };

        match tx.try_send(Entry::End(local_now())) {
            Ok(()) => {}
            Err(mpsc::error::TrySendError::Full(entry)) => {
                // Kolejka pełna: wysyłamy w tle, żeby znacznik końca nie zginął.
                if let Ok(runtime) = tokio::runtime::Handle::try_current() {
                    runtime.spawn(async move {
                        let _ = tx.send(entry).await;
                    });
                }
            }
            Err(mpsc::error::TrySendError::Closed(_)) => {}
        }
    }

    /// Zamyka wszystkie sesje i czeka, aż pliki zostaną dopisane.
    /// Wołane przy zamykaniu aplikacji, po zatrzymaniu procesów.
    pub async fn close_all(&self) {
        let stamp = local_now();
        let sessions: Vec<(mpsc::Sender<Entry>, Option<JoinHandle<()>>)> = {
            let mut sessions = lock(&self.sessions);
            sessions
                .values_mut()
                .map(|session| (session.tx.clone(), session.done.take()))
                .collect()
        };

        let mut handles = Vec::new();
        for (tx, handle) in sessions {
            let _ = tokio::time::timeout(SHUTDOWN_TIMEOUT, tx.send(Entry::End(stamp))).await;
            if let Some(handle) = handle {
                handles.push(handle);
            }
        }
        for handle in handles {
            if tokio::time::timeout(SHUTDOWN_TIMEOUT, handle)
                .await
                .is_err()
            {
                tracing::warn!("logi czatu: zapis nie zakończył się w wyznaczonym czasie");
            }
        }
    }

    /// Kończy wszystkie aktywne sesje (bez czekania na zadania).
    fn close_sessions(&self) {
        let ids: Vec<String> = {
            let sessions = lock(&self.sessions);
            sessions.keys().cloned().collect()
        };
        for id in ids {
            self.end_session(&id);
        }
    }

    /// Nazwa katalogu konta: jego etykieta, a gdy jej nie znamy - identyfikator.
    fn account_label(&self, account_id: &str) -> String {
        self.labels
            .read()
            .unwrap_or_else(|error| error.into_inner())
            .get(account_id)
            .cloned()
            .unwrap_or_else(|| account_id.to_string())
    }

    /// Usuwa wpis sesji, jeśli to wciąż ta sama sesja (nowa mogła już wystartować).
    fn drop_session(&self, connection_id: &str, generation: u64) {
        let mut sessions = lock(&self.sessions);
        if sessions
            .get(connection_id)
            .is_some_and(|session| session.generation == generation)
        {
            sessions.remove(connection_id);
        }
    }

    /// Zgłasza błąd zapisu: jeden toast z kodem błędu i parametrami.
    fn report_write(&self, target: &Target, connection_id: &str, error: &std::io::Error) {
        tracing::warn!(%error, connection = %connection_id, "logi czatu: błąd zapisu");
        let _ = self.errors.send(AppError::with_params(
            "chatlog.writeFailed",
            json!({
                "accountId": target.account_id,
                "server": target.server,
                "osError": error.raw_os_error().unwrap_or(-1),
            }),
        ));
    }
}

/// Otwiera (tworzy) plik logu dla danej daty.
async fn open_log_file(target: &Target, date: &str) -> std::io::Result<BufWriter<tokio::fs::File>> {
    let path = log_file_path(&target.root, &target.server, &target.account_label, date);
    if !is_inside(&target.root, &path) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "ścieżka logu poza folderem logów",
        ));
    }
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let file = tokio::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .await?;
    Ok(BufWriter::new(file))
}

/// Zadanie zapisujące jedno połączenie: czyta kolejkę i pisze plik.
///
/// Wszystkie błędy kończą zadanie (i zgłaszają jeden toast) - zapis dla tego
/// połączenia zostaje wyłączony do następnego startu.
async fn run_session(
    log: Arc<ChatLog>,
    connection_id: String,
    generation: u64,
    target: Target,
    mut rx: mpsc::Receiver<Entry>,
    dropped: Arc<AtomicU64>,
) {
    let mut file: Option<BufWriter<tokio::fs::File>> = None;
    let mut current_date = String::new();
    let mut pending_end: Option<LocalDateTime> = None;
    let mut ticker = tokio::time::interval(FLUSH_INTERVAL);
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

    loop {
        let next = if pending_end.is_some() {
            // Okres łaski: jeszcze zbieramy linie, potem zamykamy plik znacznikiem.
            match tokio::time::timeout(CLOSE_GRACE, rx.recv()).await {
                Ok(Some(entry)) => Next::Entry(entry),
                Ok(None) => Next::Closed,
                Err(_) => Next::GraceExpired,
            }
        } else {
            tokio::select! {
                entry = rx.recv() => match entry {
                    Some(entry) => Next::Entry(entry),
                    None => Next::Closed,
                },
                _ = ticker.tick() => Next::Tick,
            }
        };

        match next {
            Next::Tick => {
                if let Some(writer) = file.as_mut()
                    && let Err(error) = writer.flush().await
                {
                    log.report_write(&target, &connection_id, &error);
                    break;
                }
            }
            Next::Entry(Entry::End(stamp)) => {
                if pending_end.is_none() {
                    pending_end = Some(stamp);
                }
            }
            Next::Entry(Entry::Line {
                stamp,
                stream,
                text,
            }) => {
                // Zmiana dnia otwiera nowy plik, nawet w działającej sesji.
                let date = stamp.date();
                if file.is_none() || date != current_date {
                    if let Some(mut previous) = file.take()
                        && let Err(error) = previous.flush().await
                    {
                        log.report_write(&target, &connection_id, &error);
                        break;
                    }
                    match open_log_file(&target, &date).await {
                        Ok(mut writer) => {
                            let head = marker(&stamp, SESSION_START);
                            if let Err(error) = writer.write_all(head.as_bytes()).await {
                                log.report_write(&target, &connection_id, &error);
                                break;
                            }
                            file = Some(writer);
                            current_date = date;
                        }
                        Err(error) => {
                            log.report_write(&target, &connection_id, &error);
                            break;
                        }
                    }
                }

                let Some(writer) = file.as_mut() else {
                    break;
                };
                let omitted = dropped.swap(0, Ordering::Relaxed);
                if omitted > 0 {
                    let line = omitted_line(&stamp, omitted);
                    if let Err(error) = writer.write_all(line.as_bytes()).await {
                        log.report_write(&target, &connection_id, &error);
                        break;
                    }
                }
                let line = format!("{}\n", format_line(&stamp, stream, &text));
                if let Err(error) = writer.write_all(line.as_bytes()).await {
                    log.report_write(&target, &connection_id, &error);
                    break;
                }
            }
            Next::Closed | Next::GraceExpired => {
                if let Some(stamp) = pending_end.take()
                    && let Some(writer) = file.as_mut()
                {
                    let tail = marker(&stamp, SESSION_END);
                    if let Err(error) = writer.write_all(tail.as_bytes()).await {
                        log.report_write(&target, &connection_id, &error);
                    }
                }
                break;
            }
        }
    }

    if let Some(mut writer) = file.take()
        && let Err(error) = writer.flush().await
    {
        log.report_write(&target, &connection_id, &error);
    }
    log.drop_session(&connection_id, generation);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::account::{AccountType, ServerEntry};

    fn temp_root(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("mcc-chatlog-{}-{tag}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("katalog testowy");
        dir
    }

    fn account(id: &str, label: &str) -> Account {
        Account {
            id: id.to_string(),
            label: label.to_string(),
            account_type: AccountType::Offline,
            login: "Bot".to_string(),
            servers: vec![ServerEntry::auto("host:25565")],
            tab_order: 0,
        }
    }

    fn settings(enabled: bool, dir: Option<&str>, retention_days: u32) -> AppSettings {
        AppSettings {
            chat_log_enabled: enabled,
            chat_log_dir: dir.map(str::to_string),
            chat_log_retention_days: retention_days,
            ..Default::default()
        }
    }

    fn errors() -> (
        mpsc::UnboundedSender<AppError>,
        mpsc::UnboundedReceiver<AppError>,
    ) {
        mpsc::unbounded_channel()
    }

    /// Czeka, aż plik będzie zawierał `needle` (zapis jest asynchroniczny).
    async fn wait_for_file(path: &Path, needle: &str) -> String {
        let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
        loop {
            let content = std::fs::read_to_string(path).unwrap_or_default();
            if content.contains(needle) {
                return content;
            }
            assert!(
                tokio::time::Instant::now() < deadline,
                "brak {needle:?} w {}: {content}",
                path.display()
            );
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }

    #[test]
    fn sanitizes_windows_illegal_characters() {
        assert_eq!(
            sanitize_segment("play.example.com:25565"),
            "play.example.com_25565"
        );
        assert_eq!(sanitize_segment("a<b>c|d?e*f\"g"), "a_b_c_d_e_f_g");
        // Znaki sterujące (tu tabulator) też są niedozwolone w nazwach.
        assert_eq!(sanitize_segment("Anon\tBOT"), "Anon_BOT");
        // Końcowe kropki i spacje są na Windows niedozwolone.
        assert_eq!(sanitize_segment("Bot...  "), "Bot");
        assert_eq!(sanitize_segment("   "), "_");
        assert_eq!(sanitize_segment(".."), "_");
        assert_eq!(sanitize_segment("."), "_");
    }

    #[test]
    fn sanitizes_traversal_and_reserved_names() {
        // Próba wyjścia z folderu logów kończy się zwykłą nazwą.
        assert_eq!(sanitize_segment("../../evil"), ".._.._evil");
        assert_eq!(sanitize_segment("..\\..\\evil"), ".._.._evil");
        assert!(!sanitize_segment("../../evil").contains('/'));
        // Nazwy zarezerwowane przez Windows dostają prefiks.
        assert_eq!(sanitize_segment("NUL"), "_NUL");
        assert_eq!(sanitize_segment("com1.txt"), "_com1.txt");
    }

    #[test]
    fn sanitize_truncates_very_long_names() {
        let long = "x".repeat(500);
        assert_eq!(sanitize_segment(&long).chars().count(), MAX_SEGMENT_LEN);
    }

    #[test]
    fn path_stays_inside_logs_folder() {
        let root = PathBuf::from("C:/logs");
        let path = log_file_path(&root, "play.example.com:25565", "AnonBOT", "2026-10-16");
        assert_eq!(
            path,
            PathBuf::from("C:/logs")
                .join("play.example.com_25565")
                .join("AnonBOT")
                .join("2026-10-16.log")
        );
        assert!(is_inside(&root, &path));

        // Nawet podstawiona nazwa z `..` nie wyprowadza pliku z folderu logów.
        let sneaky = log_file_path(&root, "../../..", "../../../evil", "2026-10-16");
        assert!(
            is_inside(&root, &sneaky),
            "ścieżka musi zostać w folderze logów"
        );

        assert!(!is_inside(&root, &PathBuf::from("C:/logs/../evil.log")));
        assert!(!is_inside(&root, &PathBuf::from("C:/other/evil.log")));
    }

    #[test]
    fn line_format_has_time_and_no_color_codes() {
        let stamp = LocalDateTime::new(2026, 10, 16, 21, 3, 7);
        assert_eq!(
            format_line(&stamp, LogStream::Stdout, "§8[MCC] §aHello"),
            "[21:03:07] [MCC] Hello"
        );
        assert_eq!(
            format_line(&stamp, LogStream::Stdout, "\u{1b}[31mRed\u{1b}[0m"),
            "[21:03:07] Red"
        );
        assert_eq!(
            format_line(&stamp, LogStream::Stderr, "boom"),
            "[21:03:07] [stderr] boom"
        );
        // Znaki sterujące nie mogą rozbić jednej linii na dwie.
        assert_eq!(
            format_line(&stamp, LogStream::Stdout, "a\r\nb"),
            "[21:03:07] a  b"
        );
    }

    #[test]
    fn login_prompt_lines_are_filtered() {
        let stamp = LocalDateTime::new(2026, 10, 16, 12, 0, 0);
        let prompt = "To sign in, open https://www.microsoft.com/link in your browser and enter the code: §e7FRBLM2Y";
        assert!(device_code::is_login_prompt(prompt));
        // Zwykłe linie przechodzą.
        assert!(!device_code::is_login_prompt(
            "§8[MCC] Server was successfully joined."
        ));
        assert!(!device_code::is_login_prompt(
            "§8You chose to run in offline mode."
        ));
        // Kontrola: gdyby filtr zawiódł, kod zostałby zapisany w pliku.
        assert!(format_line(&stamp, LogStream::Stdout, "hello").contains("hello"));
    }

    #[test]
    fn date_key_arithmetic_crosses_month_and_year() {
        assert_eq!(date_key_minus_days((2026, 10, 16), 30), "2026-09-16");
        assert_eq!(date_key_minus_days((2026, 1, 1), 1), "2025-12-31");
        assert_eq!(date_key_minus_days((2024, 3, 1), 1), "2024-02-29");
        assert_eq!(date_key_minus_days((2026, 10, 16), 0), "2026-10-16");
    }

    #[test]
    fn invalid_date_keys_are_rejected() {
        assert!(is_date_key("2026-10-16"));
        assert!(is_date_key("2024-02-29"));
        assert!(!is_date_key("2026-02-30"));
        assert!(!is_date_key("2026-13-01"));
        assert!(!is_date_key("2026-1-1"));
        assert!(!is_date_key("20261016"));
        assert!(!is_date_key("not-a-date"));
        assert!(!is_date_key(""));
    }

    #[test]
    fn retention_removes_only_old_dated_logs() {
        let root = temp_root("retention");
        let server = root.join("play.example.com_25565");
        let account_dir = server.join("AnonBOT");
        std::fs::create_dir_all(&account_dir).expect("katalogi");

        let old = account_dir.join("2026-09-01.log");
        let old2 = account_dir.join("2026-09-15.log");
        let fresh = account_dir.join("2026-10-01.log");
        let today_file = account_dir.join("2026-10-16.log");
        for path in [&old, &old2, &fresh, &today_file] {
            std::fs::write(path, b"x").expect("plik logu");
        }
        // Obce pliki: inna nazwa, inne rozszerzenie, inna głębokość, sam folder.
        let foreign = account_dir.join("notes.txt");
        let rename = account_dir.join("2026-09-01.log.bak");
        let invalid = account_dir.join("2026-02-30.log");
        let deeper_dir = account_dir.join("extra");
        std::fs::create_dir_all(&deeper_dir).expect("katalog");
        let deeper = deeper_dir.join("2026-09-01.log");
        let date_dir = account_dir.join("2026-09-02.log");
        let at_root = root.join("2026-09-01.log");
        for path in [&foreign, &rename, &invalid, &deeper, &at_root] {
            std::fs::write(path, b"keep").expect("obcy plik");
        }
        std::fs::create_dir_all(&date_dir).expect("katalog o nazwie daty");

        let removed = purge_expired(&root, (2026, 10, 16), 30);
        assert_eq!(removed, 2, "usunięte mają być tylko dwa stare pliki logów");
        assert!(!old.exists() && !old2.exists());
        for path in [
            &fresh,
            &today_file,
            &foreign,
            &rename,
            &invalid,
            &deeper,
            &at_root,
        ] {
            assert!(path.exists(), "nie wolno usuwać {}", path.display());
        }
        assert!(date_dir.exists(), "katalogu o nazwie daty nie ruszamy");

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn retention_with_zero_days_keeps_everything() {
        let root = temp_root("retention-off");
        let account_dir = root.join("host").join("Bot");
        std::fs::create_dir_all(&account_dir).expect("katalogi");
        let old = account_dir.join("2000-01-01.log");
        std::fs::write(&old, b"x").expect("plik logu");

        assert!(expired_logs(&root, (2026, 10, 16), 0).is_empty());
        assert_eq!(purge_expired(&root, (2026, 10, 16), 0), 0);
        assert!(old.exists());

        let _ = std::fs::remove_dir_all(&root);
    }

    /// Tworzy junction (`mklink /J`) - nie wymaga uprawnień administratora.
    ///
    /// Zwraca `false`, gdy system nie pozwolił na utworzenie dowiązania.
    #[cfg(windows)]
    fn make_junction(link: &Path, target: &Path) -> bool {
        std::process::Command::new("cmd")
            .args(["/C", "mklink", "/J"])
            .arg(link)
            .arg(target)
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    #[test]
    #[cfg(windows)]
    fn retention_does_not_follow_junctions_outside_the_log_root() {
        let root = temp_root("retention-junction");
        // Dwa katalogi poza folderem logów: jeden udaje konto, drugi serwer.
        let outside_account = temp_root("retention-outside-account").join("Bot");
        let outside_server = temp_root("retention-outside-server");
        std::fs::create_dir_all(&outside_account).expect("katalog poza rootem");
        std::fs::create_dir_all(&outside_server).expect("katalog poza rootem");
        let victim_account = outside_account.join("2000-01-01.log");
        let victim_server = outside_server.join("2000-01-01.log");
        for victim in [&victim_account, &victim_server] {
            std::fs::write(victim, b"keep").expect("plik poza rootem");
        }

        // Junction na poziomie konta i na poziomie serwera.
        let server_dir = root.join("play.example.com_25565");
        std::fs::create_dir_all(&server_dir).expect("katalog serwera");
        let account_link = server_dir.join("Bot");
        let server_link = root.join("linked.example.com_25565");
        if !make_junction(&account_link, &outside_account)
            || !make_junction(&server_link, &outside_server)
        {
            eprintln!("pomijam: system nie pozwolił utworzyć junctionów");
            let _ = std::fs::remove_dir_all(&root);
            return;
        }

        // Sprzątanie nie może wejść za junction i usunąć plików poza folderem logów.
        assert!(expired_logs(&root, (2026, 10, 16), 30).is_empty());
        assert_eq!(purge_expired(&root, (2026, 10, 16), 30), 0);
        assert!(victim_account.exists(), "plik poza rootem ma zostać");
        assert!(victim_server.exists(), "plik poza rootem ma zostać");

        // Junction usuwamy jak zwykły katalog - `remove_dir_all` na dowiązaniu
        // mogłoby wejść do celu.
        let _ = std::fs::remove_dir(&account_link);
        let _ = std::fs::remove_dir(&server_link);
        let _ = std::fs::remove_dir_all(&root);
        let _ = std::fs::remove_dir_all(outside_account.parent().expect("katalog"));
        let _ = std::fs::remove_dir_all(&outside_server);
    }

    #[test]
    fn config_follows_settings_and_default_root() {
        let default_root = PathBuf::from("C:/data/logs");
        let own = ChatLogConfig::from_settings(
            &settings(true, Some("D:/mcc-logs"), 90),
            default_root.clone(),
        );
        assert!(own.enabled);
        assert_eq!(own.root, PathBuf::from("D:/mcc-logs"));
        assert_eq!(own.retention_days, 90);

        let fallback =
            ChatLogConfig::from_settings(&settings(false, None, 0), default_root.clone());
        assert!(!fallback.enabled);
        assert_eq!(fallback.root, default_root);
        assert_eq!(fallback.retention_days, 0);
    }

    #[tokio::test]
    async fn writer_creates_file_with_session_markers_and_clean_lines() {
        let root = temp_root("writer");
        let (tx_errors, _rx_errors) = errors();
        let log = ChatLog::new(root.clone(), tx_errors);
        log.apply_settings(&settings(true, None, 30));
        log.set_account_labels(&[account("acc-1", "AnonBOT")]);

        let id = "acc-1::host:25565";
        log.begin_session(id, "host:25565", "acc-1");
        log.write_line(id, LogStream::Stdout, "§8[MCC] §aHello");
        log.write_line(id, LogStream::Stderr, "boom");
        // Prompt logowania nie ma prawa trafić do pliku.
        log.write_line(
            id,
            LogStream::Stdout,
            "To sign in, open https://www.microsoft.com/link in your browser and enter the code: §e7FRBLM2Y",
        );
        log.end_session(id);

        let path = log_file_path(&root, "host:25565", "AnonBOT", &local_now().date());
        let content = wait_for_file(&path, "session end").await;
        assert!(content.contains("=== session start"), "{content}");
        assert!(content.contains("[MCC] Hello"), "{content}");
        assert!(content.contains("[stderr] boom"), "{content}");
        assert!(
            !content.contains('§'),
            "kody kolorów nie mogą trafiać do pliku"
        );
        assert!(
            !content.contains("7FRBLM2Y"),
            "kod logowania nie może trafić do pliku"
        );
        assert!(!content.contains("microsoft.com/link"), "{content}");

        let _ = std::fs::remove_dir_all(&root);
    }

    #[tokio::test]
    async fn disabled_chat_log_writes_nothing() {
        let root = temp_root("disabled");
        let (tx_errors, _rx_errors) = errors();
        let log = ChatLog::new(root.clone(), tx_errors);
        log.apply_settings(&settings(false, None, 30));
        log.set_account_labels(&[account("acc-1", "Bot")]);

        let id = "acc-1::host:25565";
        log.begin_session(id, "host:25565", "acc-1");
        log.write_line(id, LogStream::Stdout, "hello");
        log.end_session(id);
        tokio::time::sleep(Duration::from_millis(50)).await;

        assert!(
            !root.join("host_25565").exists(),
            "wyłączony zapis nic nie tworzy"
        );

        let _ = std::fs::remove_dir_all(&root);
    }

    /// Zmiana dnia w **działającej** sesji przenosi zapis do pliku z nową datą.
    ///
    /// Daty wstawiamy wprost do kolejki (test jest w tym samym module), więc
    /// sprawdzamy samą regułę, a nie zegar systemowy.
    #[tokio::test]
    async fn day_change_starts_a_new_file_mid_session() {
        let root = temp_root("day-change");
        let (tx_errors, _rx_errors) = errors();
        let log = ChatLog::new(root.clone(), tx_errors);
        log.apply_settings(&settings(true, None, 30));
        log.set_account_labels(&[account("acc-1", "Bot")]);

        let id = "acc-1::host:25565";
        log.begin_session(id, "host:25565", "acc-1");

        let session_tx = {
            let sessions = lock(&log.sessions);
            sessions.get(id).expect("sesja zapisu").tx.clone()
        };
        let first_day = LocalDateTime::new(2026, 1, 31, 23, 59, 0);
        let next_day = LocalDateTime::new(2026, 2, 1, 0, 1, 0);
        for (stamp, text) in [(first_day, "przed północą"), (next_day, "po północy")] {
            session_tx
                .send(Entry::Line {
                    stamp,
                    stream: LogStream::Stdout,
                    text: text.to_string(),
                })
                .await
                .expect("wysłanie linii");
        }
        log.end_session(id);

        let first_path = log_file_path(&root, "host:25565", "Bot", "2026-01-31");
        let second_path = log_file_path(&root, "host:25565", "Bot", "2026-02-01");
        let first = wait_for_file(&first_path, "przed północą").await;
        let second = wait_for_file(&second_path, "session end").await;
        assert!(first.contains("=== session start"), "{first}");
        assert!(!first.contains("po północy"), "{first}");
        assert!(second.contains("po północy"), "{second}");
        assert!(second.contains("=== session start"), "{second}");

        let _ = std::fs::remove_dir_all(&root);
    }

    /// Kolejka jest ograniczona: nadmiar linii gubimy, ale liczymy i pokazujemy
    /// w pliku jednym wierszem. Test wykorzystuje to, że zadanie zapisujące nie
    /// dostaje czasu procesora, dopóki test nie ma `await`.
    #[tokio::test]
    async fn queue_overflow_is_reported_in_the_file() {
        let root = temp_root("overflow");
        let (tx_errors, _rx_errors) = errors();
        let log = ChatLog::new(root.clone(), tx_errors);
        log.apply_settings(&settings(true, None, 30));
        log.set_account_labels(&[account("acc-1", "Bot")]);

        let id = "acc-1::host:25565";
        log.begin_session(id, "host:25565", "acc-1");
        for index in 0..QUEUE_CAPACITY + 100 {
            log.write_line(id, LogStream::Stdout, &format!("line {index}"));
        }
        log.end_session(id);

        let path = log_file_path(&root, "host:25565", "Bot", &local_now().date());
        let content = wait_for_file(&path, "lines omitted").await;
        assert!(content.contains("100 lines omitted"), "{content}");

        let _ = std::fs::remove_dir_all(&root);
    }

    /// Błąd zapisu (np. pełny dysk) kończy zapis dla tego połączenia i zgłasza
    /// dokładnie jeden toast z kodem `chatlog.writeFailed`.
    #[tokio::test]
    async fn write_error_is_reported_once_and_stops_writing() {
        let root = temp_root("write-error");
        // Pod folderem logów kładziemy plik, więc utworzenie katalogów się nie uda.
        let blocker = root.join("blocked");
        std::fs::write(&blocker, b"nie katalog").expect("plik blokujący");

        let (tx_errors, mut rx_errors) = errors();
        let log = ChatLog::new(blocker, tx_errors);
        log.apply_settings(&settings(true, None, 30));
        log.set_account_labels(&[account("acc-1", "Bot")]);

        let id = "acc-1::host:25565";
        log.begin_session(id, "host:25565", "acc-1");
        log.write_line(id, LogStream::Stdout, "hello");
        log.write_line(id, LogStream::Stdout, "drugi");
        log.end_session(id);

        let first = tokio::time::timeout(Duration::from_secs(5), rx_errors.recv())
            .await
            .expect("błąd zapisu powinien zostać zgłoszony")
            .expect("kanał błędów otwarty");
        assert_eq!(first.code, "chatlog.writeFailed");
        assert_eq!(first.params["server"], "host:25565");

        // Zapis dla tego połączenia jest wyłączony, więc kolejne błędy nie lecą.
        log.write_line(id, LogStream::Stdout, "trzeci");
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert!(rx_errors.try_recv().is_err(), "toast ma być tylko jeden");

        let _ = std::fs::remove_dir_all(&root);
    }
}

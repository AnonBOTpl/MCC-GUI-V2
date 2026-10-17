//! Testy menedżera procesów z fałszywym procesem potomnym (`fake_mcc`).
//! Nie wymagają prawdziwego MCC ani serwera Minecraft.
//!
//! Model: konto ma listę serwerów, a każde połączenie (konto + serwer) to osobny
//! proces uruchamiany ze **wspólnego** katalogu konta (wspólna sesja MCC).

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use mcc_gui_v2_lib::account::{Account, AccountStatus, AccountType, ServerEntry};
use mcc_gui_v2_lib::connection::connection_id;
use mcc_gui_v2_lib::events::{EventSink, LogStream};
use mcc_gui_v2_lib::process_manager::ProcessManager;

/// Serwer używany w testach (fake_mcc nie łączy się z niczym naprawdę).
const SERVER: &str = "127.0.0.1:9";

#[derive(Debug, Clone, PartialEq)]
enum Event {
    Log { stream: LogStream, line: String },
    Status(AccountStatus),
    Error(String),
    DeviceCode { url: String, code: String },
    Joined,
}

#[derive(Default)]
struct Collector {
    events: Mutex<Vec<Event>>,
}

impl Collector {
    fn snapshot(&self) -> Vec<Event> {
        self.events.lock().expect("mutex").clone()
    }
}

impl EventSink for Collector {
    fn emit_log(&self, _account_id: &str, stream: LogStream, line: String, _ts: i64) {
        self.events
            .lock()
            .expect("mutex")
            .push(Event::Log { stream, line });
    }

    fn emit_status(&self, _account_id: &str, status: AccountStatus) {
        self.events
            .lock()
            .expect("mutex")
            .push(Event::Status(status));
    }

    fn emit_error(&self, code: &str, _params: serde_json::Value) {
        self.events
            .lock()
            .expect("mutex")
            .push(Event::Error(code.to_string()));
    }

    fn emit_device_code(&self, _account_id: &str, url: String, code: String) {
        self.events
            .lock()
            .expect("mutex")
            .push(Event::DeviceCode { url, code });
    }

    fn emit_joined(&self, _account_id: &str) {
        self.events.lock().expect("mutex").push(Event::Joined);
    }
}

fn fake_exe() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_fake_mcc"))
}

fn account(id: &str, servers: &[&str]) -> Account {
    Account {
        id: id.to_string(),
        label: id.to_string(),
        account_type: AccountType::Offline,
        login: "TestBot".to_string(),
        servers: servers.iter().map(|s| ServerEntry::auto(s)).collect(),
        tab_order: 0,
    }
}

fn conn(id: &str, server: &str) -> String {
    connection_id(id, server)
}

/// Unikalny katalog testowy.
///
/// Sam znacznik czasu nie wystarczy: testy w jednym pliku dzielą PID i przy
/// równoległym starcie mogłyby trafić w ten sam katalog, a wtedy sprzątanie
/// jednego testu kasuje katalog drugiego. Licznik atomowy to wyklucza.
fn temp_dir() -> PathBuf {
    use std::sync::atomic::{AtomicU64, Ordering};
    static SEQ: AtomicU64 = AtomicU64::new(0);

    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let seq = SEQ.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("mcc-pm-test-{}-{nanos}-{seq}", std::process::id()))
}

async fn wait_for(
    collector: &Arc<Collector>,
    predicate: impl Fn(&[Event]) -> bool,
    timeout: Duration,
) -> bool {
    let deadline = Instant::now() + timeout;
    loop {
        if predicate(&collector.snapshot()) {
            return true;
        }
        if Instant::now() >= deadline {
            return false;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}

async fn started_manager() -> (Arc<Collector>, PathBuf, ProcessManager<Collector>) {
    let collector = Arc::new(Collector::default());
    let dir = temp_dir();
    let manager = ProcessManager::new(Some(fake_exe()), dir.clone(), collector.clone())
        .expect("menedżer powinien się utworzyć");
    (collector, dir, manager)
}

/// Rejestruje konto, opcjonalnie tworzy wyzwalacz promptu i startuje połączenie.
async fn start_account(
    manager: &ProcessManager<Collector>,
    dir: &Path,
    id: &str,
    server: &str,
    prompt: bool,
) {
    manager
        .add_account(account(id, &[]))
        .await
        .expect("dodanie konta");
    manager
        .add_server(id, server, None)
        .await
        .expect("dodanie serwera");
    if prompt {
        // `fake_mcc` szuka wyzwalacza w katalogu roboczym, a tym jest katalog konta.
        std::fs::write(dir.join(id).join("fake-prompt"), b"1").expect("wyzwalacz promptu");
    }
    manager
        .start_connection(id, server)
        .await
        .expect("start połączenia");
}

fn status_events(events: &[Event]) -> Vec<AccountStatus> {
    events
        .iter()
        .filter_map(|event| match event {
            Event::Status(status) => Some(*status),
            _ => None,
        })
        .collect()
}

fn has_line(events: &[Event], needle: &str) -> bool {
    events
        .iter()
        .any(|event| matches!(event, Event::Log { line, .. } if line == needle))
}

fn saw_running(events: &[Event]) -> bool {
    events
        .iter()
        .any(|event| matches!(event, Event::Status(AccountStatus::Running)))
}

#[tokio::test]
async fn spawn_streams_stdout_and_stderr_then_stops() {
    let (collector, dir, manager) = started_manager().await;
    start_account(&manager, &dir, "a", SERVER, false).await;

    assert!(
        wait_for(
            &collector,
            |events| has_line(events, "§8[fake] ready"),
            Duration::from_secs(5)
        )
        .await,
        "brak linii gotowości na stdout (kody § muszą zostać zachowane)"
    );
    assert!(
        wait_for(
            &collector,
            |events| events.iter().any(
                |event| matches!(event, Event::Log { stream, .. } if *stream == LogStream::Stderr)
            ),
            Duration::from_secs(5)
        )
        .await,
        "brak linii na stderr"
    );
    assert!(
        wait_for(&collector, saw_running, Duration::from_secs(5)).await,
        "brak statusu running"
    );

    manager
        .stop_connection(&conn("a", SERVER))
        .await
        .expect("zatrzymanie");
    assert!(
        wait_for(
            &collector,
            |events| events
                .iter()
                .any(|event| matches!(event, Event::Status(AccountStatus::Stopped))),
            Duration::from_secs(5)
        )
        .await,
        "brak statusu stopped po /quit"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

/// Połączenia jednego konta dzielą katalog (wspólna sesja), ale mają różne
/// pliki ustawień - to jest istota modelu „konto + lista serwerów".
#[tokio::test]
async fn connections_of_one_account_share_the_directory_but_not_settings() {
    let (collector, dir, manager) = started_manager().await;
    manager
        .add_account(account("a", &["one:1", "two:2"]))
        .await
        .expect("dodanie konta");

    manager
        .start_connection("a", "one:1")
        .await
        .expect("start one");
    manager
        .start_connection("a", "two:2")
        .await
        .expect("start two");

    assert!(
        wait_for(
            &collector,
            |events| events
                .iter()
                .filter(|event| matches!(event, Event::Status(AccountStatus::Running)))
                .count()
                >= 2,
            Duration::from_secs(5)
        )
        .await,
        "oba połączenia powinny działać"
    );

    let entries: Vec<String> = std::fs::read_dir(dir.join("a"))
        .expect("katalog konta")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    let settings: Vec<&String> = entries
        .iter()
        .filter(|name| name.starts_with("settings-") && name.ends_with(".ini"))
        .collect();
    assert_eq!(
        settings.len(),
        2,
        "każde połączenie ma własny plik ustawień, było: {entries:?}"
    );
    assert!(
        !entries
            .iter()
            .any(|name| name == "one:1" || name == "two:2"),
        "ustawienia nie mogą tworzyć podkatalogów per serwer: {entries:?}"
    );

    manager
        .stop_connection(&conn("a", "one:1"))
        .await
        .expect("stop");
    manager
        .stop_connection(&conn("a", "two:2"))
        .await
        .expect("stop");
    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn send_input_is_echoed() {
    let (collector, dir, manager) = started_manager().await;
    start_account(&manager, &dir, "a", SERVER, false).await;
    assert!(wait_for(&collector, saw_running, Duration::from_secs(5)).await);

    manager
        .send_input(&conn("a", SERVER), "hello")
        .await
        .expect("wysłanie");
    assert!(
        wait_for(
            &collector,
            |events| has_line(events, "echo: hello"),
            Duration::from_secs(5)
        )
        .await,
        "brak echa wpisu"
    );

    manager
        .stop_connection(&conn("a", SERVER))
        .await
        .expect("zatrzymanie");
    let _ = std::fs::remove_dir_all(&dir);
}

/// R-03: dwa równoległe starty tego samego połączenia nie mogą uruchomić dwóch
/// procesów ani wyglądać w UI jak awaria.
#[tokio::test]
async fn double_start_spawns_one_process() {
    let (collector, dir, manager) = started_manager().await;
    manager
        .add_account(account("a", &[SERVER]))
        .await
        .expect("dodanie konta");

    let (first, second) = tokio::join!(
        manager.start_connection("a", SERVER),
        manager.start_connection("a", SERVER),
    );

    let successes = usize::from(first.is_ok()) + usize::from(second.is_ok());
    assert_eq!(successes, 1, "dokładnie jeden start może się udać");
    let rejected = first
        .err()
        .or_else(|| second.err())
        .expect("drugi start musi zostać odrzucony");
    assert_eq!(rejected.code, "connection.alreadyRunning");

    assert!(
        wait_for(
            &collector,
            |events| events
                .iter()
                .filter(
                    |event| matches!(event, Event::Log { line, .. } if line == "§8[fake] ready")
                )
                .count()
                >= 1,
            Duration::from_secs(5)
        )
        .await,
        "żaden proces nie wystartował"
    );
    // Chwila, żeby ewentualny drugi proces też zdążył wypisać gotowość.
    tokio::time::sleep(Duration::from_millis(300)).await;

    assert_eq!(
        manager.active_count().await,
        1,
        "mapa musi trzymać dokładnie jeden proces"
    );
    let events = collector.snapshot();
    let ready = events
        .iter()
        .filter(|event| matches!(event, Event::Log { line, .. } if line == "§8[fake] ready"))
        .count();
    assert_eq!(ready, 1, "tylko jeden proces MCC może wystartować");
    assert!(
        !status_events(&events).contains(&AccountStatus::Crashed),
        "odrzucony start nie może kończyć się fałszywą awarią"
    );

    manager
        .stop_connection(&conn("a", SERVER))
        .await
        .expect("zatrzymanie");
    let _ = std::fs::remove_dir_all(&dir);
}

/// R-04: zapis do `stdin`, którego proces nie czyta, nie może zablokować
/// zatrzymania połączenia (ani reszty aplikacji przez globalną blokadę mapy).
#[tokio::test]
async fn stalled_stdin_does_not_block_stop() {
    let (collector, dir, manager) = started_manager().await;
    // Wyzwalacz musi istnieć, zanim atrapa wystartuje.
    std::fs::create_dir_all(dir.join("a")).expect("katalog konta");
    std::fs::write(dir.join("a").join("fake-stall-stdin"), b"1").expect("wyzwalacz");
    start_account(&manager, &dir, "a", SERVER, false).await;
    assert!(wait_for(&collector, saw_running, Duration::from_secs(5)).await);

    let manager = Arc::new(manager);
    let sender = {
        let manager = Arc::clone(&manager);
        tokio::spawn(async move {
            // 300 wpisów po 512 bajtów nie mieści się w buforze potoku.
            for _ in 0..300 {
                let _ = manager
                    .send_input(&conn("a", SERVER), &"x".repeat(512))
                    .await;
            }
        })
    };
    tokio::time::sleep(Duration::from_millis(300)).await;

    let outcome = tokio::time::timeout(
        Duration::from_secs(8),
        manager.stop_connection(&conn("a", SERVER)),
    )
    .await;
    assert!(
        outcome.is_ok(),
        "stop_connection wisi, gdy zapis do stdin stoi"
    );
    outcome
        .expect("timeout zatrzymania")
        .expect("zatrzymanie połączenia");

    assert!(
        wait_for(
            &collector,
            |events| events
                .iter()
                .any(|event| matches!(event, Event::Status(AccountStatus::Stopped))),
            Duration::from_secs(5)
        )
        .await,
        "po wymuszonym killu musi przyjść status stopped"
    );
    assert_eq!(manager.active_count().await, 0);

    sender.abort();
    let _ = sender.await;
    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn invalid_input_is_rejected() {
    let (_collector, dir, manager) = started_manager().await;
    start_account(&manager, &dir, "a", SERVER, false).await;

    let id = conn("a", SERVER);
    assert_eq!(
        manager.send_input(&id, "a\nb").await.unwrap_err().code,
        "input.controlChars"
    );
    assert_eq!(
        manager.send_input(&id, "  ").await.unwrap_err().code,
        "input.empty"
    );

    manager.stop_connection(&id).await.expect("zatrzymanie");
    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn send_input_all_respects_delay() {
    let (collector, dir, manager) = started_manager().await;
    start_account(&manager, &dir, "a", SERVER, false).await;
    start_account(&manager, &dir, "b", SERVER, false).await;
    assert!(
        wait_for(
            &collector,
            |events| events
                .iter()
                .filter(|event| matches!(event, Event::Status(AccountStatus::Running)))
                .count()
                >= 2,
            Duration::from_secs(5)
        )
        .await
    );

    let start = Instant::now();
    let sent = manager
        .send_input_all("broadcast", 200)
        .await
        .expect("wysłanie do wszystkich");
    let elapsed = start.elapsed();

    assert_eq!(sent.len(), 2, "wpis powinien trafić do obu połączeń");
    assert!(
        elapsed >= Duration::from_millis(200),
        "oczekiwano co najmniej jednego opóźnienia, minęło {elapsed:?}"
    );

    manager
        .stop_connection(&conn("a", SERVER))
        .await
        .expect("stop a");
    manager
        .stop_connection(&conn("b", SERVER))
        .await
        .expect("stop b");
    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn crash_is_reported() {
    let (collector, dir, manager) = started_manager().await;
    start_account(&manager, &dir, "a", SERVER, false).await;
    assert!(wait_for(&collector, saw_running, Duration::from_secs(5)).await);

    manager
        .send_input(&conn("a", SERVER), "crash")
        .await
        .expect("wysłanie");

    assert!(
        wait_for(
            &collector,
            |events| events
                .iter()
                .any(|event| matches!(event, Event::Status(AccountStatus::Crashed))),
            Duration::from_secs(5)
        )
        .await,
        "brak statusu crashed"
    );
    assert!(
        wait_for(
            &collector,
            |events| events.iter().any(
                |event| matches!(event, Event::Error(code) if code == "process.exitedUnexpectedly")
            ),
            Duration::from_secs(5)
        )
        .await,
        "brak zdarzenia błędu o nieoczekiwanym zakończeniu"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

/// `/quit` wpisane przez użytkownika to normalne zamknięcie, a nie awaria.
/// Proces kończy się sam (bez naszego `/quit`), więc bez rozpoznania komendy
/// strażnik zgłosiłby fałszywy `Crashed` i błąd w UI.
#[tokio::test]
async fn user_quit_is_stopped_not_crashed() {
    let (collector, dir, manager) = started_manager().await;
    start_account(&manager, &dir, "a", SERVER, false).await;
    assert!(wait_for(&collector, saw_running, Duration::from_secs(5)).await);

    manager
        .send_input(&conn("a", SERVER), "/quit")
        .await
        .expect("wysłanie /quit");

    assert!(
        wait_for(
            &collector,
            |events| events
                .iter()
                .any(|event| matches!(event, Event::Status(AccountStatus::Stopped))),
            Duration::from_secs(5)
        )
        .await,
        "po /quit oczekiwano statusu stopped"
    );

    let events = collector.snapshot();
    assert!(
        !status_events(&events).contains(&AccountStatus::Crashed),
        "/quit nie może być raportowane jako awaria"
    );
    assert!(
        !events.iter().any(
            |event| matches!(event, Event::Error(code) if code == "process.exitedUnexpectedly")
        ),
        "po /quit nie może pojawić się błąd o nieoczekiwanym zakończeniu"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

/// Regresja: `/quit` wpisane przez użytkownika może nie zamknąć procesu (MCC zjada tę
/// linię jako odpowiedź na swoje pytanie - `docs/MCC_NOTES.md` §3.2), ale nasz stan
/// wewnętrzny i tak staje się `Stopping`. Kolejne zatrzymanie (przycisk „Disconnect”)
/// musi wtedy **wymusić kill**; wcześniej było cichym „nic do zrobienia”, więc proces
/// zostawał uruchomiony, a UI pokazywało `running` bez sposobu, żeby go zatrzymać.
#[tokio::test]
async fn repeated_stop_forces_kill_when_quit_is_ignored() {
    let (collector, dir, manager) = started_manager().await;
    manager
        .add_account(account("a", &[]))
        .await
        .expect("dodanie konta");
    manager
        .add_server("a", SERVER, None)
        .await
        .expect("dodanie serwera");
    // Wyzwalacz musi istnieć **przed** startem procesu (atrapa czyta go raz, na starcie).
    std::fs::write(dir.join("a").join("fake-ignore-quit"), b"1").expect("wyzwalacz");
    manager
        .start_connection("a", SERVER)
        .await
        .expect("start połączenia");
    assert!(wait_for(&collector, saw_running, Duration::from_secs(5)).await);

    // Użytkownik wpisuje `/quit`: atrapa je ignoruje, proces żyje (jak MCC na promptcie).
    manager
        .send_input(&conn("a", SERVER), "/quit")
        .await
        .expect("wysłanie /quit");
    assert!(
        wait_for(
            &collector,
            |events| has_line(events, "§8[fake] ignoruję /quit"),
            Duration::from_secs(5)
        )
        .await,
        "atrapa powinna zignorować /quit"
    );
    assert_eq!(manager.active_count().await, 1, "proces nadal działa");

    // „Disconnect”: drugie zatrzymanie musi wymusić kill.
    manager
        .stop_connection(&conn("a", SERVER))
        .await
        .expect("zatrzymanie połączenia");
    assert_eq!(
        manager.active_count().await,
        0,
        "powtórne zatrzymanie powinno zabić proces"
    );
    assert!(
        !status_events(&collector.snapshot()).contains(&AccountStatus::Crashed),
        "wymuszone zatrzymanie na życzenie użytkownika nie jest awarią"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn restart_connection_restarts_the_process() {
    let (collector, dir, manager) = started_manager().await;
    start_account(&manager, &dir, "a", SERVER, false).await;
    assert!(wait_for(&collector, saw_running, Duration::from_secs(5)).await);

    manager
        .stop_connection(&conn("a", SERVER))
        .await
        .expect("zatrzymanie");
    assert!(
        wait_for(
            &collector,
            |events| events
                .iter()
                .any(|event| matches!(event, Event::Status(AccountStatus::Stopped))),
            Duration::from_secs(5)
        )
        .await
    );

    manager
        .restart_connection("a", SERVER)
        .await
        .expect("ponowne uruchomienie");
    assert!(
        wait_for(
            &collector,
            |events| events
                .iter()
                .filter(|event| matches!(event, Event::Status(AccountStatus::Running)))
                .count()
                >= 2,
            Duration::from_secs(5)
        )
        .await,
        "połączenie powinno wrócić do statusu running"
    );

    manager
        .stop_connection(&conn("a", SERVER))
        .await
        .expect("zatrzymanie");
    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn device_code_prompt_switches_to_awaiting_login() {
    let (collector, dir, manager) = started_manager().await;
    start_account(&manager, &dir, "a", SERVER, true).await;

    assert!(
        wait_for(
            &collector,
            |events| events
                .iter()
                .any(|event| matches!(event, Event::Status(AccountStatus::AwaitingLogin))),
            Duration::from_secs(5)
        )
        .await,
        "brak statusu awaiting_login po prompcie"
    );
    assert!(
        wait_for(
            &collector,
            |events| events
                .iter()
                .any(|event| matches!(event, Event::DeviceCode { url, code }
                if url == "https://www.microsoft.com/link" && code == "7FRBLM2Y")),
            Duration::from_secs(5)
        )
        .await,
        "brak zdarzenia z kodem logowania"
    );

    manager
        .stop_connection(&conn("a", SERVER))
        .await
        .expect("zatrzymanie");
    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn join_returns_to_running_after_login() {
    let (collector, dir, manager) = started_manager().await;
    start_account(&manager, &dir, "a", SERVER, true).await;

    assert!(
        wait_for(
            &collector,
            |events| events
                .iter()
                .any(|event| matches!(event, Event::Status(AccountStatus::AwaitingLogin))),
            Duration::from_secs(5)
        )
        .await,
        "połączenie powinno czekać na logowanie"
    );

    manager
        .send_input(&conn("a", SERVER), "join")
        .await
        .expect("wysłanie");

    assert!(
        wait_for(
            &collector,
            |events| events.iter().any(|event| matches!(event, Event::Joined)),
            Duration::from_secs(5)
        )
        .await,
        "brak zdarzenia joined"
    );

    // Status musi wrócić do `running` po zdaniu o połączeniu.
    let statuses = status_events(&collector.snapshot());
    let awaiting = statuses
        .iter()
        .position(|status| *status == AccountStatus::AwaitingLogin)
        .expect("awaiting_login w historii");
    assert!(
        statuses[awaiting + 1..].contains(&AccountStatus::Running),
        "po awaiting_login powinien wrócić running, było: {statuses:?}"
    );

    manager
        .stop_connection(&conn("a", SERVER))
        .await
        .expect("zatrzymanie");
    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn input_is_accepted_while_awaiting_login() {
    let (collector, dir, manager) = started_manager().await;
    start_account(&manager, &dir, "a", SERVER, true).await;

    assert!(
        wait_for(
            &collector,
            |events| events
                .iter()
                .any(|event| matches!(event, Event::Status(AccountStatus::AwaitingLogin))),
            Duration::from_secs(5)
        )
        .await,
        "połączenie powinno czekać na logowanie"
    );

    // Wpis w trakcie logowania nie może być odrzucony (MCC czyta go z `stdin`).
    manager
        .send_input(&conn("a", SERVER), "hello")
        .await
        .expect("wpis w trakcie logowania powinien zostać przyjęty");
    assert!(
        wait_for(
            &collector,
            |events| has_line(events, "echo: hello"),
            Duration::from_secs(5)
        )
        .await,
        "brak echa wpisu"
    );

    manager
        .stop_connection(&conn("a", SERVER))
        .await
        .expect("zatrzymanie");
    let _ = std::fs::remove_dir_all(&dir);
}

/// Zły adres serwera i brak zapisanego serwera są odrzucane.
/// Aktualizację wersji (`upsert`) pokrywa test jednostkowy w `process_manager`,
/// bo wymaga dostępu do prywatnego stanu menedżera.
#[tokio::test]
async fn add_server_validates_and_rejects_unknown() {
    let (_collector, dir, manager) = started_manager().await;
    manager
        .add_account(account("a", &["Host:25565"]))
        .await
        .expect("dodanie konta");

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
    assert_eq!(
        manager
            .remove_server("a", "missing:1")
            .await
            .unwrap_err()
            .code,
        "connection.notFound"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn unknown_account_and_connection_errors() {
    let (_collector, dir, manager) = started_manager().await;
    assert_eq!(
        manager
            .send_input("nope::host", "hi")
            .await
            .unwrap_err()
            .code,
        "connection.notRunning"
    );
    assert_eq!(
        manager.remove_account("nope").await.unwrap_err().code,
        "account.notFound"
    );
    assert_eq!(
        manager
            .start_connection("nope", "host:1")
            .await
            .unwrap_err()
            .code,
        "account.notFound"
    );
    // Zatrzymanie nieuruchomionego połączenia jest bezpieczne.
    manager
        .stop_connection("nope::host")
        .await
        .expect("stop nieuruchomionego połączenia");
    let _ = std::fs::remove_dir_all(&dir);
}

/// Usunięcie konta zatrzymuje wszystkie jego połączenia.
#[tokio::test]
async fn removing_account_stops_its_connections() {
    let (collector, dir, manager) = started_manager().await;
    start_account(&manager, &dir, "a", SERVER, false).await;
    assert!(wait_for(&collector, saw_running, Duration::from_secs(5)).await);

    manager.remove_account("a").await.expect("usunięcie konta");

    assert!(
        wait_for(
            &collector,
            |events| events
                .iter()
                .any(|event| matches!(event, Event::Status(AccountStatus::Stopped))),
            Duration::from_secs(5)
        )
        .await,
        "po usunięciu konta proces powinien zostać zatrzymany"
    );
    assert!(
        manager.stop_connection(&conn("a", SERVER)).await.is_ok(),
        "zatrzymanie po usunięciu konta jest bezpieczne"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn missing_mcc_path_errors() {
    let collector = Arc::new(Collector::default());
    let dir = temp_dir();
    let manager = ProcessManager::new(None, dir.clone(), collector).expect("utworzenie menedżera");
    manager
        .add_account(account("a", &[SERVER]))
        .await
        .expect("dodanie konta");
    assert_eq!(
        manager
            .start_connection("a", SERVER)
            .await
            .unwrap_err()
            .code,
        "mcc.executableNotSet"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// Licznik aktywnych połączeń jest podstawą pytania o zamknięcie aplikacji
/// (`close::should_confirm`): pytamy tylko wtedy, gdy jest co przerywać.
#[tokio::test]
async fn active_count_follows_running_connections() {
    let (collector, dir, manager) = started_manager().await;

    assert_eq!(
        manager.active_count().await,
        0,
        "świeży menedżer nie ma aktywnych połączeń"
    );

    start_account(&manager, &dir, "a", SERVER, false).await;
    assert!(wait_for(&collector, saw_running, Duration::from_secs(5)).await);
    assert_eq!(manager.active_count().await, 1);

    manager
        .stop_connection(&conn("a", SERVER))
        .await
        .expect("zatrzymanie połączenia");

    assert_eq!(
        manager.active_count().await,
        0,
        "po zatrzymaniu licznik musi wrócić do zera"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

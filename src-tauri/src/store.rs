//! Trwałość ustawień i kont w katalogu danych aplikacji.
//!
//! Pliki: `settings.json`, `accounts.json`, `presets.json`, `session.json`. Zapis jest atomowy
//! (plik tymczasowy + podmiana), żeby przerwanie zapisu nie zostawiło
//! uszkodzonego pliku.
//!
//! W plikach **nie ma żadnych sekretów** - konto nie ma pola hasła, a sesje MCC
//! żyją wyłącznie w katalogach kont i aplikacja ich nie dotyka.
//!
//! Dwa zabezpieczenia danych użytkownika:
//!
//! - **Odczyt wpis po wpisie** (`accounts.json`, R-16): jeden niepoprawny wpis
//!   nie kasuje całej listy - reszta kont wczytuje się normalnie, a o pominiętych
//!   wpisach informujemy w UI.
//! - **Blokada nadpisania** (R-10): jeśli pliku nie dało się odczytać, zapisujemy
//!   obok kopię (`<nazwa>.bad-<RRRR-MM-DD-HHMMSS>`) i **nie pozwalamy** zastąpić
//!   go wartościami domyślnymi, dopóki nie uda się go poprawnie wczytać.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::json;
use tokio::io::AsyncWriteExt;

use crate::account::Account;
use crate::error::AppError;
use crate::preset::Preset;
use crate::session::OpenTab;
use crate::settings::AppSettings;

/// Nazwa pliku z ustawieniami aplikacji.
pub const SETTINGS_FILE: &str = "settings.json";

/// Nazwa pliku z zapisanymi kontami.
pub const ACCOUNTS_FILE: &str = "accounts.json";

/// Nazwa pliku z presetami szybkich akcji.
pub const PRESETS_FILE: &str = "presets.json";

/// Nazwa pliku z sesją: zakładki (połączenia) otwarte przy zamknięciu aplikacji.
pub const SESSION_FILE: &str = "session.json";

/// Ostrzeżenie o pliku danych, który nie wczytał się w całości.
///
/// Ta sama forma co błąd (kod + parametry), bo frontend pokazuje je jako toast
/// tą samą drogą - Rust nie wysyła tekstów dla człowieka.
#[derive(Debug, Clone, PartialEq)]
pub struct StoreWarning {
    pub code: String,
    pub params: serde_json::Value,
}

/// Problem z plikiem wykryty przy odczycie.
struct FileIssue {
    /// Czy zapis ma być wstrzymany, żeby nie nadpisać pliku wartościami domyślnymi.
    blocks_writes: bool,
    /// Czy użytkownik został już o tym pliku powiadomiony (i czy powstała kopia).
    reported: bool,
}

/// Czyta i zapisuje pliki JSON w katalogu danych aplikacji.
pub struct Store {
    root: PathBuf,
    /// Problemy z plikami, kluczowane nazwą pliku. `std::sync::Mutex`, bo sekcje
    /// krytyczne są krótkie i nie zawierają `await`.
    issues: Mutex<HashMap<String, FileIssue>>,
    /// Ostrzeżenia do pokazania w UI - opróżniane przy pierwszej komendzie.
    warnings: Mutex<Vec<StoreWarning>>,
    /// Numer porządkowy każdego zapisu - trafia do nazwy pliku tymczasowego, żeby
    /// dwa zapisy nigdy nie pisały do tej samej nazwy (R-35). Licznik, a nie
    /// losowość, bo wtedy zachowanie jest powtarzalne w testach.
    temp_seq: AtomicU64,
    /// Blokada zapisu **per plik** (R-35): dwa zapisy tego samego pliku idą
    /// jeden po drugim, a różnych plików dalej równolegle. Zewnętrzny
    /// `std::sync::Mutex` trzymamy tylko na czas wyszukania wpisu (bez `await`),
    /// właściwa blokada jest asynchroniczna.
    locks: Mutex<HashMap<String, std::sync::Arc<tokio::sync::Mutex<()>>>>,
}

impl Store {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            issues: Mutex::new(HashMap::new()),
            warnings: Mutex::new(Vec::new()),
            temp_seq: AtomicU64::new(0),
            locks: Mutex::new(HashMap::new()),
        }
    }

    /// Blokada zapisu danego pliku, tworzona przy pierwszym użyciu.
    fn file_lock(&self, file: &str) -> std::sync::Arc<tokio::sync::Mutex<()>> {
        let mut locks = self
            .locks
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        locks
            .entry(file.to_string())
            .or_insert_with(|| std::sync::Arc::new(tokio::sync::Mutex::new(())))
            .clone()
    }

    /// Katalog danych aplikacji (tu też żyją katalogi kont).
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Bierze ostrzeżenia do pokazania w UI (lista zostaje pusta).
    ///
    /// Pliki czytamy przy starcie, czyli zanim okno zdąży podłączyć nasłuchy,
    /// więc ostrzeżenia czekają tutaj na pierwszą komendę z frontendu.
    pub fn take_warnings(&self) -> Vec<StoreWarning> {
        let mut warnings = self
            .warnings
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        std::mem::take(&mut *warnings)
    }

    /// Wczytuje ustawienia. Brak pliku daje wartości domyślne, a plik uszkodzony
    /// (albo nieczytelny) zostawia kopię obok i wstrzymuje zapis do niego.
    pub async fn load_settings(&self) -> AppSettings {
        self.read_or_default(SETTINGS_FILE, AppSettings::default)
            .await
    }

    /// Zapisuje ustawienia atomowo.
    pub async fn save_settings(&self, settings: &AppSettings) -> Result<(), AppError> {
        self.write_json(SETTINGS_FILE, settings).await
    }

    /// Wczytuje zapisane konta (bez uruchamiania procesów).
    ///
    /// Czytanie jest **wpis po wpisie** (R-16): konta, których nie da się
    /// odczytać albo które nie przechodzą walidacji, są pomijane i policzone,
    /// a reszta listy trafia do panelu. Dzięki temu jeden zepsuty wpis nie
    /// zamienia wszystkich kont w pustą listę (a pierwszy zapis nie utrwala tej
    /// pustki na dysku).
    pub async fn load_accounts(&self) -> Vec<Account> {
        let file = ACCOUNTS_FILE;
        let raw = match self.read_file(file).await {
            Ok(Some(raw)) => raw,
            Ok(None) => return Vec::new(),
            Err(error) => {
                tracing::warn!(%error, file = %self.root.join(file).display(), "nie udało się odczytać listy kont");
                self.report_unreadable(file, None).await;
                return Vec::new();
            }
        };

        let entries: Vec<serde_json::Value> = match serde_json::from_str(&raw) {
            Ok(entries) => entries,
            Err(error) => {
                tracing::warn!(%error, file = %self.root.join(file).display(), "uszkodzony plik kont; zostawiam kopię i nie nadpisuję pliku");
                self.report_unreadable(file, Some(&raw)).await;
                return Vec::new();
            }
        };

        let mut kept: Vec<Account> = Vec::with_capacity(entries.len());
        let mut ids: HashSet<String> = HashSet::with_capacity(entries.len());
        let mut dropped = 0usize;

        for entry in entries {
            let account = match serde_json::from_value::<Account>(entry) {
                Ok(account) => account,
                Err(error) => {
                    tracing::warn!(%error, "pominięty wpis konta: nie da się go odczytać");
                    dropped += 1;
                    continue;
                }
            };
            let account = match crate::account::validate_account(account) {
                Ok(account) => account,
                Err(error) => {
                    tracing::warn!(%error, "pominięty wpis konta: nie przechodzi walidacji");
                    dropped += 1;
                    continue;
                }
            };
            // Powtórzony identyfikator zostawia pierwszy wpis - inaczej lista
            // nie przeszłaby zapisu (`account.duplicateId`).
            if !ids.insert(account.id.clone()) {
                tracing::warn!(id = %account.id, "pominięty wpis konta: powtórzony identyfikator");
                dropped += 1;
                continue;
            }
            kept.push(account);
        }

        if dropped > 0 {
            tracing::warn!(dropped, file, "pominięto wpisy w pliku kont");
            self.report_skipped(file, dropped, &raw).await;
        } else {
            self.clear_issue(file);
        }

        kept
    }

    /// Zapisuje konta atomowo.
    pub async fn save_accounts(&self, accounts: &[Account]) -> Result<(), AppError> {
        self.write_json(ACCOUNTS_FILE, &accounts.to_vec()).await
    }

    /// Wczytuje presety szybkich akcji przy starcie. Czytanie jest **łagodne**:
    /// brak pliku oznacza po prostu „brak presetów”, a plik ręcznie zepsuty
    /// logujemy (z kopią obok) i wracamy do pustej listy, żeby aplikacja wystartowała.
    pub async fn load_presets(&self) -> Vec<Preset> {
        self.read_or_default(PRESETS_FILE, Vec::new).await
    }

    /// Ponowne wczytanie presetów dla przycisku `Reload`.
    ///
    /// W przeciwieństwie do startu jest **ścisłe**: jeśli plik istnieje, ale jest
    /// niepoprawnym JSON-em (np. po ręcznej edycji), zwracamy błąd, bo wtedy
    /// użytkownik musi się o tym dowiedzieć - inaczej zobaczyłby pusty pasek
    /// bez wyjaśnienia i mógłby przypadkiem nadpisać plik. Udany odczyt zdejmuje
    /// blokadę zapisu (to jest droga naprawy bez restartu aplikacji).
    pub async fn reload_presets(&self) -> Result<Vec<Preset>, AppError> {
        let path = self.root.join(PRESETS_FILE);
        let raw = match tokio::fs::read_to_string(&path).await {
            Ok(raw) => raw,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                self.clear_issue(PRESETS_FILE);
                return Ok(Vec::new());
            }
            Err(error) => {
                tracing::warn!(%error, file = %path.display(), "nie udało się odczytać presetów");
                self.report_unreadable(PRESETS_FILE, None).await;
                return Err(AppError::new("presets.readFailed"));
            }
        };

        match serde_json::from_str(&raw) {
            Ok(presets) => {
                self.clear_issue(PRESETS_FILE);
                Ok(presets)
            }
            Err(error) => {
                tracing::warn!(%error, file = %path.display(), "uszkodzony plik presetów");
                self.report_unreadable(PRESETS_FILE, Some(&raw)).await;
                Err(AppError::new("presets.readFailed"))
            }
        }
    }

    /// Zapisuje presety szybkich akcji atomowo.
    pub async fn save_presets(&self, presets: &[Preset]) -> Result<(), AppError> {
        self.write_json(PRESETS_FILE, &presets.to_vec()).await
    }

    /// Wczytuje zakładki z poprzedniej sesji. Czytanie jest **łagodne**: brak
    /// pliku albo uszkodzony JSON oznacza po prostu brak zakładek do
    /// przywrócenia - stary plik sesji nie może blokować startu aplikacji.
    pub async fn load_session(&self) -> Vec<OpenTab> {
        self.read_or_default(SESSION_FILE, Vec::new).await
    }

    /// Zapisuje zakładki sesji atomowo.
    pub async fn save_session(&self, tabs: &[OpenTab]) -> Result<(), AppError> {
        self.write_json(SESSION_FILE, &tabs.to_vec()).await
    }

    /// Odczyt z odzyskiwaniem: brak pliku to nie błąd, uszkodzony plik logujemy,
    /// zostawiamy obok kopię i wstrzymujemy zapis do niego.
    async fn read_or_default<T: DeserializeOwned>(&self, file: &str, default: fn() -> T) -> T {
        let raw = match self.read_file(file).await {
            Ok(Some(raw)) => raw,
            Ok(None) => return default(),
            Err(error) => {
                tracing::warn!(%error, file = %self.root.join(file).display(), "nie udało się odczytać pliku");
                self.report_unreadable(file, None).await;
                return default();
            }
        };

        match serde_json::from_str(&raw) {
            Ok(value) => {
                self.clear_issue(file);
                value
            }
            Err(error) => {
                tracing::warn!(%error, file = %self.root.join(file).display(), "uszkodzony plik JSON; zostawiam kopię i nie nadpisuję pliku");
                self.report_unreadable(file, Some(&raw)).await;
                default()
            }
        }
    }

    /// Czyta plik jako tekst: `Ok(None)` oznacza brak pliku, `Err` inny błąd.
    async fn read_file(&self, file: &str) -> Result<Option<String>, std::io::Error> {
        match tokio::fs::read_to_string(self.root.join(file)).await {
            Ok(raw) => Ok(Some(raw)),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error),
        }
    }

    /// Atomowy zapis JSON: najpierw plik tymczasowy, potem podmiana.
    ///
    /// Zapis do pliku, którego nie udało się wczytać, jest **wstrzymany** (R-10):
    /// inaczej pierwszy zapis zastąpiłby dane użytkownika wartościami domyślnymi.
    async fn write_json<T: Serialize>(&self, file: &str, value: &T) -> Result<(), AppError> {
        // Zapis tego samego pliku czeka na poprzedni (R-35). Sprawdzenie blokady
        // (R-10) musi być **w środku**, żeby równoległy zapis, który ją zdejmie
        // udanym zapisem, nie został wyprzedzony przez własny odczyt stanu.
        let lock = self.file_lock(file);
        let _guard = lock.lock().await;

        if self.write_blocked(file) {
            tracing::warn!(file, "zapis wstrzymany: plik nie wczytał się poprawnie");
            return Err(AppError::with_params(
                "store.fileQuarantined",
                json!({ "file": file }),
            ));
        }

        let path = self.root.join(file);
        let json = serde_json::to_string_pretty(value).map_err(|error| {
            tracing::error!(%error, file = %path.display(), "nie udało się zserializować danych");
            AppError::new("store.writeFailed")
        })?;

        // Stała nazwa (`settings.json.tmp`) należała do wszystkich zapisów naraz,
        // więc drugi zapis mógł nadpisać plik, który pierwszy właśnie podmieniał.
        let seq = self.temp_seq.fetch_add(1, Ordering::Relaxed);
        let temp = self.root.join(format!("{file}.{seq}.tmp"));
        if let Err(error) = write_and_flush(&temp, json.as_bytes()).await {
            tracing::error!(%error, file = %temp.display(), "nie udało się zapisać pliku tymczasowego");
            // Nieudany zapis nie może zostawić śmieci obok plików danych.
            let _ = tokio::fs::remove_file(&temp).await;
            return Err(AppError::new("store.writeFailed"));
        }
        if let Err(error) = tokio::fs::rename(&temp, &path).await {
            tracing::error!(%error, file = %path.display(), "nie udało się podmienić pliku");
            let _ = tokio::fs::remove_file(&temp).await;
            return Err(AppError::new("store.writeFailed"));
        }

        // Na dysku jest teraz poprawny plik, więc blokada nie jest już potrzebna.
        self.clear_issue(file);
        Ok(())
    }

    /// Reakcja na nieczytelny plik: kopia obok, blokada zapisu i ostrzeżenie w UI.
    async fn report_unreadable(&self, file: &str, content: Option<&str>) {
        if !self.note_issue(file, true) {
            return;
        }
        self.backup_file(file, content).await;
        self.push_warning("store.fileUnreadable", json!({ "file": file }));
    }

    /// Reakcja na wpisy pominięte przy odczycie listy (R-16).
    ///
    /// Zapis nie jest blokowany - dobre wpisy są na dysku i można je utrwalić -
    /// ale plik z pominiętymi wpisami zapisujemy obok, żeby nic nie zginęło.
    async fn report_skipped(&self, file: &str, count: usize, content: &str) {
        if !self.note_issue(file, false) {
            return;
        }
        self.backup_file(file, Some(content)).await;
        self.push_warning(
            "store.entriesSkipped",
            json!({ "file": file, "count": count }),
        );
    }

    /// Zapisuje kopię pliku obok: `<nazwa>.bad-<RRRR-MM-DD-HHMMSS>`.
    ///
    /// Treść bierzemy z pamięci, jeśli ją mamy (przy nieczytelnym pliku czytamy
    /// go ponownie), więc kopia jest dokładnie tym, co leżało na dysku.
    async fn backup_file(&self, file: &str, content: Option<&str>) {
        let path = self.root.join(file);
        let now = crate::chatlog::local_now();
        let name = format!("{file}.bad-{}-{}", now.date(), now.time().replace(':', ""));
        let target = self.root.join(&name);

        let result = match content {
            Some(raw) => tokio::fs::write(&target, raw.as_bytes()).await,
            None => tokio::fs::copy(&path, &target).await.map(|_| ()),
        };

        match result {
            Ok(()) => tracing::warn!(
                file,
                backup = %target.display(),
                "kopia danych, których nie dało się wczytać"
            ),
            Err(error) => tracing::error!(
                %error,
                file,
                backup = %target.display(),
                "nie udało się zapisać kopii danych, których nie dało się wczytać"
            ),
        }
    }

    /// Zapisuje problem z plikiem. Zwraca `true`, jeśli to pierwsze zgłoszenie
    /// w tym uruchomieniu (czyli trzeba zrobić kopię i pokazać ostrzeżenie).
    fn note_issue(&self, file: &str, blocks_writes: bool) -> bool {
        let mut issues = self
            .issues
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let issue = issues.entry(file.to_string()).or_insert(FileIssue {
            blocks_writes: false,
            reported: false,
        });
        issue.blocks_writes |= blocks_writes;
        if issue.reported {
            return false;
        }
        issue.reported = true;
        true
    }

    /// Udany odczyt (albo zapis) zdejmuje blokadę zapisu.
    fn clear_issue(&self, file: &str) {
        let mut issues = self
            .issues
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        issues.remove(file);
    }

    /// Czy zapis do pliku jest wstrzymany po nieudanym odczycie.
    fn write_blocked(&self, file: &str) -> bool {
        let issues = self
            .issues
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        issues
            .get(file)
            .map(|issue| issue.blocks_writes)
            .unwrap_or(false)
    }

    /// Kolejkuje ostrzeżenie dla UI.
    fn push_warning(&self, code: &str, params: serde_json::Value) {
        let mut warnings = self
            .warnings
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        warnings.push(StoreWarning {
            code: code.to_string(),
            params,
        });
    }
}

/// Zapisuje treść do pliku tymczasowego i **czeka, aż dane będą na dysku** (R-34).
///
/// Sama atomowa podmiana (`rename`) nie wystarczy: bez `sync_all` po utracie
/// zasilania na dysku mógłby znaleźć się plik pusty albo ucięty, mimo że podmiana
/// wyglądała na udaną. `write_all` zamiast `fs::write` daje też pełną kontrolę
/// nad uchwytem, który musi zostać domknięty przed `rename` (Windows nie pozwala
/// podmienić pliku, który ktoś jeszcze trzyma otwarty).
async fn write_and_flush(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let mut handle = tokio::fs::File::create(path).await?;
    handle.write_all(bytes).await?;
    handle.sync_all().await?;
    // Uchwyt domyka się tutaj, przed `rename` w miejscu wywołania.
    drop(handle);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::account::{AccountType, ServerEntry};

    fn temp_root(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("mcc-store-{}-{tag}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("katalog testowy");
        dir
    }

    fn sample_account() -> Account {
        Account {
            id: "acc-1".into(),
            label: "Main".into(),
            account_type: AccountType::Microsoft,
            login: "user@example.com".into(),
            servers: vec![
                ServerEntry::auto("play.example.com:25565"),
                ServerEntry {
                    address: "other.net".into(),
                    version: Some("1.21".into()),
                },
            ],
            tab_order: 3,
        }
    }

    /// Pliki kopii zapasowych (`*.bad-*`) w katalogu danych.
    fn backups(root: &Path) -> Vec<String> {
        let mut names: Vec<String> = std::fs::read_dir(root)
            .expect("odczyt katalogu")
            .flatten()
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .filter(|name| name.contains(".bad-"))
            .collect();
        names.sort();
        names
    }

    #[tokio::test]
    async fn missing_files_yield_defaults() {
        let root = temp_root("missing");
        let store = Store::new(root);
        assert_eq!(store.load_settings().await, AppSettings::default());
        assert!(store.load_accounts().await.is_empty());
        // Brak pliku to nie problem: nic nie blokujemy i nie ostrzegamy.
        assert!(store.take_warnings().is_empty());
        store
            .save_accounts(&[sample_account()])
            .await
            .expect("zapis kont bez wcześniejszego odczytu");
    }

    #[tokio::test]
    async fn settings_round_trip_through_disk() {
        let store = Store::new(temp_root("settings"));
        let settings = AppSettings {
            language: "pl".into(),
            restore_tabs: false,
            ..Default::default()
        };

        store
            .save_settings(&settings)
            .await
            .expect("zapis ustawień");
        assert_eq!(store.load_settings().await, settings);
    }

    #[tokio::test]
    async fn session_round_trip_through_disk() {
        let store = Store::new(temp_root("session"));
        let tabs = vec![
            OpenTab {
                account_id: "acc-1".into(),
                server: "play.example.com".into(),
            },
            OpenTab {
                account_id: "acc-2".into(),
                server: "mc.example.net".into(),
            },
        ];

        store.save_session(&tabs).await.expect("zapis sesji");
        assert_eq!(store.load_session().await, tabs);

        // Brak pliku to brak zakładek, nie błąd.
        let empty = Store::new(temp_root("session-missing"));
        assert!(empty.load_session().await.is_empty());
    }

    /// Pliki tymczasowe w katalogu danych (po udanym zapisie ma ich nie być).
    fn temporary_files(root: &Path) -> Vec<String> {
        std::fs::read_dir(root)
            .expect("odczyt katalogu")
            .flatten()
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .filter(|name| name.contains(".tmp"))
            .collect()
    }

    /// R-35: sto równoległych zapisów tego samego pliku. Przy stałej nazwie
    /// `*.json.tmp` i bez blokady per plik zapisy nadpisywały sobie plik
    /// tymczasowy: część kończyła się błędem podmiany, a plik mógł zostać ucięty.
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn concurrent_writes_of_one_file_do_not_collide() {
        let root = temp_root("concurrent");
        let store = std::sync::Arc::new(Store::new(root.clone()));

        let mut tasks = Vec::new();
        for index in 0..100u64 {
            let store = std::sync::Arc::clone(&store);
            tasks.push(tokio::spawn(async move {
                let settings = AppSettings {
                    send_all_delay_ms: index,
                    ..Default::default()
                };
                store.save_settings(&settings).await.map(|()| index)
            }));
        }

        let mut written = Vec::new();
        for task in tasks {
            written.push(
                task.await
                    .expect("zadanie zapisu")
                    .expect("każdy równoległy zapis ma się udać"),
            );
        }
        assert_eq!(written.len(), 100);

        // Plik jest kompletny (poprawny JSON) i zgodny z jednym z zapisów,
        // a po plikach tymczasowych nie ma śladu.
        let raw = std::fs::read_to_string(root.join(SETTINGS_FILE)).expect("plik ustawień");
        let saved: AppSettings =
            serde_json::from_str(&raw).expect("poprawny JSON po równoległych zapisach");
        assert!(
            written.contains(&saved.send_all_delay_ms),
            "na dysku ma być jeden z zapisanych stanów, a nie mieszanka"
        );
        assert!(
            temporary_files(&root).is_empty(),
            "zostały pliki tymczasowe: {:?}",
            temporary_files(&root)
        );
    }

    /// R-34: treść idzie przez `write_all` + `sync_all`, więc plik ma dokładnie
    /// tyle bajtów, ile podano (krótsza treść nie zostawia ogona poprzedniej),
    /// a uchwyt jest domknięty przed podmianą - inaczej Windows nie pozwoliłby
    /// przemianować otwartego pliku.
    #[tokio::test]
    async fn write_and_flush_leaves_exact_content_and_closes_the_handle() {
        let root = temp_root("flush");
        let temp = root.join("settings.json.0.tmp");

        write_and_flush(&temp, b"pierwsza, dluzsza tresc")
            .await
            .expect("pierwszy zapis");
        write_and_flush(&temp, b"krotka")
            .await
            .expect("drugi zapis");

        assert_eq!(std::fs::read(&temp).expect("odczyt"), b"krotka");
        std::fs::rename(&temp, root.join(SETTINGS_FILE)).expect("rename po zapisie");
    }

    #[tokio::test]
    async fn accounts_round_trip_through_disk() {
        let store = Store::new(temp_root("accounts"));
        let accounts = vec![sample_account()];
        store.save_accounts(&accounts).await.expect("zapis kont");
        assert_eq!(store.load_accounts().await, accounts);
    }

    #[tokio::test]
    async fn presets_round_trip_through_disk() {
        let store = Store::new(temp_root("presets"));
        let presets = vec![
            Preset {
                id: "p1".into(),
                label: "/home".into(),
                text: "/send /home".into(),
                color: Some("blue".into()),
                enabled: true,
            },
            Preset {
                id: "p2".into(),
                label: "/reco".into(),
                text: "/send /reco".into(),
                color: None,
                enabled: false,
            },
        ];

        store.save_presets(&presets).await.expect("zapis presetów");
        assert_eq!(store.load_presets().await, presets);
        assert_eq!(
            store.reload_presets().await.expect("przeładowanie"),
            presets
        );
    }

    #[tokio::test]
    async fn missing_presets_file_is_an_empty_list() {
        let store = Store::new(temp_root("presets-missing"));
        assert!(store.load_presets().await.is_empty());
        assert!(store.reload_presets().await.expect("brak pliku").is_empty());
    }

    #[tokio::test]
    async fn reload_reports_a_broken_presets_file() {
        let root = temp_root("presets-corrupt");
        std::fs::write(root.join(PRESETS_FILE), b"[ { oops").expect("zapis uszkodzonego pliku");
        let store = Store::new(root);

        // Start nie może się wywalić na zepsutym pliku...
        assert!(store.load_presets().await.is_empty());
        // ...ale `Reload` musi powiedzieć użytkownikowi, że plik jest zepsuty.
        assert_eq!(
            store.reload_presets().await.unwrap_err().code,
            "presets.readFailed"
        );
    }

    #[tokio::test]
    async fn corrupt_file_falls_back_to_defaults() {
        let root = temp_root("corrupt");
        std::fs::write(root.join(SETTINGS_FILE), b"{ not json").expect("zapis uszkodzonego pliku");
        let store = Store::new(root);
        assert_eq!(store.load_settings().await, AppSettings::default());
    }

    #[tokio::test]
    async fn bad_entry_does_not_take_the_whole_accounts_list_down() {
        let root = temp_root("accounts-partial");
        // Drugi wpis nie ma loginu, więc nie da się go wczytać.
        let json = r#"[
            {"id":"acc-1","label":"Main","type":"microsoft","login":"a@b.c","servers":[],"tabOrder":0},
            {"id":"acc-2","type":"offline"}
        ]"#;
        std::fs::write(root.join(ACCOUNTS_FILE), json).expect("zapis kont");
        let store = Store::new(root.clone());

        let accounts = store.load_accounts().await;

        // Reszta listy zostaje, a o pominietym wpisie informujemy UI.
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].id, "acc-1");
        let warnings = store.take_warnings();
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].code, "store.entriesSkipped");
        assert_eq!(warnings[0].params["file"], ACCOUNTS_FILE);
        assert_eq!(warnings[0].params["count"], 1);
        // Kopia pliku z pominiętym wpisem zostaje obok.
        assert_eq!(backups(&root).len(), 1);
        // Dobry wpis można utrwalić - zapis nie jest wstrzymany.
        store.save_accounts(&accounts).await.expect("zapis kont");
    }

    #[tokio::test]
    async fn invalid_entry_is_dropped_and_counted() {
        let root = temp_root("accounts-invalid");
        // Identyfikator ze separatorem ścieżki (R-01) nie może trafić do panelu.
        let json = r#"[
            {"id":"../evil","label":"Zlo","type":"offline","login":"bot","servers":[],"tabOrder":0}
        ]"#;
        std::fs::write(root.join(ACCOUNTS_FILE), json).expect("zapis kont");
        let store = Store::new(root);

        assert!(store.load_accounts().await.is_empty());
        assert_eq!(store.take_warnings()[0].code, "store.entriesSkipped");
    }

    #[tokio::test]
    async fn one_bad_entry_is_reported_only_once() {
        let root = temp_root("accounts-partial-twice");
        std::fs::write(
            root.join(ACCOUNTS_FILE),
            r#"[
                {"id":"acc-1","label":"Main","type":"microsoft","login":"a@b.c","servers":[],"tabOrder":0},
                {"id":"acc-2","type":"offline"}
            ]"#,
        )
        .expect("zapis kont");
        let store = Store::new(root.clone());

        // Start czyta konta dwa razy (panel kont i zakładki sesji) - kopia
        // i ostrzeżenie mają powstać raz.
        assert_eq!(store.load_accounts().await.len(), 1);
        assert_eq!(store.load_accounts().await.len(), 1);

        assert_eq!(store.take_warnings().len(), 1);
        assert_eq!(backups(&root).len(), 1);
    }

    #[tokio::test]
    async fn unreadable_file_is_kept_and_its_write_is_paused() {
        let root = temp_root("accounts-quarantine");
        let broken = b"[ { oops".to_vec();
        std::fs::write(root.join(ACCOUNTS_FILE), &broken).expect("zapis uszkodzonego pliku");
        let store = Store::new(root.clone());

        assert!(store.load_accounts().await.is_empty());
        let warnings = store.take_warnings();
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].code, "store.fileUnreadable");
        assert_eq!(warnings[0].params["file"], ACCOUNTS_FILE);
        assert_eq!(backups(&root).len(), 1);

        // Zapis nie nadpisuje tego, czego nie umieliśmy odczytać.
        let error = store
            .save_accounts(&[sample_account()])
            .await
            .expect_err("zapis do nieczytelnego pliku ma być wstrzymany");
        assert_eq!(error.code, "store.fileQuarantined");
        assert_eq!(error.params["file"], ACCOUNTS_FILE);
        assert_eq!(
            std::fs::read(root.join(ACCOUNTS_FILE)).expect("odczyt pliku"),
            broken,
            "zawartość pliku nie może się zmienić"
        );
    }

    #[tokio::test]
    async fn a_successful_read_lifts_the_write_block() {
        let root = temp_root("accounts-fixed");
        std::fs::write(root.join(ACCOUNTS_FILE), b"{ oops").expect("zapis uszkodzonego pliku");
        let store = Store::new(root.clone());
        assert!(store.load_accounts().await.is_empty());
        assert!(
            store
                .save_accounts(&[sample_account()])
                .await
                .expect_err("zapis wstrzymany")
                .code
                .eq("store.fileQuarantined")
        );

        // Użytkownik poprawił plik - udany odczyt zdejmuje blokadę.
        std::fs::write(root.join(ACCOUNTS_FILE), b"[]").expect("poprawiony plik");
        assert!(store.load_accounts().await.is_empty());
        store
            .save_accounts(&[sample_account()])
            .await
            .expect("zapis po poprawieniu pliku");
    }

    #[tokio::test]
    async fn broken_settings_file_is_not_overwritten_by_defaults() {
        let root = temp_root("settings-quarantine");
        let broken = b"{ not json".to_vec();
        std::fs::write(root.join(SETTINGS_FILE), &broken).expect("zapis uszkodzonego pliku");
        let store = Store::new(root.clone());

        assert_eq!(store.load_settings().await, AppSettings::default());
        assert_eq!(store.take_warnings()[0].code, "store.fileUnreadable");

        let error = store
            .save_settings(&AppSettings::default())
            .await
            .expect_err("zapis do nieczytelnego pliku ma być wstrzymany");
        assert_eq!(error.code, "store.fileQuarantined");
        assert_eq!(
            std::fs::read(root.join(SETTINGS_FILE)).expect("odczyt pliku"),
            broken
        );
    }

    #[tokio::test]
    async fn fixing_presets_with_reload_lifts_the_block() {
        let root = temp_root("presets-quarantine");
        std::fs::write(root.join(PRESETS_FILE), b"[ { oops").expect("zapis uszkodzonego pliku");
        let store = Store::new(root.clone());

        assert!(store.load_presets().await.is_empty());
        assert!(
            store
                .save_presets(&[])
                .await
                .expect_err("zapis wstrzymany")
                .code
                .eq("store.fileQuarantined")
        );

        // `Reload` po poprawieniu pliku jest drogą naprawy bez restartu.
        std::fs::write(root.join(PRESETS_FILE), b"[]").expect("poprawiony plik");
        assert!(
            store
                .reload_presets()
                .await
                .expect("udany odczyt")
                .is_empty()
        );
        store.save_presets(&[]).await.expect("zapis po naprawie");
    }

    #[tokio::test]
    async fn accounts_file_has_no_password_field() {
        let store = Store::new(temp_root("secrets"));
        store
            .save_accounts(&[sample_account()])
            .await
            .expect("zapis kont");
        let raw = std::fs::read_to_string(store.root().join(ACCOUNTS_FILE)).expect("odczyt pliku");
        let lowered = raw.to_ascii_lowercase();
        assert!(
            !lowered.contains("password"),
            "plik kont nie może zawierać hasła"
        );
        assert!(
            !lowered.contains("token"),
            "plik kont nie może zawierać tokenu"
        );
    }
}

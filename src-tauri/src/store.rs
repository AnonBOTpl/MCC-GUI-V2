//! Trwałość ustawień i kont w katalogu danych aplikacji.
//!
//! Pliki: `settings.json`, `accounts.json`, `presets.json`, `session.json`. Zapis jest atomowy
//! (plik tymczasowy + podmiana), żeby przerwanie zapisu nie zostawiło
//! uszkodzonego pliku.
//!
//! W plikach **nie ma żadnych sekretów** - konto nie ma pola hasła, a sesje MCC
//! żyją wyłącznie w katalogach kont i aplikacja ich nie dotyka.

use std::path::{Path, PathBuf};

use serde::Serialize;
use serde::de::DeserializeOwned;

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

/// Czyta i zapisuje pliki JSON w katalogu danych aplikacji.
pub struct Store {
    root: PathBuf,
}

impl Store {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    /// Katalog danych aplikacji (tu też żyją katalogi kont).
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Wczytuje ustawienia. Brak pliku lub uszkodzony plik daje wartości domyślne
    /// (z ostrzeżeniem), żeby aplikacja zawsze wystartowała.
    pub async fn load_settings(&self) -> AppSettings {
        self.read_or_default(SETTINGS_FILE, AppSettings::default)
            .await
    }

    /// Zapisuje ustawienia atomowo.
    pub async fn save_settings(&self, settings: &AppSettings) -> Result<(), AppError> {
        self.write_json(SETTINGS_FILE, settings).await
    }

    /// Wczytuje zapisane konta (bez uruchamiania procesów).
    pub async fn load_accounts(&self) -> Vec<Account> {
        self.read_or_default(ACCOUNTS_FILE, Vec::new).await
    }

    /// Zapisuje konta atomowo.
    pub async fn save_accounts(&self, accounts: &[Account]) -> Result<(), AppError> {
        self.write_json(ACCOUNTS_FILE, &accounts.to_vec()).await
    }

    /// Wczytuje presety szybkich akcji przy starcie. Czytanie jest **łagodne**:
    /// brak pliku oznacza po prostu „brak presetów”, a plik ręcznie zepsuty
    /// logujemy i wracamy do pustej listy, żeby aplikacja wystartowała.
    pub async fn load_presets(&self) -> Vec<Preset> {
        self.read_or_default(PRESETS_FILE, Vec::new).await
    }

    /// Ponowne wczytanie presetów dla przycisku `Reload`.
    ///
    /// W przeciwieństwie do startu jest **ścisłe**: jeśli plik istnieje, ale jest
    /// niepoprawnym JSON-em (np. po ręcznej edycji), zwracamy błąd, bo wtedy
    /// użytkownik musi się o tym dowiedzieć - inaczej zobaczyłby pusty pasek
    /// bez wyjaśnienia i mógłby przypadkiem nadpisać plik.
    pub async fn reload_presets(&self) -> Result<Vec<Preset>, AppError> {
        let path = self.root.join(PRESETS_FILE);
        let raw = match tokio::fs::read_to_string(&path).await {
            Ok(raw) => raw,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => {
                tracing::warn!(%error, file = %path.display(), "nie udało się odczytać presetów");
                return Err(AppError::new("presets.readFailed"));
            }
        };

        serde_json::from_str(&raw).map_err(|error| {
            tracing::warn!(%error, file = %path.display(), "uszkodzony plik presetów");
            AppError::new("presets.readFailed")
        })
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

    /// Odczyt z odzyskiwaniem: brak pliku to nie błąd, uszkodzony plik logujemy.
    async fn read_or_default<T: DeserializeOwned>(&self, file: &str, default: fn() -> T) -> T {
        let path = self.root.join(file);
        let raw = match tokio::fs::read_to_string(&path).await {
            Ok(raw) => raw,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return default(),
            Err(error) => {
                tracing::warn!(%error, file = %path.display(), "nie udało się odczytać pliku");
                return default();
            }
        };

        match serde_json::from_str(&raw) {
            Ok(value) => value,
            Err(error) => {
                tracing::warn!(%error, file = %path.display(), "uszkodzony plik JSON; używam wartości domyślnych");
                default()
            }
        }
    }

    /// Atomowy zapis JSON: najpierw plik tymczasowy, potem podmiana.
    async fn write_json<T: Serialize>(&self, file: &str, value: &T) -> Result<(), AppError> {
        let path = self.root.join(file);
        let json = serde_json::to_string_pretty(value).map_err(|error| {
            tracing::error!(%error, file = %path.display(), "nie udało się zserializować danych");
            AppError::new("store.writeFailed")
        })?;

        let temp = path.with_extension("json.tmp");
        if let Err(error) = tokio::fs::write(&temp, json.as_bytes()).await {
            tracing::error!(%error, file = %temp.display(), "nie udało się zapisać pliku tymczasowego");
            return Err(AppError::new("store.writeFailed"));
        }
        if let Err(error) = tokio::fs::rename(&temp, &path).await {
            tracing::error!(%error, file = %path.display(), "nie udało się podmienić pliku");
            let _ = tokio::fs::remove_file(&temp).await;
            return Err(AppError::new("store.writeFailed"));
        }
        Ok(())
    }
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

    #[tokio::test]
    async fn missing_files_yield_defaults() {
        let root = temp_root("missing");
        let store = Store::new(root);
        assert_eq!(store.load_settings().await, AppSettings::default());
        assert!(store.load_accounts().await.is_empty());
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

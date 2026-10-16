//! Ustawienia aplikacji (kontrakt z frontendem) i walidacja ścieżki do MCC.
//!
//! Ustawienia trzymamy w `settings.json` w katalogu danych aplikacji
//! (zapis/odczyt w `store.rs`). Rust nie wysyła do UI tekstów dla człowieka,
//! więc błędy walidacji mają stabilne kody i parametry.
//!
//! `MCC_EXECUTABLE` zostaje jako fallback deweloperski: normalną drogą jest
//! ścieżka zapisana tutaj i wybrana w UI.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::AppError;

/// Domyślne opóźnienie między kontami przy „wyślij do wszystkich" (ms).
pub const DEFAULT_SEND_ALL_DELAY_MS: u64 = 250;

/// Górny limit opóźnienia między kontami (ms).
pub const MAX_SEND_ALL_DELAY_MS: u64 = 10_000;

/// Domyślny rozmiar bufora logu na konto (linie).
pub const DEFAULT_LOG_BUFFER_SIZE: usize = 5000;

/// Dopuszczalny zakres rozmiaru bufora logu.
pub const MIN_LOG_BUFFER_SIZE: usize = 200;

/// Górny limit rozmiaru bufora logu.
pub const MAX_LOG_BUFFER_SIZE: usize = 200_000;

/// Ile adresów serwerów pamiętamy w historii.
pub const MAX_SERVER_HISTORY: usize = 30;

/// Najdłuższy zapamiętywany adres serwera.
pub const MAX_SERVER_LEN: usize = 200;

/// Domyślna liczba dni przechowywania logów czatu (0 = bez limitu).
pub const DEFAULT_CHAT_LOG_RETENTION_DAYS: u32 = 30;

/// Górny limit retencji logów czatu (10 lat) - chroni przed literówką w polu
/// liczbowym, która kazałaby trzymać logi w nieskończoność.
pub const MAX_CHAT_LOG_RETENTION_DAYS: u32 = 3650;

/// Nazwa pliku próbnego przy sprawdzaniu, czy folder logów jest zapisywalny.
const CHAT_LOG_PROBE_FILE: &str = ".mcc-gui-write-test";

/// Języki obsługiwane przez aplikację (`en` = źródło prawdy).
const SUPPORTED_LANGUAGES: [&str; 2] = ["en", "pl"];

fn default_language() -> String {
    "en".to_string()
}

fn default_send_all_delay_ms() -> u64 {
    DEFAULT_SEND_ALL_DELAY_MS
}

fn default_log_buffer_size() -> usize {
    DEFAULT_LOG_BUFFER_SIZE
}

/// Zakładki są domyślnie przywracane; użytkownik może to wyłączyć w ustawieniach.
fn default_restore_tabs() -> bool {
    true
}

/// Domyślnie **pytamy** o potwierdzenie, gdy zamykanie przerywa połączenia.
/// Stare pliki `settings.json` bez tego pola dostają tę wartość dzięki `serde`.
fn default_confirm_close() -> bool {
    true
}

/// Logi czatu zapisujemy domyślnie (decyzja użytkownika) - wyłączenie jest
/// świadomym wyborem w ustawieniach.
fn default_chat_log_enabled() -> bool {
    true
}

fn default_chat_log_retention_days() -> u32 {
    DEFAULT_CHAT_LOG_RETENTION_DAYS
}

/// Ustawienia aplikacji. Nazwy pól w JSON-ie są `camelCase` (kontrakt z frontendem).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    /// Ścieżka do `MinecraftClient.exe`; `None` = jeszcze nie wybrana.
    #[serde(default)]
    pub mcc_path: Option<String>,
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default = "default_send_all_delay_ms")]
    pub send_all_delay_ms: u64,
    #[serde(default = "default_log_buffer_size")]
    pub log_buffer_size: usize,
    /// Czy przywracać zapisane zakładki przy starcie (decyzja użytkownika).
    #[serde(default = "default_restore_tabs")]
    pub restore_tabs: bool,
    /// Czy pytać o potwierdzenie, gdy zamknięcie okna przerywa działające
    /// połączenia. Użytkownik może też wybrać „nie pytaj więcej” w samym pytaniu.
    #[serde(default = "default_confirm_close")]
    pub confirm_close: bool,
    /// Czy zapisywać logi czatu do plików (`PLAN.md` §5, domyślnie tak).
    #[serde(default = "default_chat_log_enabled")]
    pub chat_log_enabled: bool,
    /// Własny folder logów czatu; `None` = `<katalog danych aplikacji>/logs`.
    #[serde(default)]
    pub chat_log_dir: Option<String>,
    /// Ile dni przechowywać logi czatu (`0` = bez limitu).
    #[serde(default = "default_chat_log_retention_days")]
    pub chat_log_retention_days: u32,
    /// Historia adresów serwerów, najnowsze pierwsze.
    #[serde(default)]
    pub server_history: Vec<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            mcc_path: None,
            language: default_language(),
            send_all_delay_ms: default_send_all_delay_ms(),
            log_buffer_size: default_log_buffer_size(),
            restore_tabs: default_restore_tabs(),
            confirm_close: default_confirm_close(),
            chat_log_enabled: default_chat_log_enabled(),
            chat_log_dir: None,
            chat_log_retention_days: default_chat_log_retention_days(),
            server_history: Vec::new(),
        }
    }
}

/// Sprawdza i porządkuje ścieżkę do `MinecraftClient.exe`.
///
/// Wymagania są celowo łagodne - plik musi istnieć, być plikiem (nie katalogiem)
/// i mieć rozszerzenie `.exe`. Nazwy nie wymuszamy (użytkownik może mieć własną
/// kopię), ale nietypową nazwę logujemy jako ostrzeżenie.
pub fn validate_mcc_path(raw: &str) -> Result<PathBuf, AppError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(AppError::new("mcc.pathEmpty"));
    }

    let path = PathBuf::from(trimmed);
    if !path.exists() {
        return Err(AppError::new("mcc.pathNotFound"));
    }
    if !path.is_file() {
        return Err(AppError::new("mcc.pathNotFile"));
    }

    let is_exe = path
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("exe"));
    if !is_exe {
        return Err(AppError::new("mcc.pathNotExe"));
    }

    let looks_like_mcc = path
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.to_ascii_lowercase().starts_with("minecraftclient"));
    if !looks_like_mcc {
        tracing::warn!(path = %path.display(), "wybrany plik nie nazywa się MinecraftClient*");
    }

    Ok(path)
}

/// Sprawdza folder logów czatu: musi być ścieżką absolutną, musi dać się
/// utworzyć i musi być zapisywalny.
///
/// Pusty tekst („folder domyślny”) obsługuje `normalize`, zanim tu trafi.
/// Zapisywalność sprawdzamy próbnym plikiem, bo sam istniejący katalog jeszcze
/// nic nie mówi (np. katalog tylko do odczytu albo dysk sieciowy bez prawa zapisu).
pub fn validate_chat_log_dir(raw: &str) -> Result<PathBuf, AppError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(AppError::new("chatlog.dirEmpty"));
    }

    let path = PathBuf::from(trimmed);
    if !path.is_absolute() {
        return Err(AppError::new("chatlog.dirNotAbsolute"));
    }

    if let Err(error) = std::fs::create_dir_all(&path) {
        tracing::warn!(%error, dir = %path.display(), "logi czatu: nie udało się utworzyć folderu");
        return Err(AppError::with_params(
            "chatlog.dirNotWritable",
            serde_json::json!({ "osError": error.raw_os_error().unwrap_or(-1) }),
        ));
    }

    let probe = path.join(CHAT_LOG_PROBE_FILE);
    if let Err(error) = std::fs::write(&probe, b"") {
        tracing::warn!(%error, dir = %path.display(), "logi czatu: folder nie jest zapisywalny");
        return Err(AppError::with_params(
            "chatlog.dirNotWritable",
            serde_json::json!({ "osError": error.raw_os_error().unwrap_or(-1) }),
        ));
    }
    let _ = std::fs::remove_file(&probe);

    Ok(path)
}

/// Sprawdza i normalizuje ustawienia przed zapisem.
///
/// Zwraca ustawienia gotowe do zapisania albo stabilny błąd walidacji.
pub fn normalize(mut settings: AppSettings) -> Result<AppSettings, AppError> {
    settings.language = settings.language.trim().to_ascii_lowercase();
    if !SUPPORTED_LANGUAGES.contains(&settings.language.as_str()) {
        return Err(AppError::new("settings.languageInvalid"));
    }

    if settings.send_all_delay_ms > MAX_SEND_ALL_DELAY_MS {
        return Err(AppError::with_params(
            "settings.delayOutOfRange",
            serde_json::json!({ "max": MAX_SEND_ALL_DELAY_MS }),
        ));
    }

    if !(MIN_LOG_BUFFER_SIZE..=MAX_LOG_BUFFER_SIZE).contains(&settings.log_buffer_size) {
        return Err(AppError::with_params(
            "settings.bufferOutOfRange",
            serde_json::json!({ "min": MIN_LOG_BUFFER_SIZE, "max": MAX_LOG_BUFFER_SIZE }),
        ));
    }

    if settings.chat_log_retention_days > MAX_CHAT_LOG_RETENTION_DAYS {
        return Err(AppError::with_params(
            "settings.retentionOutOfRange",
            serde_json::json!({ "max": MAX_CHAT_LOG_RETENTION_DAYS }),
        ));
    }

    // Pusta ścieżka oznacza „nie ustawiono"; niepusta musi przejść walidację.
    settings.mcc_path = match settings.mcc_path.as_deref().map(str::trim) {
        None | Some("") => None,
        Some(raw) => Some(validate_mcc_path(raw)?.to_string_lossy().into_owned()),
    };

    settings.chat_log_dir = match settings.chat_log_dir.as_deref().map(str::trim) {
        None | Some("") => None,
        Some(raw) => Some(validate_chat_log_dir(raw)?.to_string_lossy().into_owned()),
    };

    settings.server_history = normalize_server_history(&settings.server_history);

    Ok(settings)
}

/// Porządkuje historię serwerów: obcina, usuwa puste i duplikaty, ogranicza liczbę.
fn normalize_server_history(entries: &[String]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for entry in entries {
        let trimmed = entry.trim();
        if trimmed.is_empty() {
            continue;
        }
        let trimmed: String = trimmed.chars().take(MAX_SERVER_LEN).collect();
        if out.iter().any(|existing| existing == &trimmed) {
            continue;
        }
        out.push(trimmed);
        if out.len() >= MAX_SERVER_HISTORY {
            break;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_fill_in_from_empty_json() {
        let settings: AppSettings = serde_json::from_str("{}").expect("puste ustawienia");
        assert_eq!(settings, AppSettings::default());
        assert_eq!(settings.language, "en");
        assert!(settings.restore_tabs);
        // Stare pliki bez tego pola muszą dalej pytać o potwierdzenie.
        assert!(settings.confirm_close);
        assert_eq!(settings.send_all_delay_ms, DEFAULT_SEND_ALL_DELAY_MS);
        assert_eq!(settings.log_buffer_size, DEFAULT_LOG_BUFFER_SIZE);
        // Logi czatu są domyślnie włączone (decyzja użytkownika).
        assert!(settings.chat_log_enabled);
        assert_eq!(settings.chat_log_dir, None);
        assert_eq!(
            settings.chat_log_retention_days,
            DEFAULT_CHAT_LOG_RETENTION_DAYS
        );
    }

    #[test]
    fn round_trip_keeps_camel_case_contract() {
        let settings = AppSettings {
            mcc_path: None,
            language: "pl".into(),
            send_all_delay_ms: 400,
            log_buffer_size: 9000,
            restore_tabs: false,
            confirm_close: false,
            chat_log_enabled: false,
            chat_log_dir: Some("D:/mcc-logs".into()),
            chat_log_retention_days: 0,
            server_history: vec!["play.example.com:25565".into()],
        };
        let json = serde_json::to_string(&settings).expect("serializacja");
        assert!(json.contains("\"sendAllDelayMs\":400"));
        assert!(json.contains("\"logBufferSize\":9000"));
        assert!(json.contains("\"restoreTabs\":false"));
        assert!(json.contains("\"confirmClose\":false"));
        assert!(json.contains("\"chatLogEnabled\":false"));
        assert!(json.contains("\"chatLogRetentionDays\":0"));
        let back: AppSettings = serde_json::from_str(&json).expect("deserializacja");
        assert_eq!(back, settings);
    }
    #[test]
    fn normalize_rejects_unknown_language() {
        let settings = AppSettings {
            language: "de".into(),
            ..Default::default()
        };
        assert_eq!(
            normalize(settings).unwrap_err().code,
            "settings.languageInvalid"
        );
    }

    #[test]
    fn normalize_rejects_out_of_range_values() {
        let delay = AppSettings {
            send_all_delay_ms: MAX_SEND_ALL_DELAY_MS + 1,
            ..Default::default()
        };
        assert_eq!(
            normalize(delay).unwrap_err().code,
            "settings.delayOutOfRange"
        );

        let buffer = AppSettings {
            log_buffer_size: MIN_LOG_BUFFER_SIZE - 1,
            ..Default::default()
        };
        assert_eq!(
            normalize(buffer).unwrap_err().code,
            "settings.bufferOutOfRange"
        );
    }

    #[test]
    fn normalize_treats_blank_mcc_path_as_unset() {
        let settings = AppSettings {
            mcc_path: Some("   ".into()),
            ..Default::default()
        };
        let normalized = normalize(settings).expect("pusta ścieżka jest dozwolona");
        assert_eq!(normalized.mcc_path, None);
    }

    #[test]
    fn normalize_cleans_server_history() {
        let settings = AppSettings {
            server_history: vec![
                "  play.example.com  ".into(),
                "".into(),
                "play.example.com".into(),
                "other.example.com".into(),
                "x".repeat(MAX_SERVER_LEN + 50),
            ],
            ..Default::default()
        };
        let normalized = normalize(settings).expect("poprawna historia");
        assert_eq!(normalized.server_history.len(), 3);
        assert_eq!(normalized.server_history[0], "play.example.com");
        assert_eq!(normalized.server_history[1], "other.example.com");
        assert_eq!(normalized.server_history[2].chars().count(), MAX_SERVER_LEN);
    }

    #[test]
    fn normalize_rejects_missing_mcc_path() {
        let settings = AppSettings {
            mcc_path: Some("C:/definitely/not/here/MinecraftClient.exe".into()),
            ..Default::default()
        };
        assert_eq!(normalize(settings).unwrap_err().code, "mcc.pathNotFound");
    }

    #[test]
    fn normalize_rejects_retention_above_limit() {
        let settings = AppSettings {
            chat_log_retention_days: MAX_CHAT_LOG_RETENTION_DAYS + 1,
            ..Default::default()
        };
        assert_eq!(
            normalize(settings).unwrap_err().code,
            "settings.retentionOutOfRange"
        );
    }

    #[test]
    fn normalize_treats_blank_log_dir_as_default() {
        let settings = AppSettings {
            chat_log_dir: Some("   ".into()),
            ..Default::default()
        };
        let normalized = normalize(settings).expect("pusty folder jest dozwolony");
        assert_eq!(normalized.chat_log_dir, None);
    }

    #[test]
    fn log_dir_must_be_absolute_and_writable() {
        assert_eq!(
            validate_chat_log_dir("logs").unwrap_err().code,
            "chatlog.dirNotAbsolute"
        );

        // Nieistniejący folder w katalogu tymczasowym jest zakładany.
        let dir = std::env::temp_dir()
            .join(format!("mcc-chatlog-dir-{}", std::process::id()))
            .join("nested");
        let _ = std::fs::remove_dir_all(&dir);
        let validated =
            validate_chat_log_dir(&dir.to_string_lossy()).expect("folder do utworzenia");
        assert_eq!(validated, dir);
        assert!(dir.is_dir(), "walidacja ma założyć brakujący folder");
        // Plik próbny nie może zostawać na dysku.
        assert!(!dir.join(super::CHAT_LOG_PROBE_FILE).exists());

        // Ścieżka wskazująca na plik nie jest folderem.
        let file = dir.join("plik.txt");
        std::fs::write(&file, b"x").expect("plik testowy");
        assert!(validate_chat_log_dir(&file.to_string_lossy()).is_err());

        let _ = std::fs::remove_dir_all(
            std::env::temp_dir().join(format!("mcc-chatlog-dir-{}", std::process::id())),
        );
    }

    #[test]
    fn validate_mcc_path_rejects_directory_and_other_extension() {
        let dir = std::env::temp_dir();
        assert_eq!(
            validate_mcc_path(&dir.to_string_lossy()).unwrap_err().code,
            "mcc.pathNotFile"
        );
        assert_eq!(validate_mcc_path("").unwrap_err().code, "mcc.pathEmpty");
    }

    /// Nazwa pliku nie ma znaczenia: oficjalne wydania nazywają się
    /// `MinecraftClient-<data>-<build>-win-x64.exe`, a użytkownik może mieć
    /// dowolną własną kopię. Blokują tylko: brak pliku, katalog i brak `.exe`.
    #[test]
    fn validate_mcc_path_accepts_any_exe_name() {
        let release = std::env::temp_dir().join(format!(
            "MinecraftClient-20260929-519-win-x64-{}.exe",
            std::process::id()
        ));
        std::fs::write(&release, b"stub").expect("utworzenie pliku testowego");
        assert_eq!(
            validate_mcc_path(&release.to_string_lossy()).expect("poprawna ścieżka"),
            release
        );
        let _ = std::fs::remove_file(&release);

        let renamed = std::env::temp_dir().join(format!("mcc-{}-client.exe", std::process::id()));
        std::fs::write(&renamed, b"stub").expect("utworzenie pliku testowego");
        assert!(validate_mcc_path(&renamed.to_string_lossy()).is_ok());
        let _ = std::fs::remove_file(&renamed);
    }

    #[test]
    fn validate_mcc_path_accepts_existing_exe() {
        let unique = format!("mcc-test-{}-MinecraftClient.exe", std::process::id());
        let file = std::env::temp_dir().join(unique);
        std::fs::write(&file, b"stub").expect("utworzenie pliku testowego");
        let validated = validate_mcc_path(&file.to_string_lossy()).expect("poprawna ścieżka");
        assert_eq!(validated, file);
        let _ = std::fs::remove_file(&file);

        let txt = file.with_extension("txt");
        std::fs::write(&txt, b"stub").expect("utworzenie pliku testowego");
        assert_eq!(
            validate_mcc_path(&txt.to_string_lossy()).unwrap_err().code,
            "mcc.pathNotExe"
        );
        let _ = std::fs::remove_file(&txt);
    }
}

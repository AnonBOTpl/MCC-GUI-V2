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

    // Pusta ścieżka oznacza „nie ustawiono"; niepusta musi przejść walidację.
    settings.mcc_path = match settings.mcc_path.as_deref().map(str::trim) {
        None | Some("") => None,
        Some(raw) => Some(validate_mcc_path(raw)?.to_string_lossy().into_owned()),
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
        assert_eq!(settings.send_all_delay_ms, DEFAULT_SEND_ALL_DELAY_MS);
        assert_eq!(settings.log_buffer_size, DEFAULT_LOG_BUFFER_SIZE);
    }

    #[test]
    fn round_trip_keeps_camel_case_contract() {
        let settings = AppSettings {
            mcc_path: None,
            language: "pl".into(),
            send_all_delay_ms: 400,
            log_buffer_size: 9000,
            restore_tabs: false,
            server_history: vec!["play.example.com:25565".into()],
        };
        let json = serde_json::to_string(&settings).expect("serializacja");
        assert!(json.contains("\"sendAllDelayMs\":400"));
        assert!(json.contains("\"logBufferSize\":9000"));
        assert!(json.contains("\"restoreTabs\":false"));
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
    fn validate_mcc_path_rejects_directory_and_other_extension() {
        let dir = std::env::temp_dir();
        assert_eq!(
            validate_mcc_path(&dir.to_string_lossy()).unwrap_err().code,
            "mcc.pathNotFile"
        );
        assert_eq!(validate_mcc_path("").unwrap_err().code, "mcc.pathEmpty");
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

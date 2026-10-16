//! Punkt wejścia backendu Tauri.
//!
//! Etap 1: menedżer procesów MCC (jedno konto = jeden `MinecraftClient.exe`).
//! Ścieżkę do `MinecraftClient.exe` pobieramy tymczasowo ze zmiennej środowiskowej
//! `MCC_EXECUTABLE`; wybór z UI i zapis w ustawieniach dojdą w Etapie 4.

pub mod account;
pub mod connection;
pub mod device_code;
pub mod diag;
pub mod error;
pub mod events;
pub mod preset;
pub mod process_manager;
pub mod session;
pub mod settings;
pub mod store;

mod commands;
mod input;
mod job_object;
mod launch;

use std::path::PathBuf;
use std::sync::Arc;

use tauri::{Manager, WindowEvent};
use tracing_subscriber::EnvFilter;

use commands::{AppState, TauriSink};
use process_manager::ProcessManager;
use settings::AppSettings;
use store::Store;

/// Inicjalizuje logowanie przez `tracing`. Poziom sterowany zmienną `RUST_LOG`.
fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
}

/// Wyznacza ścieżkę do `MinecraftClient.exe`.
///
/// Kolejność: najpierw ścieżka zapisana w ustawieniach (wybrana w UI), a jeśli
/// jej nie ma albo jest nieaktualna - zmienna `MCC_EXECUTABLE`, która została
/// jako wygodny fallback deweloperski.
fn resolve_mcc_exe(settings: &AppSettings) -> Option<PathBuf> {
    if let Some(saved) = settings.mcc_path.as_deref() {
        match settings::validate_mcc_path(saved) {
            Ok(path) => return Some(path),
            Err(error) => {
                tracing::warn!(%error, path = %saved, "zapisana ścieżka MCC jest nieaktualna");
            }
        }
    }

    std::env::var("MCC_EXECUTABLE")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .map(PathBuf::from)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    init_tracing();

    tauri::Builder::default()
        // Otwieranie adresu logowania (tylko `https://` z promptu MCC).
        .plugin(tauri_plugin_opener::init())
        // Schowek obsługujemy z Rusta, żeby WebView2 nie pokazywał natywnego
        // okna uprawnień przy `navigator.clipboard`.
        .plugin(tauri_plugin_clipboard_manager::init())
        // Natywny wybór pliku z `MinecraftClient.exe` (wołany z Rusta, bez uprawnień JS).
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let handle = app.handle().clone();

            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;

            let accounts_root = data_dir.join("accounts");
            std::fs::create_dir_all(&accounts_root)?;

            // Znacznik startu w `diag.log`, dopisywany tylko przy włączonej
            // diagnostyce (`MCC_GUI_DIAG=1` - patrz `src-tauri/src/diag.rs`).
            // Dzięki temu widać, czy nowe uruchomienie trafiło do tego samego pliku
            // co poprzednie, i w jakiej wersji.
            if diag::enabled() {
                let marker = serde_json::json!({
                    "event": "start",
                    "version": env!("CARGO_PKG_VERSION"),
                    "t": diag::now_ms(),
                });
                if let Err(error) = diag::append(&data_dir, &marker) {
                    tracing::warn!(%error, "diagnostyka: nie udało się zapisać znacznika startu");
                }
            }

            // Ustawienia wczytujemy synchronicznie: jesteśmy jeszcze przed
            // startem pętli zdarzeń, a menedżer potrzebuje ścieżki MCC od razu.
            let store = Arc::new(Store::new(data_dir));
            let loaded = tauri::async_runtime::block_on(store.load_settings());
            let settings = settings::normalize(loaded).unwrap_or_else(|error| {
                tracing::warn!(%error, "niepoprawne ustawienia na dysku; używam domyślnych");
                AppSettings::default()
            });

            let mcc_exe = resolve_mcc_exe(&settings);
            if mcc_exe.is_none() {
                tracing::warn!(
                    "ścieżka MCC nie jest ustawiona - dodanie konta zwróci błąd mcc.executableNotSet"
                );
            }

            let sink = Arc::new(TauriSink::new(handle));
            let manager = Arc::new(ProcessManager::new(mcc_exe, accounts_root, sink)?);
            app.manage(AppState {
                manager: manager.clone(),
                store,
                settings: tokio::sync::Mutex::new(settings),
            });

            Ok(())
        })
        // Przy zamknięciu okna próbujemy łagodnie zatrzymać procesy; gwarancją
        // braku osieroconych procesów jest Job Object (kill-on-close).
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { .. } = event {
                let app = window.app_handle().clone();
                tauri::async_runtime::spawn(async move {
                    app.state::<AppState>().manager.stop_all().await;
                });
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::add_account,
            commands::list_accounts,
            commands::save_accounts,
            commands::remove_account,
            commands::add_server,
            commands::remove_server,
            commands::start_connection,
            commands::stop_connection,
            commands::restart_connection,
            commands::send_input,
            commands::send_input_all,
            commands::open_login_url,
            commands::copy_login_code,
            commands::get_settings,
            commands::save_settings,
            commands::pick_mcc_executable,
            commands::list_presets,
            commands::save_presets,
            commands::reload_presets,
            commands::get_open_tabs,
            commands::save_open_tabs,
            commands::diag_enabled,
            commands::diag_report,
        ])
        .run(tauri::generate_context!())
        .expect("nie udało się uruchomić aplikacji Tauri");
}

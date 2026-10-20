//! Punkt wejścia backendu Tauri.
//!
//! Etap 1: menedżer procesów MCC (jedno konto = jeden `MinecraftClient.exe`).
//! Ścieżkę do `MinecraftClient.exe` pobieramy tymczasowo ze zmiennej środowiskowej
//! `MCC_EXECUTABLE`; wybór z UI i zapis w ustawieniach dojdą w Etapie 4.

pub mod account;
pub mod chatlog;
pub mod clipboard;
pub mod close;
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
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use tauri::{Emitter, Manager, WindowEvent};
use tracing_subscriber::EnvFilter;

use chatlog::ChatLog;
use commands::{AppState, TauriSink};
use process_manager::ProcessManager;
use settings::AppSettings;
use store::Store;

/// Inicjalizuje logowanie przez `tracing`. Poziom sterowany zmienną `RUST_LOG`.
fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
}

/// Wyznacza ścieżkę do `MinecraftClient.exe`: wyłącznie z ustawień.
fn resolve_mcc_exe(settings: &AppSettings) -> Option<PathBuf> {
    resolve_mcc_exe_with(settings, |name| std::env::var(name).ok())
}

/// Sedno [`resolve_mcc_exe`] z wstrzykiwanym odczytem środowiska (do testów).
///
/// Kolejność: ścieżka zapisana w ustawieniach (wybrana w UI), a jeśli jej nie ma
/// albo jest nieaktualna - zmienna `MCC_EXECUTABLE` (R-33). Ta zmienna to
/// **fallback deweloperski** ustawiany przez `run-dev.bat`, więc w wersji wydanej
/// jest wyłączony: użytkownik instalatora ma wskazać plik w ustawieniach, a nie
/// dziedziczyć ścieżkę ze środowiska.
fn resolve_mcc_exe_with(
    settings: &AppSettings,
    read_env: impl Fn(&str) -> Option<String>,
) -> Option<PathBuf> {
    if let Some(saved) = settings.mcc_path.as_deref() {
        match settings::validate_mcc_path(saved) {
            Ok(path) => return Some(path),
            Err(error) => {
                tracing::warn!(%error, path = %saved, "zapisana ścieżka MCC jest nieaktualna");
            }
        }
    }

    if !cfg!(debug_assertions) {
        return None;
    }

    read_env("MCC_EXECUTABLE")
        .filter(|value| !value.trim().is_empty())
        .map(PathBuf::from)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    init_tracing();

    let builder = tauri::Builder::default()
        // Otwieranie adresu logowania (tylko `https://` z promptu MCC).
        .plugin(tauri_plugin_opener::init())
        // Schowek obsługujemy z Rusta, żeby WebView2 nie pokazywał natywnego
        // okna uprawnień przy `navigator.clipboard`.
        .plugin(tauri_plugin_clipboard_manager::init())
        // Natywny wybór pliku z `MinecraftClient.exe` (wołany z Rusta, bez uprawnień JS).
        .plugin(tauri_plugin_dialog::init());

    // Jedna instancja aplikacji **tylko w buildzie release**: `run-dev.bat`
    // uruchamia build debug, więc kopia deweloperska musi dać się trzymać obok
    // zainstalowanej wersji. Drugie uruchomienie tylko wyciąga istniejące okno
    // na wierzch i kończy się samo.
    #[cfg(not(debug_assertions))]
    let builder = builder.plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
        match app.get_webview_window("main") {
            Some(window) => {
                // Kolejność jak w Windows: najpierw odminimalizuj, potem pokaż
                // i dopiero na końcu ustaw fokus.
                let _ = window.unminimize();
                let _ = window.show();
                let _ = window.set_focus();
            }
            None => tracing::warn!("drugie uruchomienie: nie znalazłem okna 'main'"),
        }
    }));

    builder
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
            // Ustawienia z dysku normalizujemy **łagodnie** (R-10): nieaktualna
            // ścieżka do MCC (np. po przeniesieniu folderu) albo jedna zła liczba
            // zeruje tylko swoje pole, a nie cały obiekt. Wcześniej taki błąd
            // zastępował wszystkie ustawienia domyślnymi, a następny zapis
            // utrwalał je na dysku.
            let settings::LoadedSettings { settings, problems } =
                settings::normalize_loaded(loaded);
            for problem in &problems {
                tracing::warn!(
                    code = %problem.code,
                    "ustawienia: pole nie przeszło walidacji, używam wartości domyślnej"
                );
            }

            let mcc_exe = resolve_mcc_exe(&settings);
            if mcc_exe.is_none() {
                tracing::warn!(
                    "ścieżka MCC nie jest ustawiona - dodanie konta zwróci błąd mcc.executableNotSet"
                );
            }

            // Logi czatu (`PLAN.md` §5): serwis zapisuje linie wyjścia MCC
            // w osobnym zadaniu, a błędy zapisu odbieramy tutaj i pokazujemy
            // jako zwykły toast (`app://error`).
            let (error_tx, mut error_rx) = tokio::sync::mpsc::unbounded_channel();
            let chat_log = ChatLog::new(store.root().join(chatlog::LOGS_DIR), error_tx);
            chat_log.apply_settings(&settings);
            let saved_accounts = tauri::async_runtime::block_on(store.load_accounts());
            chat_log.set_account_labels(&saved_accounts);

            let sink = Arc::new(TauriSink::new(handle.clone(), chat_log.clone()));
            let manager = Arc::new(ProcessManager::new(mcc_exe, accounts_root, sink)?);
            app.manage(AppState {
                manager: manager.clone(),
                store,
                settings: tokio::sync::Mutex::new(settings),
                settings_problems: std::sync::Mutex::new(problems),
                chat_log: chat_log.clone(),
                close_confirmed: AtomicBool::new(false),
            });

            let errors_handle = handle.clone();
            tauri::async_runtime::spawn(async move {
                while let Some(error) = error_rx.recv().await {
                    if let Err(emit_error) = errors_handle.emit(
                        "app://error",
                        events::ErrorPayload {
                            code: error.code,
                            params: error.params,
                        },
                    ) {
                        tracing::warn!(%emit_error, "nie udało się pokazać błędu zapisu logów czatu");
                    }
                }
            });

            // Retencja logów czatu: czyszczenie przy starcie (pierwszy „tick”
            // przedawnia się natychmiast) i raz na dobę.
            let retention_log = chat_log.clone();
            tauri::async_runtime::spawn(async move {
                let mut ticker = tokio::time::interval(Duration::from_secs(24 * 60 * 60));
                ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
                loop {
                    ticker.tick().await;
                    let root = retention_log.root();
                    let days = retention_log.retention_days();
                    let today = chatlog::today();
                    match tokio::task::spawn_blocking(move || {
                        chatlog::purge_expired(&root, today, days)
                    })
                    .await
                    {
                        Ok(removed) => tracing::info!(removed, "logi czatu: czyszczenie zakończone"),
                        Err(error) => tracing::warn!(%error, "logi czatu: czyszczenie nie powiodło się"),
                    }
                }
            });

            Ok(())
        })
        // Zamknięcie okna obsługujemy w **jednym** miejscu, więc ta sama reguła
        // działa dla przycisku w pasku tytułu, Alt+F4 i paska zadań.
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let app = window.app_handle().clone();

                // Bez zamontowanego stanu nie ma czego sprawdzać - zamykamy jak dawniej.
                let Some(state) = app.try_state::<AppState>() else {
                    return;
                };

                // Obie blokady trzymamy tylko na chwilę (licznik połączeń i jedno pole
                // ustawień), a ten wątek nie jest w kontekście asynchronicznym, więc
                // `block_on` jest tu bezpieczny - tak samo jak przy starcie aplikacji.
                let active = tauri::async_runtime::block_on(state.manager.active_count());
                let confirm_setting = tauri::async_runtime::block_on(async {
                    state.settings.lock().await.confirm_close
                });
                let already_confirmed = state.close_confirmed.load(Ordering::SeqCst);

                if !close::should_confirm(confirm_setting, already_confirmed, active) {
                    // Zamykamy: procesy zatrzymujemy w tle, a brak osieroconych
                    // procesów gwarantuje dodatkowo Job Object (kill-on-close).
                    tauri::async_runtime::spawn(async move {
                        app.state::<AppState>().manager.stop_all().await;
                    });
                    return;
                }

                // Wstrzymujemy zamknięcie i pytamy użytkownika w interfejsie.
                // Odpowiedzią jest komenda `close_app`; gdy użytkownik anuluje,
                // nie dzieje się nic i okno zostaje otwarte.
                api.prevent_close();
                if let Err(error) = app.emit(
                    "app://close-requested",
                    events::CloseRequestedPayload { active },
                ) {
                    tracing::error!(%error, "nie udało się zapytać o zamknięcie aplikacji");
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::add_account,
            commands::list_accounts,
            commands::register_accounts,
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
            commands::copy_text,
            commands::get_settings,
            commands::save_settings,
            commands::pick_mcc_executable,
            commands::chat_log_dir,
            commands::open_chat_log_folder,
            commands::pick_chat_log_folder,
            commands::list_presets,
            commands::save_presets,
            commands::reload_presets,
            commands::get_open_tabs,
            commands::save_open_tabs,
            commands::close_app,
            commands::diag_enabled,
            commands::diag_report,
        ])
        .run(tauri::generate_context!())
        .expect("nie udało się uruchomić aplikacji Tauri");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Ścieżka do pliku `.exe`, który naprawdę istnieje.
    ///
    /// Walidacja wymaga pliku o rozszerzeniu `.exe`, więc atrapa musi być na dysku.
    fn temp_exe(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("mcc-lib-{}-{tag}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("katalog testowy");
        let path = dir.join("MinecraftClient.exe");
        std::fs::write(&path, b"").expect("plik testowy");
        path
    }

    #[test]
    fn settings_path_wins_over_the_environment() {
        let saved = temp_exe("saved");
        let settings = AppSettings {
            mcc_path: Some(saved.to_string_lossy().into_owned()),
            ..Default::default()
        };

        let resolved = resolve_mcc_exe_with(&settings, |_| {
            Some(temp_exe("env").to_string_lossy().into_owned())
        });

        assert_eq!(resolved, Some(saved));
    }

    #[test]
    fn env_fallback_is_development_only() {
        // Bez zapisanej ścieżki zostaje tylko zmienna środowiskowa - i wolno jej
        // użyć **wyłącznie** w buildzie deweloperskim. Ten sam kod idzie w testach
        // wydania, więc `cargo test --release` sprawdza bramkę.
        let settings = AppSettings::default();
        let from_env = temp_exe("fallback");

        let resolved = resolve_mcc_exe_with(&settings, |name| {
            assert_eq!(name, "MCC_EXECUTABLE");
            Some(from_env.to_string_lossy().into_owned())
        });

        assert_eq!(
            resolved.is_some(),
            cfg!(debug_assertions),
            "fallback MCC_EXECUTABLE ma działać tylko w buildzie deweloperskim"
        );
    }

    #[test]
    fn empty_env_value_is_not_a_path() {
        let settings = AppSettings::default();

        let resolved = resolve_mcc_exe_with(&settings, |_| Some("   ".into()));

        assert!(resolved.is_none());
    }
}

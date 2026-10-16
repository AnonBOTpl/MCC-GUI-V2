//! Warstwa Tauri: komendy (kontrakt w `src/api/contract.ts`) i wysyłanie zdarzeń.
//! Tu jest jedyne miejsce, które zna Tauri - `process_manager` jest niezależny.
//!
//! W modelu „konto + lista serwerów" **identyfikator połączenia** (`konto::serwer`)
//! jest tym, co zdarzenia niosą w polu `accountId` - po stronie UI mapa statusów
//! i logów jest kluczowana połączeniem, nie kontem.

use std::path::PathBuf;
use std::sync::Arc;

use serde_json::{Value, json};
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::sync::Mutex;

use crate::account::{Account, AccountStatus};
use crate::error::AppError;
use crate::events::{
    DeviceCodePayload, ErrorPayload, EventSink, JoinedPayload, LogPayload, LogStream, StatusPayload,
};
use crate::preset::Preset;
use crate::process_manager::ProcessManager;
use crate::session::{self, OpenTab};
use crate::settings::AppSettings;
use crate::store::Store;

/// Implementacja `EventSink`, która wysyła zdarzenia przez Tauri.
pub struct TauriSink {
    app: AppHandle,
}

impl TauriSink {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }
}

impl EventSink for TauriSink {
    fn emit_log(&self, account_id: &str, stream: LogStream, line: String, ts: i64) {
        let _ = self.app.emit(
            "mcc://log",
            LogPayload {
                account_id: account_id.to_string(),
                stream,
                line,
                ts,
            },
        );
    }

    fn emit_status(&self, account_id: &str, status: AccountStatus) {
        let _ = self.app.emit(
            "mcc://status",
            StatusPayload {
                account_id: account_id.to_string(),
                status,
            },
        );
    }

    fn emit_error(&self, code: &str, params: Value) {
        let _ = self.app.emit(
            "app://error",
            ErrorPayload {
                code: code.to_string(),
                params,
            },
        );
    }

    fn emit_device_code(&self, account_id: &str, url: String, code: String) {
        let _ = self.app.emit(
            "mcc://device-code",
            DeviceCodePayload {
                account_id: account_id.to_string(),
                url,
                code,
            },
        );
    }

    fn emit_joined(&self, account_id: &str) {
        let _ = self.app.emit(
            "mcc://joined",
            JoinedPayload {
                account_id: account_id.to_string(),
            },
        );
    }
}

/// Stan współdzielony przez komendy.
pub struct AppState {
    pub manager: Arc<ProcessManager<TauriSink>>,
    pub store: Arc<Store>,
    /// Bieżące ustawienia (znormalizowane). Trzymane też w pamięci, żeby
    /// komendy nie czytały pliku przy każdym wywołaniu.
    pub settings: Mutex<AppSettings>,
}

/// Otwiera adres logowania Microsoft w domyślnej przeglądarce.
///
/// Dopuszczamy otwarcie tylko `https://` i tylko adresu pochodzącego
/// z promptu logowania MCC. Walidację robimy **tutaj**, po stronie Rusta, żeby
/// frontend nie mógł podać dowolnego adresu.
#[tauri::command]
pub async fn open_login_url(app: AppHandle, url: String) -> Result<(), AppError> {
    use tauri_plugin_opener::OpenerExt;

    if !crate::device_code::is_allowed_login_url(&url) {
        return Err(AppError::new("url.notAllowed"));
    }
    app.opener().open_url(&url, None::<&str>).map_err(|error| {
        tracing::warn!(%error, "nie udało się otworzyć adresu logowania");
        AppError::new("url.openFailed")
    })
}

/// Kopiuje kod logowania do schowka systemowego.
///
/// Robimy to po stronie Rusta (plugin schowka), a nie `navigator.clipboard`,
/// bo WebView2 pokazuje wtedy natywne okno uprawnień. Kod pochodzi z promptu
/// logowania MCC, więc ograniczamy długość i dopuszczamy tylko znaki kodu.
#[tauri::command]
pub async fn copy_login_code(app: AppHandle, code: String) -> Result<(), AppError> {
    use tauri_plugin_clipboard_manager::ClipboardExt;

    if !crate::device_code::is_allowed_login_code(&code) {
        return Err(AppError::new("clipboard.writeFailed"));
    }
    app.clipboard().write_text(code).map_err(|error| {
        tracing::warn!(%error, "nie udało się skopiować kodu logowania");
        AppError::new("clipboard.writeFailed")
    })
}

/// Zwraca bieżące ustawienia aplikacji.
#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> Result<AppSettings, AppError> {
    Ok(state.settings.lock().await.clone())
}

/// Zapisuje ustawienia: waliduje, utrwala na dysku i przekazuje ścieżkę MCC
/// menedżerowi procesów.
#[tauri::command]
pub async fn save_settings(
    state: State<'_, AppState>,
    settings: AppSettings,
) -> Result<AppSettings, AppError> {
    let normalized = crate::settings::normalize(settings)?;
    state.store.save_settings(&normalized).await?;
    state
        .manager
        .set_mcc_exe(normalized.mcc_path.as_deref().map(PathBuf::from));
    *state.settings.lock().await = normalized.clone();
    Ok(normalized)
}

/// Otwiera natywne okno wyboru pliku i zwraca zwalidowaną ścieżkę do
/// `MinecraftClient.exe`. `None` oznacza, że użytkownik anulował.
#[tauri::command]
pub async fn pick_mcc_executable(app: AppHandle) -> Result<Option<String>, AppError> {
    use tauri_plugin_dialog::DialogExt;

    let (tx, rx) = tokio::sync::oneshot::channel();
    app.dialog()
        .file()
        .set_title("MinecraftClient.exe")
        .add_filter("Minecraft Console Client", &["exe"])
        .pick_file(move |picked| {
            let _ = tx.send(picked);
        });

    let picked = rx.await.map_err(|_| AppError::new("dialog.failed"))?;
    let Some(file_path) = picked else {
        // Anulowanie nie jest błędem.
        return Ok(None);
    };

    let path = file_path.into_path().map_err(|error| {
        tracing::warn!(%error, "nie udało się odczytać wybranej ścieżki");
        AppError::new("dialog.failed")
    })?;

    let validated = crate::settings::validate_mcc_path(&path.to_string_lossy())?;
    Ok(Some(validated.to_string_lossy().into_owned()))
}

/// Zwraca presety szybkich akcji zapisane w `presets.json`.
///
/// Start jest łagodny: brak pliku to po prostu pusta lista, a ręcznie zepsuty
/// plik logujemy i zwracamy pustą listę, żeby aplikacja zawsze wystartowała.
#[tauri::command]
pub async fn list_presets(state: State<'_, AppState>) -> Result<Vec<Preset>, AppError> {
    Ok(state.store.load_presets().await)
}

/// Zapisuje presety: waliduje listę i utrwala ją atomowo. Zwraca wersję
/// znormalizowaną, więc UI nigdy nie pokazuje danych, których backend nie przyjął.
#[tauri::command]
pub async fn save_presets(
    state: State<'_, AppState>,
    presets: Vec<Preset>,
) -> Result<Vec<Preset>, AppError> {
    let normalized = crate::preset::validate_presets(presets)?;
    state.store.save_presets(&normalized).await?;
    Ok(normalized)
}

/// Wczytuje `presets.json` ponownie (przycisk `Reload`), bez łagodzenia błędów:
/// uszkodzony plik zwraca `presets.readFailed`, żeby użytkownik wiedział, że
/// jego ręczna edycja się nie sparsowała.
#[tauri::command]
pub async fn reload_presets(state: State<'_, AppState>) -> Result<Vec<Preset>, AppError> {
    state.store.reload_presets().await
}

/// Zwraca konta zapisane w `accounts.json` (bez uruchamiania procesów).
/// Używane przy starcie aplikacji (Etap 4).
#[tauri::command]
pub async fn list_accounts(state: State<'_, AppState>) -> Result<Vec<Account>, AppError> {
    Ok(state.store.load_accounts().await)
}

/// Zapisuje listę kont i rejestruje je w menedżerze (bez uruchamiania procesów).
/// Lista jest walidowana i normalizowana.
#[tauri::command]
pub async fn save_accounts(
    state: State<'_, AppState>,
    accounts: Vec<Account>,
) -> Result<(), AppError> {
    let normalized = crate::account::validate_accounts(accounts)?;
    state.store.save_accounts(&normalized).await?;
    state.manager.sync_accounts(&normalized).await;
    Ok(())
}

/// Dodaje konto (bez uruchamiania połączeń - użytkownik startuje je sam).
#[tauri::command]
pub async fn add_account(state: State<'_, AppState>, account: Account) -> Result<(), AppError> {
    state.manager.add_account(account).await
}

/// Usuwa konto i zatrzymuje wszystkie jego połączenia.
#[tauri::command]
pub async fn remove_account(state: State<'_, AppState>, id: String) -> Result<(), AppError> {
    state.manager.remove_account(&id).await
}

/// Dopisuje serwer do konta (bez uruchamiania procesu). Ten sam adres
/// aktualizuje wymuszoną wersję Minecrafta (`None` = `auto`).
#[tauri::command]
pub async fn add_server(
    state: State<'_, AppState>,
    account_id: String,
    server: String,
    version: Option<String>,
) -> Result<(), AppError> {
    state
        .manager
        .add_server(&account_id, &server, version.as_deref())
        .await
}

/// Usuwa serwer z konta, zatrzymując jego połączenie, jeśli działa.
#[tauri::command]
pub async fn remove_server(
    state: State<'_, AppState>,
    account_id: String,
    server: String,
) -> Result<(), AppError> {
    state.manager.remove_server(&account_id, &server).await
}

/// Uruchamia połączenie (konto + serwer).
#[tauri::command]
pub async fn start_connection(
    state: State<'_, AppState>,
    account_id: String,
    server: String,
) -> Result<(), AppError> {
    state.manager.start_connection(&account_id, &server).await
}

/// Zatrzymuje połączenie po jego identyfikatorze (`konto::serwer`).
#[tauri::command]
pub async fn stop_connection(state: State<'_, AppState>, id: String) -> Result<(), AppError> {
    state.manager.stop_connection(&id).await
}

/// Ponownie uruchamia połączenie.
#[tauri::command]
pub async fn restart_connection(
    state: State<'_, AppState>,
    account_id: String,
    server: String,
) -> Result<(), AppError> {
    state.manager.restart_connection(&account_id, &server).await
}

/// Wysyła wpis do jednego połączenia.
#[tauri::command]
pub async fn send_input(
    state: State<'_, AppState>,
    id: String,
    text: String,
) -> Result<(), AppError> {
    state.manager.send_input(&id, &text).await
}

/// Wysyła wpis do wszystkich działających połączeń.
#[tauri::command]
pub async fn send_input_all(
    state: State<'_, AppState>,
    text: String,
) -> Result<Vec<String>, AppError> {
    let delay = state.settings.lock().await.send_all_delay_ms;
    state.manager.send_input_all(&text, delay).await
}

/// Zwraca zakładki z poprzedniej sesji.
///
/// Odcedzanie względem zapisanych kont robimy tutaj, a nie w UI: konto mogło
/// zostać usunięte, a serwer skasowany, więc plik sesji jest danymi wejściowymi
/// jak każde inne - walidacja wejścia należy do Rusta.
#[tauri::command]
pub async fn get_open_tabs(state: State<'_, AppState>) -> Result<Vec<OpenTab>, AppError> {
    let tabs = state.store.load_session().await;
    let accounts = state.store.load_accounts().await;
    let (kept, dropped) = session::sanitize_open_tabs(tabs, &accounts);
    if dropped > 0 {
        tracing::warn!(dropped, "sesja: pominięto zakładki bez konta albo serwera");
    }
    Ok(kept)
}

/// Zapisuje zakładki sesji (bez uruchamiania czegokolwiek).
///
/// Zwracamy listę po odcedzeniu - tak jak `save_settings` oddaje wersję
/// znormalizowaną, żeby UI nie trzymało stanu, którego backend nie przyjął.
#[tauri::command]
pub async fn save_open_tabs(
    state: State<'_, AppState>,
    tabs: Vec<OpenTab>,
) -> Result<Vec<OpenTab>, AppError> {
    let accounts = state.store.load_accounts().await;
    let (kept, dropped) = session::sanitize_open_tabs(tabs, &accounts);
    if dropped > 0 {
        tracing::warn!(dropped, "sesja: pominięto zakładki bez konta albo serwera");
    }
    state.store.save_session(&kept).await?;
    Ok(kept)
}

/// Czy diagnostyka zamrożenia widoku logu jest włączona (opcjonalna,
/// `MCC_GUI_DIAG=1` - patrz `src-tauri/src/diag.rs`).
///
/// Frontend pyta o to raz przy starcie: przy wyłączonej diagnostyce nie zakłada
/// żadnej pętli ani nie woła `diag_report`.
#[tauri::command]
pub fn diag_enabled() -> bool {
    crate::diag::enabled()
}

/// Zapisuje raport diagnostyczny do `diag.log` w katalogu danych aplikacji
/// (tylko przy `MCC_GUI_DIAG=1` - patrz `src-tauri/src/diag.rs`).
///
/// Nie zwracamy błędu do UI: diagnostyka jest narzędziem deweloperskim i nie
/// może pokazywać użytkownikowi toastów. Niepowodzenie ląduje w `tracing`.
#[tauri::command]
pub fn diag_report(app: tauri::AppHandle, report: Value) {
    if !crate::diag::enabled() {
        return;
    }

    let data_dir = match app.path().app_data_dir() {
        Ok(dir) => dir,
        Err(error) => {
            tracing::warn!(%error, "diagnostyka: brak katalogu danych aplikacji");
            return;
        }
    };

    // Stan okna jest tu najważniejszy: pokazuje, czy zamrożenie zbiegło się
    // z minimalizacją albo utratą widoczności okna.
    let window = app.get_webview_window("main");
    let window_state = json!({
        "minimized": window.as_ref().and_then(|w| w.is_minimized().ok()).unwrap_or(false),
        "focused": window.as_ref().and_then(|w| w.is_focused().ok()).unwrap_or(false),
        "visible": window.as_ref().and_then(|w| w.is_visible().ok()).unwrap_or(false),
    });

    let entry = json!({
        "t": crate::diag::now_ms(),
        "rust": crate::diag::counters(),
        "window": window_state,
        "ui": report,
    });

    if let Err(error) = crate::diag::append(&data_dir, &entry) {
        tracing::warn!(%error, "diagnostyka: nie udało się zapisać raportu");
    }
}

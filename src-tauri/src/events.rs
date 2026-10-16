//! Zdarzenia wysyłane do frontendu (kontrakt w `src/api/contract.ts`) oraz abstrakcja
//! `EventSink`, dzięki której `process_manager` nie zależy od Tauri i da się
//! testować bez uruchamiania aplikacji.

use serde::Serialize;
use serde_json::Value;

use crate::account::AccountStatus;

/// Strumień, z którego pochodzi linia.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LogStream {
    Stdout,
    Stderr,
}

/// Payload zdarzenia `mcc://log`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogPayload {
    pub account_id: String,
    pub stream: LogStream,
    /// Linia bez terminatora (`\r\n` / `\n`), poza tym niezmieniona.
    pub line: String,
    /// Czas odebrania linii, ms od epoki Unix.
    pub ts: i64,
}

/// Payload zdarzenia `mcc://status`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusPayload {
    pub account_id: String,
    pub status: AccountStatus,
}

/// Payload zdarzenia `app://error`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorPayload {
    pub code: String,
    pub params: Value,
}

/// Payload zdarzenia `mcc://device-code` - prompt logowania Microsoft.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceCodePayload {
    pub account_id: String,
    /// Adres do otwarcia (tylko `https://`, walidowany w Rustcie).
    pub url: String,
    /// Kod, który użytkownik przepisuje w przeglądarce.
    pub code: String,
}

/// Payload zdarzenia `mcc://joined` - klient połączył się z serwerem.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JoinedPayload {
    pub account_id: String,
}

/// Odbiorca zdarzeń. Implementacja w aplikacji wysyła je przez Tauri, a w
/// testach zbiera do wektora.
pub trait EventSink: Send + Sync + 'static {
    fn emit_log(&self, account_id: &str, stream: LogStream, line: String, ts: i64);
    fn emit_status(&self, account_id: &str, status: AccountStatus);
    fn emit_error(&self, code: &str, params: Value);
    /// Prompt logowania Microsoft (Etap 3).
    fn emit_device_code(&self, account_id: &str, url: String, code: String);
    /// Klient połączył się z serwerem (Etap 3).
    fn emit_joined(&self, account_id: &str);
}

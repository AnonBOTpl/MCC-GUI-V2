//! Stabilne kody błędów przekazywane do frontendu.
//!
//! Rust nie wysyła do UI tekstów dla człowieka - tylko kod i parametry, które
//! frontend tłumaczy. Dlatego błędy systemowe
//! przekazujemy jako liczbę (`osError`), a opis zostaje w logach `tracing`.

use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppError {
    pub code: String,
    pub params: Value,
}

impl AppError {
    /// Błąd bez parametrów.
    pub fn new(code: &str) -> Self {
        Self {
            code: code.to_string(),
            params: Value::Object(Default::default()),
        }
    }

    /// Błąd z parametrami (np. `limit` albo `osError`).
    pub fn with_params(code: &str, params: Value) -> Self {
        Self {
            code: code.to_string(),
            params,
        }
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.code, self.params)
    }
}

impl std::error::Error for AppError {}

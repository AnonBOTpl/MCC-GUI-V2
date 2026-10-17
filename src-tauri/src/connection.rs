//! Połączenie = para (konto, serwer). Jedno połączenie = jeden proces MCC.
//!
//! Model „konto + lista serwerów": konto ma **jeden katalog** (wspólną sesję MCC,
//! czyli jedno logowanie Microsoft na wszystkie swoje serwery), a każde
//! połączenie dostaje **własny plik ustawień** (`settings-<serwer>.ini`) i własny
//! proces uruchamiany z katalogu konta.
//!
//! Dlaczego tak: MCC czyta sesję z katalogu roboczego procesu (sprawdzone
//! w źródle MCC i na prawdziwym pliku), więc wspólny katalog = wspólna sesja.

use serde::{Deserialize, Serialize};

/// Połączenie konta z serwerem.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Connection {
    /// Klucz połączenia; ten sam serwer na dwóch kontach to dwa różne klucze.
    pub id: String,
    pub account_id: String,
    pub server: String,
}

/// Buduje klucz połączenia. **Ta sama reguła jest w frontendzie**
/// (`src/api/contract.ts` → `connectionId`) - zmiana tutaj wymaga zmiany tam.
pub fn connection_id(account_id: &str, server: &str) -> String {
    format!("{account_id}::{server}")
}

/// Rozbija identyfikator połączenia na `(konto, serwer)`.
///
/// Dzielimy tylko na **pierwszym** `::`: adres serwera zawiera dwukropki
/// (`host:25565`), więc podział „na ostatnim” byłby błędny. Używane m.in. przez
/// logi czatu, które z identyfikatora robią nazwy katalogów.
pub fn split_connection_id(id: &str) -> Option<(&str, &str)> {
    id.split_once("::")
}

impl Connection {
    pub fn new(account_id: &str, server: &str) -> Self {
        Self {
            id: connection_id(account_id, server),
            account_id: account_id.to_string(),
            server: server.to_string(),
        }
    }
}

/// Maksymalna długość adresu serwera.
pub const MAX_SERVER_LEN: usize = 200;

/// Nazwa pliku ustawień połączenia. Musi być bezpieczna dla systemu plików
/// (adres może mieć `:` i `/`) i **stabilna** dla danego adresu, żeby restart
/// aplikacji trafiał w ten sam plik.
pub fn settings_file_name(server: &str) -> String {
    let mut slug = String::new();
    for ch in server.chars() {
        if ch.is_ascii_alphanumeric() || ch == '.' || ch == '-' {
            slug.push(ch);
        } else {
            slug.push('_');
        }
        if slug.len() >= 40 {
            break;
        }
    }
    format!("settings-{slug}-{:08x}.ini", fnv1a(server))
}

/// Prosty FNV-1a (32 bit) do rozróżnienia adresów o tym samym „slug".
fn fnv1a(value: &str) -> u32 {
    let mut hash: u32 = 0x811c_9dc5;
    for byte in value.as_bytes() {
        hash ^= u32::from(*byte);
        hash = hash.wrapping_mul(0x0100_0193);
    }
    hash
}

/// Sprawdza i normalizuje adres serwera.
///
/// Znaki sterujące (`\n`, `\r`, `\t`) są odrzucane: adres trafia do literału
/// TOML w `settings-*.ini`, więc taki znak rozjeżdża cały plik ustawień (R-09).
pub fn validate_server(server: &str) -> Result<String, crate::error::AppError> {
    let trimmed = server.trim();
    if trimmed.is_empty() || trimmed.len() > MAX_SERVER_LEN || trimmed.chars().any(char::is_control)
    {
        return Err(crate::error::AppError::new("server.invalid"));
    }
    Ok(trimmed.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connection_id_separates_accounts_and_servers() {
        assert_eq!(connection_id("a", "host:1"), "a::host:1");
        assert_ne!(connection_id("a", "host:1"), connection_id("b", "host:1"));
        assert_ne!(connection_id("a", "host:1"), connection_id("a", "host:2"));
    }

    #[test]
    fn split_connection_id_keeps_server_ports() {
        assert_eq!(
            split_connection_id("acc-1::play.example.com:25565"),
            Some(("acc-1", "play.example.com:25565"))
        );
        assert_eq!(split_connection_id("acc-1::host"), Some(("acc-1", "host")));
        assert_eq!(split_connection_id("acc-1"), None);
    }

    #[test]
    fn settings_file_name_is_safe_and_stable() {
        let name = settings_file_name("play.example.com:25565");
        assert!(name.starts_with("settings-play.example.com_25565-"));
        assert!(name.ends_with(".ini"));
        assert!(
            !name.contains(':') && !name.contains('/') && !name.contains('\\'),
            "nazwa pliku nie może zawierać znaków specjalnych: {name}"
        );
        assert_eq!(name, settings_file_name("play.example.com:25565"));
    }

    #[test]
    fn settings_file_name_differs_for_different_servers() {
        assert_ne!(
            settings_file_name("host:1"),
            settings_file_name("host:2"),
            "różne serwery muszą mieć różne pliki ustawień"
        );
    }

    #[test]
    fn validate_server_rejects_control_chars() {
        for address in ["host\n:1", "ho\rst", "a\tb"] {
            assert_eq!(
                validate_server(address).unwrap_err().code,
                "server.invalid",
                "adres {address:?}"
            );
        }
    }

    #[test]
    fn validate_server_trims_and_rejects_empty_or_long() {
        assert_eq!(
            validate_server("  host:25565  ").expect("poprawny adres"),
            "host:25565"
        );
        assert_eq!(validate_server("   ").unwrap_err().code, "server.invalid");
        assert_eq!(
            validate_server(&"a".repeat(MAX_SERVER_LEN + 1))
                .unwrap_err()
                .code,
            "server.invalid"
        );
    }
}

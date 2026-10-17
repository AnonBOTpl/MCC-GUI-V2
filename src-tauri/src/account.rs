//! Model konta i jego statusu (kontrakt z frontendem).
//!
//! Konto to **tożsamość logowania** (jeden login = jeden katalog = jedna sesja
//! MCC) i lista serwerów. Każda para (konto, serwer) to osobne połączenie i
//! osobny proces - patrz `connection.rs`.

use serde::{Deserialize, Serialize};

use crate::error::AppError;

/// Typ konta. Microsoft nie używa hasła (device code), offline też nie (hasło `-`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccountType {
    Microsoft,
    Offline,
}

/// Status konta widoczny w UI. Serializowany jako `snake_case` (kontrakt).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccountStatus {
    Stopped,
    Starting,
    AwaitingLogin,
    Running,
    Stopping,
    Crashed,
}

/// Serwer przypisany do konta: adres i opcjonalnie **wymuszona wersja Minecrafta**.
///
/// Wersja bywa konieczna: gdy serwer nie odpowiada na ping statusu, MCC pyta
/// w konsoli „Server version:" i czeka na wpis. Ustawiona wersja trafia do
/// `MinecraftVersion` w pliku ustawień połączenia i MCC nie pyta wcale.
/// `None` oznacza `auto`, czyli normalne wykrywanie przez MCC.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerEntry {
    pub address: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

impl ServerEntry {
    /// Serwer bez wymuszonej wersji (`auto`).
    pub fn auto(address: &str) -> Self {
        Self {
            address: address.to_string(),
            version: None,
        }
    }
}

/// Deserializacja znosi **oba** kształty: zwykły łańcuch (starsze pliki
/// `accounts.json`, gdzie serwer to była sama nazwa) oraz obiekt
/// `{ address, version }`. Dzięki temu aktualizacja aplikacji nie psuje zapisanych kont.
impl<'de> Deserialize<'de> for ServerEntry {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Repr {
            Plain(String),
            Full {
                address: String,
                #[serde(default)]
                version: Option<String>,
            },
        }

        Ok(match Repr::deserialize(deserializer)? {
            Repr::Plain(address) => ServerEntry {
                address,
                version: None,
            },
            Repr::Full { address, version } => ServerEntry { address, version },
        })
    }
}

/// Konto użytkownika. Pole `type` w JSON-ie to `account_type` w Rust.
/// Nie ma tu hasła i nigdy nie będzie - hasła nie przechowujemy w ogóle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub id: String,
    pub label: String,
    #[serde(rename = "type")]
    pub account_type: AccountType,
    pub login: String,
    /// Zapisane serwery tego konta. Ten sam login może być połączony z wieloma
    /// serwerami naraz - każde połączenie to osobny proces MCC.
    #[serde(default)]
    pub servers: Vec<ServerEntry>,
    #[serde(default)]
    pub tab_order: i64,
}

/// Maksymalna długość tekstowych pól konta w pliku `accounts.json`.
pub const MAX_ACCOUNT_TEXT_LEN: usize = 200;

/// Maksymalna długość identyfikatora konta.
pub const MAX_ACCOUNT_ID_LEN: usize = 64;

/// Maksymalna liczba serwerów przypisanych do jednego konta.
pub const MAX_SERVERS_PER_ACCOUNT: usize = 100;

/// Maksymalna długość wymuszonej wersji Minecrafta (np. `1.21.4`).
pub const MAX_VERSION_LEN: usize = 32;

/// Czy identyfikator nadaje się na nazwę katalogu konta.
///
/// `accounts_root.join(id)` jest ścieżką, więc identyfikator nie może zawierać
/// separatorów, dwukropka ani kropek (`..` wyprowadziłoby katalog poza
/// `accounts/`). Dozwolone są znaki, które generuje UI (`acc-<base36>-<n>`)
/// i starsze zapisy: litery, cyfry, `-` i `_`. Dzięki temu istniejące
/// `accounts.json` przechodzą walidację bez zmian.
pub fn is_valid_account_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= MAX_ACCOUNT_ID_LEN
        && id
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
}

/// Sprawdza i normalizuje listę kont przed zapisem na dysk (kontrakt komend Tauri).
///
/// Walidujemy **każde** wejście do komendy Tauri: identyfikator
/// musi być niepusty i unikalny, login nie może być pusty, a adresy serwerów
/// muszą być poprawne. Lista serwerów może być pusta (konto bez serwerów), ale
/// duplikaty są usuwane (bez rozróżniania wielkości liter - nazwy hostów).
pub fn validate_accounts(accounts: Vec<Account>) -> Result<Vec<Account>, AppError> {
    let mut normalized = Vec::with_capacity(accounts.len());
    let mut seen_ids = std::collections::HashSet::with_capacity(accounts.len());

    for account in accounts {
        let account = validate_account(account)?;
        if !seen_ids.insert(account.id.clone()) {
            return Err(AppError::new("account.duplicateId"));
        }
        normalized.push(account);
    }

    Ok(normalized)
}

/// Sprawdza i normalizuje **jedno** konto (bez pilnowania unikalności w liście).
///
/// Wydzielone z [`validate_accounts`], bo tej samej reguły używa wczytywanie
/// `accounts.json` wpis po wpisie (R-16): wpis, który jej nie przechodzi, jest
/// pomijany razem z ostrzeżeniem, a nie wycina całej listy kont.
pub fn validate_account(account: Account) -> Result<Account, AppError> {
    let id = account.id.trim().to_string();
    if !is_valid_account_id(&id) {
        return Err(AppError::new("account.invalidId"));
    }

    let login = account.login.trim().to_string();
    if login.is_empty() || login.len() > MAX_ACCOUNT_TEXT_LEN {
        return Err(AppError::new("account.loginRequired"));
    }
    // Znak sterujący (np. `\n`) rozjeżdża generowany TOML `settings-*.ini`,
    // a login trafia do niego jako literał w cudzysłowach.
    if login.chars().any(char::is_control) {
        return Err(AppError::new("account.loginControlChars"));
    }

    if account.servers.len() > MAX_SERVERS_PER_ACCOUNT {
        return Err(AppError::new("account.tooManyServers"));
    }

    let mut servers: Vec<ServerEntry> = Vec::with_capacity(account.servers.len());
    let mut seen_servers = std::collections::HashSet::new();
    for entry in account.servers {
        let address = crate::connection::validate_server(&entry.address)?;
        let version = normalize_version(entry.version.as_deref())?;
        if seen_servers.insert(address.to_lowercase()) {
            servers.push(ServerEntry { address, version });
        }
    }

    let label = account.label.trim();
    let label = if label.is_empty() {
        login.clone()
    } else {
        label.chars().take(MAX_ACCOUNT_TEXT_LEN).collect()
    };

    Ok(Account {
        id,
        label,
        account_type: account.account_type,
        login,
        servers,
        tab_order: account.tab_order.max(0),
    })
}

/// Normalizuje wymuszoną wersję Minecrafta: puste znaczy `auto` (`None`),
/// a zbyt długa albo ze znakami sterującymi jest odrzucana.
pub fn normalize_version(raw: Option<&str>) -> Result<Option<String>, AppError> {
    let Some(raw) = raw else {
        return Ok(None);
    };
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    if trimmed.len() > MAX_VERSION_LEN || trimmed.chars().any(char::is_control) {
        return Err(AppError::new("server.invalidVersion"));
    }
    Ok(Some(trimmed.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn account(id: &str, login: &str, servers: &[&str]) -> Account {
        Account {
            id: id.into(),
            label: String::new(),
            account_type: AccountType::Microsoft,
            login: login.into(),
            servers: servers.iter().map(|s| ServerEntry::auto(s)).collect(),
            tab_order: 0,
        }
    }

    #[test]
    fn validate_trims_and_fills_label_from_login() {
        let normalized = validate_accounts(vec![account(" acc-1 ", "  a@b.c  ", &[" host:123 "])])
            .expect("konto powinno przejść walidację");
        assert_eq!(normalized.len(), 1);
        assert_eq!(normalized[0].id, "acc-1");
        assert_eq!(normalized[0].login, "a@b.c");
        assert_eq!(normalized[0].servers, vec![ServerEntry::auto("host:123")]);
        assert_eq!(normalized[0].label, "a@b.c");
    }

    #[test]
    fn validate_allows_account_without_servers() {
        let normalized =
            validate_accounts(vec![account("a", "l", &[])]).expect("konto bez serwerów jest ok");
        assert!(normalized[0].servers.is_empty());
    }

    #[test]
    fn validate_dedupes_servers_case_insensitively() {
        let normalized = validate_accounts(vec![account(
            "a",
            "l",
            &["Host:25565", "host:25565", "other:1"],
        )])
        .expect("walidacja");
        assert_eq!(
            normalized[0].servers,
            vec![
                ServerEntry::auto("Host:25565"),
                ServerEntry::auto("other:1")
            ],
            "duplikat bez rozróżniania wielkości liter ma zostać usunięty"
        );
    }

    #[test]
    fn validate_rejects_bad_server_in_list() {
        let error =
            validate_accounts(vec![account("a", "l", &["ok:1", "  "])]).expect_err("pusty serwer");
        assert_eq!(error.code, "server.invalid");
    }

    #[test]
    fn validate_rejects_duplicate_ids() {
        let error = validate_accounts(vec![account("same", "a", &[]), account("same", "b", &[])])
            .expect_err("duplikat identyfikatora musi zostać odrzucony");
        assert_eq!(error.code, "account.duplicateId");
    }

    #[test]
    fn validate_rejects_empty_login_and_id() {
        let login_error =
            validate_accounts(vec![account("a", "   ", &[])]).expect_err("pusty login");
        assert_eq!(login_error.code, "account.loginRequired");

        let id_error = validate_accounts(vec![account("  ", "l", &[])]).expect_err("pusty id");
        assert_eq!(id_error.code, "account.invalidId");
    }

    /// R-01: identyfikator trafia do `accounts_root.join(id)`, więc separator
    /// ścieżki albo `..` wyprowadza katalog poza `accounts/`.
    #[test]
    fn validate_rejects_path_separators_in_id() {
        for id in ["../evil", "a/../b", "a/b", "a\\b", ".", "..", "C:", "a:b"] {
            let error = validate_accounts(vec![account(id, "l", &[])])
                .expect_err("taki identyfikator musi zostać odrzucony");
            assert_eq!(error.code, "account.invalidId", "id {id:?}");
        }
    }

    /// Identyfikatory, które aplikacja naprawdę zapisuje (UI generuje
    /// `acc-<base36>-<n>`), muszą dalej przechodzić walidację - inaczej
    /// istniejące `accounts.json` przestałyby się wczytywać.
    #[test]
    fn validate_accepts_ids_used_by_the_app() {
        for id in ["acc-1", "acc-m6xk1p-2", "e2e-tabs", "A_b-9"] {
            validate_accounts(vec![account(id, "l", &[])])
                .unwrap_or_else(|error| panic!("id {id:?} powinien przejść: {}", error.code));
        }
    }

    /// R-09: znaki sterujące w loginie rozjeżdżają generowany TOML.
    #[test]
    fn validate_rejects_control_chars_in_login_and_server() {
        let login = validate_accounts(vec![account("a", "a\nb", &[])])
            .expect_err("login ze znakiem sterującym musi zostać odrzucony");
        assert_eq!(login.code, "account.loginControlChars");

        let server = validate_accounts(vec![account("a", "l", &["host\n:1"])])
            .expect_err("adres ze znakiem sterującym musi zostać odrzucony");
        assert_eq!(server.code, "server.invalid");
    }

    #[test]
    fn validate_rejects_too_many_servers_and_clamps_tab_order() {
        let many: Vec<ServerEntry> = (0..MAX_SERVERS_PER_ACCOUNT + 1)
            .map(|i| ServerEntry::auto(&format!("host{i}:1")))
            .collect();
        let mut over = account("a", "l", &[]);
        over.servers = many;
        assert_eq!(
            validate_accounts(vec![over]).unwrap_err().code,
            "account.tooManyServers"
        );

        let mut negative = account("a", "l", &[]);
        negative.tab_order = -5;
        let normalized = validate_accounts(vec![negative]).expect("walidacja");
        assert_eq!(normalized[0].tab_order, 0);
    }

    #[test]
    fn account_deserializes_from_contract_shape() {
        let json = r#"{
            "id": "acc-1",
            "label": "Main",
            "type": "microsoft",
            "login": "user@example.com",
            "servers": ["play.example.com:25565", "other.net"],
            "tabOrder": 0
        }"#;
        let account: Account = serde_json::from_str(json).expect("konto powinno się sparsować");
        assert_eq!(account.id, "acc-1");
        assert_eq!(account.account_type, AccountType::Microsoft);
        assert_eq!(account.servers.len(), 2);
        // Stary kształt (sam łańcuch) nadal działa i znaczy `auto`.
        assert_eq!(
            account.servers[0],
            ServerEntry::auto("play.example.com:25565")
        );
    }

    #[test]
    fn server_entry_deserializes_from_object_with_version() {
        let json = r#"{
            "id": "a", "label": "L", "type": "offline", "login": "Bot",
            "servers": [{ "address": "play.example.com", "version": "1.21" }]
        }"#;
        let account: Account = serde_json::from_str(json).expect("konto powinno się sparsować");
        assert_eq!(
            account.servers[0],
            ServerEntry {
                address: "play.example.com".into(),
                version: Some("1.21".into())
            }
        );
    }

    #[test]
    fn version_is_trimmed_and_blank_means_auto() {
        let normalized = validate_accounts(vec![Account {
            servers: vec![
                ServerEntry {
                    address: "a:1".into(),
                    version: Some("  1.21  ".into()),
                },
                ServerEntry {
                    address: "b:2".into(),
                    version: Some("   ".into()),
                },
            ],
            ..account("a", "l", &[])
        }])
        .expect("walidacja");
        assert_eq!(normalized[0].servers[0].version.as_deref(), Some("1.21"));
        assert_eq!(normalized[0].servers[1].version, None);
    }

    #[test]
    fn version_rejects_control_chars_and_overlong_values() {
        let bad = validate_accounts(vec![Account {
            servers: vec![ServerEntry {
                address: "a:1".into(),
                version: Some("1\n2".into()),
            }],
            ..account("a", "l", &[])
        }]);
        assert_eq!(bad.unwrap_err().code, "server.invalidVersion");

        let long = validate_accounts(vec![Account {
            servers: vec![ServerEntry {
                address: "a:1".into(),
                version: Some("1".repeat(MAX_VERSION_LEN + 1)),
            }],
            ..account("a", "l", &[])
        }]);
        assert_eq!(long.unwrap_err().code, "server.invalidVersion");
    }

    #[test]
    fn servers_and_tab_order_default_when_missing() {
        let json = r#"{
            "id": "a", "label": "L", "type": "offline", "login": "Bot"
        }"#;
        let account: Account = serde_json::from_str(json).expect("konto powinno się sparsować");
        assert!(account.servers.is_empty());
        assert_eq!(account.tab_order, 0);
    }

    #[test]
    fn status_serializes_as_snake_case() {
        let value = serde_json::to_string(&AccountStatus::AwaitingLogin).expect("serializacja");
        assert_eq!(value, "\"awaiting_login\"");
    }
}

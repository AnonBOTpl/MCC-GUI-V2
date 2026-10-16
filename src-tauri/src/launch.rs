//! Budowanie argumentów uruchomienia MCC i per-połączenie `settings-*.ini`.
//!
//! Uruchamiamy `MinecraftClient.exe settings-<serwer>.ini BasicIO` z katalogiem
//! roboczym ustawionym na katalog **konta**. Pierwszy argument to dowolny
//! istniejący plik `.ini` (dowód: `Program.cs:150` w MCC - `File.Exists(args[0])
//! && GetExtension(args[0]) == ".ini"`), więc:
//!
//! - każdy serwer konta ma własny plik ustawień (własne `Server`),
//! - wszystkie procesy konta dzielą jeden katalog roboczy, czyli **jedną sesję
//!   MCC** - jedno logowanie Microsoft obsługuje wszystkie serwery konta.
//!
//! Nie podajemy hasła ani loginu w linii poleceń (inaczej niż GUI dołączane do
//! MCC) - wszystko jest w pliku `.ini`.

use crate::account::{Account, AccountType, ServerEntry};

/// Argument wymuszający czysty strumień linii po potoku (z kodami `§`).
pub const BASIC_IO_ARG: &str = "BasicIO";

/// Argumenty przekazywane do `MinecraftClient.exe`: plik ustawień i `BasicIO`.
pub fn build_args(settings_file_name: &str) -> Vec<String> {
    vec![settings_file_name.to_string(), BASIC_IO_ARG.to_string()]
}

/// Escapuje tekst do bezpiecznego literału TOML (cudzysłowy i ukośniki).
fn toml_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Generuje minimalny plik ustawień dla połączenia (konto + konkretny serwer).
pub fn build_settings_ini(account: &Account, server: &ServerEntry) -> String {
    let (account_type, password) = match account.account_type {
        AccountType::Microsoft => ("microsoft", String::new()),
        AccountType::Offline => ("mojang", "-".to_string()),
    };

    // `auto` = MCC sam wykryje wersję przez ping. Gdy serwer nie odpowiada na
    // ping, MCC pyta o wersję w konsoli - dlatego użytkownik może ją wymusić
    // per serwer (`ServerEntry::version`).
    let version = server.version.as_deref().unwrap_or("auto");

    // Komentarze po polsku, bo to plik generowany przez tę aplikację.
    // `Language = "en_us"` to język gry (nazwy przedmiotów); własne komunikaty MCC
    // wymuszamy na angielskie przez `LoadMccTranslation = false` (patrz MCC_NOTES §1).
    format!(
        "# Wygenerowane przez MCC GUI V2. Zmiany zostaną nadpisane.\n\
         [Main.General]\n\
         Account = {{ Login = \"{login}\", Password = \"{password}\" }}\n\
         Server = {{ Host = \"{server}\" }}\n\
         AccountType = \"{account_type}\"\n\
         Method = \"mcc\"\n\
         \n\
         [Main.Advanced]\n\
         Language = \"en_us\"\n\
         LoadMccTranslation = false\n\
         EnableSentry = false\n\
         InternalCmdChar = \"slash\"\n\
         MinecraftVersion = \"{version}\"\n",
        login = toml_escape(&account.login),
        password = password,
        server = toml_escape(&server.address),
        version = toml_escape(version),
        account_type = account_type,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connection::settings_file_name;

    fn account(account_type: AccountType) -> Account {
        Account {
            id: "acc-1".into(),
            label: "Test".into(),
            account_type,
            login: "user@example.com".into(),
            servers: vec![ServerEntry::auto("example.com:25565")],
            tab_order: 0,
        }
    }

    fn server(address: &str) -> ServerEntry {
        ServerEntry::auto(address)
    }

    fn server_with_version(address: &str, version: &str) -> ServerEntry {
        ServerEntry {
            address: address.into(),
            version: Some(version.into()),
        }
    }

    #[test]
    fn args_are_settings_file_then_basic_io() {
        let name = settings_file_name("example.com:25565");
        assert_eq!(build_args(&name), vec![name.clone(), "BasicIO".to_string()]);
    }

    #[test]
    fn settings_use_the_server_of_the_connection() {
        let ini = build_settings_ini(&account(AccountType::Microsoft), &server("other.net:1234"));
        assert!(ini.contains("Server = { Host = \"other.net:1234\" }"));
    }

    #[test]
    fn version_defaults_to_auto_and_can_be_forced() {
        let auto = build_settings_ini(&account(AccountType::Microsoft), &server("s:1"));
        assert!(auto.contains("MinecraftVersion = \"auto\""));

        let forced = build_settings_ini(
            &account(AccountType::Microsoft),
            &server_with_version("play.example.com", "1.21"),
        );
        assert!(forced.contains("MinecraftVersion = \"1.21\""));
    }

    #[test]
    fn microsoft_uses_microsoft_and_empty_password() {
        let ini = build_settings_ini(&account(AccountType::Microsoft), &server("s:1"));
        assert!(ini.contains("Account = { Login = \"user@example.com\", Password = \"\" }"));
        assert!(ini.contains("AccountType = \"microsoft\""));
        assert!(ini.contains("Method = \"mcc\""));
    }

    #[test]
    fn offline_uses_mojang_and_dash_password() {
        let ini = build_settings_ini(&account(AccountType::Offline), &server("s:1"));
        assert!(ini.contains("Account = { Login = \"user@example.com\", Password = \"-\" }"));
        assert!(ini.contains("AccountType = \"mojang\""));
    }

    #[test]
    fn forces_english_mcc_messages_and_no_sentry() {
        let ini = build_settings_ini(&account(AccountType::Microsoft), &server("s:1"));
        assert!(ini.contains("LoadMccTranslation = false"));
        assert!(ini.contains("EnableSentry = false"));
        assert!(ini.contains("Language = \"en_us\""));
    }

    #[test]
    fn escapes_quotes_in_login_and_server() {
        let mut acc = account(AccountType::Offline);
        acc.login = "we\"ird".into();
        let ini = build_settings_ini(&acc, &server("a\"b"));
        assert!(ini.contains("Login = \"we\\\"ird\""));
        assert!(ini.contains("Host = \"a\\\"b\""));
    }
}

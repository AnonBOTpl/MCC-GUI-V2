//! Sesja: które **zakładki** (połączenia) były otwarte przy zamknięciu aplikacji.
//!
//! Plik `session.json` trzyma same pary (konto, serwer) - nic więcej. Trzymamy tę
//! informację osobno od `accounts.json`, bo otwarte okno jest stanem **sesji**, a
//! nie cechą konta (wzorzec plik-na-sprawę jak `presets.json`).
//!
//! Przywracanie zakładek **nie uruchamia niczego**: procesy MCC startują tylko z
//! wyraźnej decyzji użytkownika. Dlatego ten moduł operuje wyłącznie na danych - nie zna
//! `ProcessManager` i nie da się z niego przypadkiem wystartować połączenia.

use serde::{Deserialize, Serialize};

use crate::account::Account;

/// Zakładka połączenia zapisana na dysk: konto i jego serwer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenTab {
    pub account_id: String,
    pub server: String,
}

/// Górny limit zakładek w pliku sesji.
///
/// Realnie zakładek nie może być więcej niż kont razy serwery (plan mówi o 100
/// serwerach na konto), ale plik z dysku jest danymi wejściowymi jak każde inne,
/// więc ograniczamy go, zanim cokolwiek zbudujemy w UI.
pub const MAX_OPEN_TABS: usize = 200;

/// Odcedza zakładki z pliku sesji względem aktualnych kont.
///
/// Zostają tylko wpisy wskazujące na **istniejące** konto i jego istniejący
/// serwer, bez duplikatów i w kolejności zapisu (to kolejność zakładek). Konto
/// mogło zostać usunięte, a serwer skasowany - wtedy wpis po prostu znika, bo
/// inaczej stary plik blokowałby start aplikacji.
///
/// Adresy porównujemy bez rozróżniania wielkości liter (nazwy hostów), a do
/// wyniku trafia **kanoniczna postać z konta**, nie surowy tekst z pliku.
/// Zwracamy też liczbę odrzuconych wpisów - wołający loguje ją przez `tracing`.
pub fn sanitize_open_tabs(tabs: Vec<OpenTab>, accounts: &[Account]) -> (Vec<OpenTab>, usize) {
    let mut kept: Vec<OpenTab> = Vec::new();
    let mut dropped = 0usize;

    for tab in tabs {
        if kept.len() >= MAX_OPEN_TABS {
            dropped += 1;
            continue;
        }

        let Some(account) = accounts.iter().find(|item| item.id == tab.account_id) else {
            dropped += 1;
            continue;
        };

        let wanted = tab.server.trim().to_lowercase();
        let Some(server) = account
            .servers
            .iter()
            .find(|entry| entry.address.to_lowercase() == wanted)
        else {
            dropped += 1;
            continue;
        };

        let candidate = OpenTab {
            account_id: account.id.clone(),
            server: server.address.clone(),
        };
        if kept.contains(&candidate) {
            dropped += 1;
            continue;
        }

        kept.push(candidate);
    }

    (kept, dropped)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::account::{AccountType, ServerEntry};

    fn account(id: &str, servers: &[&str]) -> Account {
        Account {
            id: id.into(),
            label: id.into(),
            account_type: AccountType::Microsoft,
            login: format!("{id}@example.com"),
            servers: servers.iter().map(|s| ServerEntry::auto(s)).collect(),
            tab_order: 0,
        }
    }

    fn tab(account_id: &str, server: &str) -> OpenTab {
        OpenTab {
            account_id: account_id.into(),
            server: server.into(),
        }
    }

    #[test]
    fn keeps_tabs_pointing_at_existing_account_and_server() {
        let accounts = vec![account("a", &["one:1", "two:2"]), account("b", &["one:1"])];
        let (kept, dropped) = sanitize_open_tabs(
            vec![tab("b", "one:1"), tab("a", "two:2"), tab("a", "one:1")],
            &accounts,
        );

        assert_eq!(dropped, 0);
        assert_eq!(
            kept,
            vec![tab("b", "one:1"), tab("a", "two:2"), tab("a", "one:1")],
            "kolejność zapisu to kolejność zakładek"
        );
    }

    #[test]
    fn drops_unknown_account_and_unknown_server() {
        let accounts = vec![account("a", &["one:1"])];
        let (kept, dropped) = sanitize_open_tabs(
            vec![tab("a", "one:1"), tab("a", "gone:2"), tab("ghost", "one:1")],
            &accounts,
        );

        assert_eq!(dropped, 2);
        assert_eq!(kept, vec![tab("a", "one:1")]);
    }

    #[test]
    fn matches_server_case_insensitively_and_returns_canonical_form() {
        let accounts = vec![account("a", &["Play.Example.COM"])];
        let (kept, dropped) = sanitize_open_tabs(vec![tab("a", " play.example.com ")], &accounts);

        assert_eq!(dropped, 0);
        assert_eq!(
            kept,
            vec![tab("a", "Play.Example.COM")],
            "do wyniku trafia adres z konta, nie surowy tekst z pliku"
        );
    }

    #[test]
    fn removes_duplicates() {
        let accounts = vec![account("a", &["one:1"])];
        let (kept, dropped) = sanitize_open_tabs(
            vec![tab("a", "one:1"), tab("a", "ONE:1"), tab("a", "one:1")],
            &accounts,
        );

        assert_eq!(kept.len(), 1);
        assert_eq!(dropped, 2);
    }

    #[test]
    fn caps_the_number_of_tabs() {
        // Więcej zakładek niż limit, ale wszystkie poprawne: nadmiar wypada.
        let servers: Vec<String> = (0..MAX_OPEN_TABS + 5)
            .map(|i| format!("host{i}:1"))
            .collect();
        let addresses: Vec<&str> = servers.iter().map(String::as_str).collect();
        let accounts = vec![account("a", &addresses)];
        let tabs: Vec<OpenTab> = servers.iter().map(|s| tab("a", s)).collect();

        let (kept, dropped) = sanitize_open_tabs(tabs, &accounts);
        assert_eq!(kept.len(), MAX_OPEN_TABS);
        assert_eq!(dropped, 5);
    }

    #[test]
    fn empty_session_stays_empty() {
        let (kept, dropped) = sanitize_open_tabs(Vec::new(), &[account("a", &["one:1"])]);
        assert!(kept.is_empty());
        assert_eq!(dropped, 0);
    }

    #[test]
    fn session_deserializes_from_contract_shape() {
        let json = r#"[{ "accountId": "acc-1", "server": "play.example.com" }]"#;
        let tabs: Vec<OpenTab> = serde_json::from_str(json).expect("sesja powinna się sparsować");
        assert_eq!(tabs, vec![tab("acc-1", "play.example.com")]);
    }

    #[test]
    fn session_serializes_to_contract_shape() {
        let value =
            serde_json::to_string(&vec![tab("acc-1", "play.example.com")]).expect("serializacja");
        assert_eq!(
            value,
            r#"[{"accountId":"acc-1","server":"play.example.com"}]"#
        );
    }
}

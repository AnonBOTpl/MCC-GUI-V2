//! Reguła potwierdzania zamknięcia aplikacji.
//!
//! Zamknięcie okna przechodzi przez jedno zdarzenie Tauri (`CloseRequested`),
//! więc obsługujemy w nim **wszystkie** drogi zamknięcia: przycisk w naszym
//! pasku tytułu, Alt+F4 i zamknięcie z paska zadań.
//!
//! Decyzję „pytać czy nie” trzymamy jako czystą funkcję, żeby dała się
//! przetestować bez okna, procesów i pętli zdarzeń.

/// Czy przy zamykaniu okna zapytać użytkownika o rozłączenie połączeń?
///
/// Pytamy tylko wtedy, gdy spełnione są **wszystkie** warunki:
/// - użytkownik chce, żeby pytać (przełącznik w ustawieniach),
/// - nie potwierdził już tego zamknięcia (inaczej drugie `CloseRequested`
///   zapytałoby w kółko),
/// - działa co najmniej jedno połączenie. Bez aktywnego połączenia nie ma czego
///   przerywać, więc pytanie byłoby tylko przeszkodą w zamknięciu okna.
pub fn should_confirm(
    confirm_setting: bool,
    already_confirmed: bool,
    active_connections: usize,
) -> bool {
    confirm_setting && !already_confirmed && active_connections > 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asks_when_setting_is_on_and_a_connection_is_active() {
        assert!(should_confirm(true, false, 1));
        assert!(should_confirm(true, false, 7));
    }

    #[test]
    fn does_not_ask_without_active_connections() {
        assert!(!should_confirm(true, false, 0));
    }

    #[test]
    fn does_not_ask_when_the_setting_is_off() {
        assert!(!should_confirm(false, false, 3));
    }

    #[test]
    fn does_not_ask_a_second_time_after_confirmation() {
        assert!(!should_confirm(true, true, 3));
        assert!(!should_confirm(false, true, 0));
    }
}

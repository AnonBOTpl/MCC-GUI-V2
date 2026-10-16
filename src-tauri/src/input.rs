//! Walidacja tekstu wysyłanego do `stdin` MCC.
//!
//! Zasady wysyłki: jedna linia na raz, bez wbudowanych znaków nowej linii
//! (żeby jeden wpis nie wstrzyknął dodatkowej komendy) i bez znaków sterujących.

use serde_json::json;

use crate::error::AppError;

/// Maksymalna długość wpisu użytkownika (zapobiega nadużyciom).
pub const MAX_INPUT_LEN: usize = 512;

/// Waliduje i zwraca tekst gotowy do wysłania (bez terminatora linii).
pub fn validate_input(raw: &str) -> Result<String, AppError> {
    if raw.trim().is_empty() {
        return Err(AppError::new("input.empty"));
    }
    if raw.chars().count() > MAX_INPUT_LEN {
        return Err(AppError::with_params(
            "input.tooLong",
            json!({ "limit": MAX_INPUT_LEN }),
        ));
    }
    // `is_control` obejmuje `\n`, `\r`, tabulatory i inne znaki sterujące.
    if raw.chars().any(char::is_control) {
        return Err(AppError::new("input.controlChars"));
    }
    Ok(raw.to_string())
}

/// Czy wpis zamyka MCC?
///
/// MCC obsługuje `/quit` i `/exit`, opcjonalnie z kodem wyjścia
/// (patrz `reference/.../Commands/Exit.cs`). Rozpoznajemy to, żeby zamknięcie
/// zainicjowane **przez użytkownika** trafiło do statusu `stopped`, a nie
/// `crashed`: proces kończy się wtedy sam, bez naszego `/quit`, więc bez tej
/// wiedzy wyglądałby jak awaria (i pokazywał fałszywy błąd w UI).
pub fn is_quit_command(raw: &str) -> bool {
    let trimmed = raw.trim().to_ascii_lowercase();
    let mut parts = trimmed.split_whitespace();
    let Some(command) = parts.next() else {
        return false;
    };
    if command != "/quit" && command != "/exit" {
        return false;
    }
    // Dozwolony jest tylko opcjonalny kod wyjścia i nic więcej.
    match parts.next() {
        None => true,
        Some(code) => code.parse::<i32>().is_ok() && parts.next().is_none(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_plain_text() {
        assert_eq!(validate_input("hello world").unwrap(), "hello world");
    }

    #[test]
    fn accepts_slash_command() {
        assert_eq!(validate_input("/reco").unwrap(), "/reco");
    }

    #[test]
    fn rejects_empty_and_whitespace() {
        assert_eq!(validate_input("").unwrap_err().code, "input.empty");
        assert_eq!(validate_input("   ").unwrap_err().code, "input.empty");
    }

    #[test]
    fn rejects_embedded_newlines() {
        assert_eq!(
            validate_input("hello\n/quit").unwrap_err().code,
            "input.controlChars"
        );
        assert_eq!(
            validate_input("a\r\nb").unwrap_err().code,
            "input.controlChars"
        );
        assert_eq!(
            validate_input("a\tb").unwrap_err().code,
            "input.controlChars"
        );
    }

    #[test]
    fn detects_quit_commands() {
        assert!(is_quit_command("/quit"));
        assert!(is_quit_command("  /QUIT  "));
        assert!(is_quit_command("/exit"));
        assert!(is_quit_command("/quit 0"));
        assert!(is_quit_command("/exit 3"));
    }

    #[test]
    fn does_not_treat_other_input_as_quit() {
        assert!(!is_quit_command("hello"));
        assert!(!is_quit_command("/reco"));
        assert!(!is_quit_command("/quit now"));
        assert!(!is_quit_command("/quit 0 extra"));
        assert!(!is_quit_command("say /quit"));
        assert!(!is_quit_command(""));
    }

    #[test]
    fn rejects_too_long() {
        let long = "x".repeat(MAX_INPUT_LEN + 1);
        assert_eq!(validate_input(&long).unwrap_err().code, "input.tooLong");
        let at_limit = "x".repeat(MAX_INPUT_LEN);
        assert!(validate_input(&at_limit).is_ok());
    }
}

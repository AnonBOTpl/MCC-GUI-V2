//! Tekst wkładany do schowka systemowego („Kopiuj” i „Kopiuj wszystko”
//! w widoku logu, Paczka 3).
//!
//! Zapis robimy po stronie Rusta (plugin schowka), tak samo jak kod logowania
//! (`copy_login_code`). Powody: `navigator.clipboard` w WebView2 pokazuje
//! natywne okno uprawnień, a komenda własna nie wymaga żadnego wpisu
//! w `capabilities/default.json` (uprawnienia dotyczą komend Tauri, nie naszych).
//!
//! Frontend wysyła linie rozdzielone `\n`; zamianę na końce linii Windows
//! robimy tutaj, bo takiego tekstu oczekuje schowek (`CF_UNICODETEXT`).

use serde_json::json;

use crate::error::AppError;

/// Maksymalna długość tekstu w znakach. Bufor logu to domyślnie 5000 linii, więc
/// limit leży daleko poza realnym użyciem („Kopiuj wszystko”) i służy tylko
/// ograniczeniu nadużyć.
pub const MAX_CLIPBOARD_LEN: usize = 8 * 1024 * 1024;

/// Waliduje tekst zaznaczonych wierszy i zwraca go gotowy do schowka.
pub fn clipboard_text(raw: &str) -> Result<String, AppError> {
    if raw.is_empty() {
        return Err(AppError::new("clipboard.writeFailed"));
    }
    // Sprowadzamy końce linii do `\n`, żeby ewentualne `\r` z wyjścia MCC nie
    // tworzyły linii pustych ani nie mieszały się z normalizacją poniżej.
    let normalized = raw.replace("\r\n", "\n").replace('\r', "\n");
    if normalized.chars().count() > MAX_CLIPBOARD_LEN {
        return Err(AppError::with_params(
            "clipboard.tooLong",
            json!({ "limit": MAX_CLIPBOARD_LEN }),
        ));
    }
    // Z wyjścia MCC dopuszczamy tylko tabulator i znak nowej linii; pozostałe
    // znaki sterujące (m.in. `\0` czy `\x1b`) nie mają prawa się tu znaleźć.
    if normalized
        .chars()
        .any(|c| c.is_control() && c != '\n' && c != '\t')
    {
        return Err(AppError::new("clipboard.writeFailed"));
    }
    Ok(normalized.replace('\n', "\r\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_plain_text() {
        assert_eq!(clipboard_text("hello").unwrap(), "hello");
    }

    #[test]
    fn uses_windows_line_endings() {
        assert_eq!(clipboard_text("a\nb").unwrap(), "a\r\nb");
        assert_eq!(clipboard_text("a\r\nb").unwrap(), "a\r\nb");
        assert_eq!(clipboard_text("a\rb").unwrap(), "a\r\nb");
    }

    #[test]
    fn keeps_tabs_and_polish_characters() {
        assert_eq!(
            clipboard_text("gracz\tzabił żubra").unwrap(),
            "gracz\tzabił żubra"
        );
    }

    #[test]
    fn rejects_empty_text() {
        assert_eq!(
            clipboard_text("").unwrap_err().code,
            "clipboard.writeFailed"
        );
    }

    #[test]
    fn rejects_control_characters() {
        assert_eq!(
            clipboard_text("a\u{0}b").unwrap_err().code,
            "clipboard.writeFailed"
        );
        assert_eq!(
            clipboard_text("a\u{1b}[31mb").unwrap_err().code,
            "clipboard.writeFailed"
        );
    }

    #[test]
    fn rejects_too_long_text() {
        let long = "x".repeat(MAX_CLIPBOARD_LEN + 1);
        assert_eq!(clipboard_text(&long).unwrap_err().code, "clipboard.tooLong");
    }
}

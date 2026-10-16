//! Wykrywanie promptu logowania Microsoft (device code) w surowym wyjściu MCC.
//!
//! Wzorce pochodzą z realnej próbki wyjścia MCC i z jego kodu źródłowego:
//!
//! ```text
//! Connecting to Microsoft...
//! To sign in, open https://www.microsoft.com/link in your browser and enter the code: §e7FRBLM2Y
//! Waiting for authentication to complete...
//! ```
//!
//! Odpowiada to szablonowi `mcc.device_code_prompt` =
//! `To sign in, open {0} in your browser and enter the code: §e{1}`. Kod jest
//! poprzedzony kodem koloru, więc najpierw zdejmujemy formatowanie.
//!
//! Moduł jest czysty (bez Tauri), więc ma testy jednostkowe.

/// Wykryty prompt logowania: adres do otwarcia i kod do przepisania.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceCodePrompt {
    pub url: String,
    pub code: String,
}

/// Fragment szablonu przed adresem.
const URL_MARKER: &str = "To sign in, open ";

/// Fragment szablonu między adresem a kodem.
const CODE_MARKER: &str = " in your browser and enter the code: ";

/// Skrócone markery promptu (bez otaczających spacji) - używane przy filtrowaniu
/// logów czatu, gdzie linia może być urwana albo mieć inny przedrostek.
const URL_HINT: &str = "To sign in, open";
const CODE_HINT: &str = "in your browser and enter the code";

/// Fragment komunikatu o udanym połączeniu (`mcc.joined`).
const JOINED_MARKER: &str = "Server was successfully joined.";

/// Maksymalna długość kodu, jaką uznajemy za rozsądną (realnie 8 znaków).
const MAX_CODE_LEN: usize = 32;

/// Zdejmuje kody `§x` / `§#rrggbb` z tekstu.
fn strip_section_codes(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let mut index = 0;
    while index < chars.len() {
        if chars[index] != '§' {
            out.push(chars[index]);
            index += 1;
            continue;
        }
        // `§#rrggbb` zajmuje 8 znaków, `§x` dwa.
        if chars.get(index + 1) == Some(&'#') && index + 7 < chars.len() {
            index += 8;
        } else {
            index += 2;
        }
    }
    out
}

/// Zdejmuje sekwencje ANSI (kolory i inne CSI).
fn strip_ansi(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let chars: Vec<char> = text.chars().collect();
    let mut index = 0;
    while index < chars.len() {
        if chars[index] == '\u{1b}' && chars.get(index + 1) == Some(&'[') {
            // Pomijamy `ESC [ ... ` aż do litery kończącej sekwencję.
            let mut cursor = index + 2;
            while cursor < chars.len() {
                let ch = chars[cursor];
                if ch.is_ascii_alphabetic() {
                    break;
                }
                cursor += 1;
            }
            index = cursor.saturating_add(1);
            continue;
        }
        out.push(chars[index]);
        index += 1;
    }
    out
}

/// Zdejmuje całe formatowanie (`§` i ANSI) - dopiero potem dopasowujemy wzorce.
pub fn strip_formatting(text: &str) -> String {
    strip_ansi(&strip_section_codes(text))
}

/// Sprawdza, czy adres nadaje się do otwarcia: tylko `https://`, bez spacji
/// ani znaków sterujących (zasada: tylko adresy `https://`).
pub fn is_allowed_login_url(url: &str) -> bool {
    url.len() > "https://".len()
        && url.len() <= 2048
        && url.starts_with("https://")
        && !url
            .chars()
            .any(|ch| ch.is_whitespace() || ch.is_control() || ch == '"' || ch == '<' || ch == '>')
}

/// Sprawdza, czy tekst wygląda na kod z promptu logowania (walidacja wejścia
/// komendy `copy_login_code`). Kod MCC to krótki ciąg znaków alfanumerycznych.
pub fn is_allowed_login_code(code: &str) -> bool {
    !code.is_empty()
        && code.chars().count() <= MAX_CODE_LEN
        && code.chars().all(|ch| ch.is_ascii_alphanumeric())
}

/// Wyciąga prompt logowania z jednej linii wyjścia MCC.
///
/// Zwraca `None`, gdy linia nie jest promptem albo adres/kod nie przechodzą
/// walidacji - lepiej nie pokazać bannera niż pokazać śmieć.
pub fn detect_device_code(raw: &str) -> Option<DeviceCodePrompt> {
    // Szybki odrzut: bez markera nie ma po co zdejmować formatowania.
    if !raw.contains(URL_MARKER) {
        return None;
    }
    let plain = strip_formatting(raw);
    let after_marker = plain.split_once(URL_MARKER)?.1;
    let (url, rest) = after_marker.split_once(CODE_MARKER)?;
    let url = url.trim();
    let code = rest.trim();

    if !is_allowed_login_url(url) {
        return None;
    }
    if code.is_empty() || code.chars().count() > MAX_CODE_LEN {
        return None;
    }
    if !code.chars().all(|ch| ch.is_ascii_alphanumeric()) {
        return None;
    }

    Some(DeviceCodePrompt {
        url: url.to_string(),
        code: code.to_string(),
    })
}

/// Czy linia zawiera prompt logowania Microsoft (kod urządzenia albo link).
///
/// Używane przez zapis logów czatu: te wiersze **nigdy** nie mogą trafić do
/// pliku. Najpierw sprawdzamy pełny wzorzec, a potem same markery - gdyby MCC
/// zmienił otoczenie kodu albo adresu, wiersz nadal nie trafi na dysk.
pub fn is_login_prompt(raw: &str) -> bool {
    if detect_device_code(raw).is_some() {
        return true;
    }
    let plain = strip_formatting(raw);
    plain.contains(URL_HINT) || plain.contains(CODE_HINT)
}

/// Czy linia mówi, że klient połączył się z serwerem.
pub fn is_joined(raw: &str) -> bool {
    raw.contains(JOINED_MARKER) || strip_formatting(raw).contains(JOINED_MARKER)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_real_prompt_from_mcc_notes() {
        let line = "To sign in, open https://www.microsoft.com/link in your browser and enter the code: §e7FRBLM2Y";
        let prompt = detect_device_code(line).expect("prompt powinien być wykryty");
        assert_eq!(prompt.url, "https://www.microsoft.com/link");
        assert_eq!(prompt.code, "7FRBLM2Y");
    }

    #[test]
    fn detects_prompt_with_log_prefix_and_ansi() {
        let line = "\u{1b}[90m§8[MCC] To sign in, open https://microsoft.com/devicelogin in your browser and enter the code: \u{1b}[33mABCD1234\u{1b}[0m";
        let prompt = detect_device_code(line).expect("prompt powinien być wykryty");
        assert_eq!(prompt.url, "https://microsoft.com/devicelogin");
        assert_eq!(prompt.code, "ABCD1234");
    }

    #[test]
    fn ignores_unrelated_lines() {
        assert!(detect_device_code("Connecting to Microsoft...").is_none());
        assert!(detect_device_code("Waiting for authentication to complete...").is_none());
        assert!(detect_device_code("§8[MCC] Version is supported.").is_none());
    }

    #[test]
    fn rejects_non_https_urls() {
        let line = "To sign in, open http://evil.example.com in your browser and enter the code: §eABCD1234";
        assert!(detect_device_code(line).is_none());
        assert!(!is_allowed_login_url("http://evil.example.com"));
        assert!(!is_allowed_login_url("javascript:alert(1)"));
        assert!(!is_allowed_login_url("https://a b.example.com"));
        assert!(is_allowed_login_url("https://www.microsoft.com/link"));
    }

    #[test]
    fn rejects_empty_or_oversized_code() {
        let empty = "To sign in, open https://x.example.com in your browser and enter the code: ";
        assert!(detect_device_code(empty).is_none());
        let long = format!(
            "To sign in, open https://x.example.com in your browser and enter the code: {}",
            "A".repeat(64)
        );
        assert!(detect_device_code(&long).is_none());
    }

    #[test]
    fn login_code_validation_matches_prompt_rules() {
        assert!(is_allowed_login_code("7FRBLM2Y"));
        assert!(!is_allowed_login_code(""));
        assert!(!is_allowed_login_code("AB CD"));
        assert!(!is_allowed_login_code(&"A".repeat(64)));
    }

    #[test]
    fn rejects_code_with_spaces() {
        let line =
            "To sign in, open https://x.example.com in your browser and enter the code: AB CD";
        assert!(detect_device_code(line).is_none());
    }

    #[test]
    fn login_prompt_is_recognised_for_log_filtering() {
        // Pełny prompt z realnej próbki (`docs/MCC_NOTES.md`).
        assert!(is_login_prompt(
            "To sign in, open https://www.microsoft.com/link in your browser and enter the code: §e7FRBLM2Y"
        ));
        // Sama linia z adresem albo sam kod też nie mają prawa przejść.
        assert!(is_login_prompt(
            "§8[MCC] To sign in, open https://microsoft.com/devicelogin in your browser and enter the code: "
        ));
        assert!(is_login_prompt(
            "in your browser and enter the code: 7FRBLM2Y"
        ));
        // Zwykły ruch z serwera przechodzi do pliku.
        assert!(!is_login_prompt("§8[MCC] Server was successfully joined."));
        assert!(!is_login_prompt("<Steve> hello"));
        assert!(!is_login_prompt("§8You chose to run in offline mode."));
    }

    #[test]
    fn joined_marker_is_detected_with_formatting() {
        assert!(is_joined("§8[MCC] Server was successfully joined."));
        assert!(is_joined("Server was successfully joined."));
        assert!(!is_joined("Type '/quit' to leave the server."));
    }

    #[test]
    fn strip_formatting_removes_both_formats() {
        assert_eq!(strip_formatting("§8[MCC] §aOK"), "[MCC] OK");
        assert_eq!(strip_formatting("§#ff0000Red"), "Red");
        assert_eq!(strip_formatting("\u{1b}[31mRed\u{1b}[0m"), "Red");
    }
}

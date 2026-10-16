//! Presety szybkich akcji (kontrakt z frontendem) i ich walidacja.
//!
//! Preset to gotowy wpis, który użytkownik wysyła jednym kliknięciem: `label`
//! widać na przycisku, a `text` idzie **dosłownie** do `stdin` MCC (tak jakby
//! został wpisany w polu wpisywania). Dlatego tekst presetu przechodzi przez tę
//! samą walidację co wpis ręczny (`input::validate_input`): jedna linia, bez
//! znaków sterujących, w limicie długości.
//!
//! Wczytanie z dysku jest **łagodne** (ręcznie edytowany `presets.json` nie
//! wywala aplikacji), a walidację przechodzi dopiero zapis z UI - dzięki temu
//! frontend nigdy nie pokazuje danych, których backend by nie przyjął.
//!
//! Preset nie ma pola „wyślij do wszystkich”: o zasięgu decyduje checkbox przy
//! polu wpisywania, wspólny dla wiadomości i dla akcji.

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::error::AppError;
use crate::input;

/// Maksymalna liczba presetów (pasek ma limit trzech linii, więc wyżej to już
/// przewijanie; wartość chroni też plik przed patologicznym rozmiarem).
pub const MAX_PRESETS: usize = 60;

/// Najdłuższa etykieta presetu (tekst na przycisku).
pub const MAX_PRESET_LABEL_LEN: usize = 40;

/// Kolory presetów. Nazwy (nie wartości HEX) są kontraktem z frontendem, więc
/// paleta żyje w jednym miejscu po obu stronach.
pub const PRESET_COLORS: [&str; 5] = ["blue", "green", "amber", "red", "violet"];

/// Preset jest domyślnie włączony - wyłączenie to świadoma decyzja użytkownika,
/// a plik edytowany ręcznie nie powinien gasić wszystkich akcji.
fn default_enabled() -> bool {
    true
}

/// Pojedynczy preset szybkiej akcji. Pola w JSON-ie są `camelCase`.
///
/// Preset **nie wie**, czy wysłać do wszystkich: o tym decyduje checkbox
/// „Wyślij do wszystkich” przy polu wpisywania, tak samo jak dla wiadomości
/// wpisanych ręcznie.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Preset {
    /// Stabilny identyfikator (nadaje frontend przy dodaniu).
    pub id: String,
    /// Tekst widoczny na przycisku.
    pub label: String,
    /// To, co naprawdę trafia do MCC (np. `/send /home`).
    pub text: String,
    /// Opcjonalny kolor paska (`PRESET_COLORS`); `None` = bez paska.
    #[serde(default)]
    pub color: Option<String>,
    /// Czy przycisk akcji jest włączony (wyłączony zostaje na pasku, ale widoczny).
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

/// Sprawdza i porządkuje listę presetów przed zapisem na dysk.
///
/// Zwraca listę gotową do zapisania (etykiety przycięte), a przy pierwszym
/// problemie - stabilny kod błędu z parametrami.
pub fn validate_presets(presets: Vec<Preset>) -> Result<Vec<Preset>, AppError> {
    if presets.len() > MAX_PRESETS {
        return Err(AppError::with_params(
            "presets.tooMany",
            json!({ "limit": MAX_PRESETS }),
        ));
    }

    let mut normalized = Vec::with_capacity(presets.len());
    let mut seen: Vec<String> = Vec::with_capacity(presets.len());

    for preset in presets {
        let id = preset.id.trim().to_string();
        if id.is_empty() {
            return Err(AppError::new("presets.invalidId"));
        }
        if seen.contains(&id) {
            return Err(AppError::new("presets.duplicateId"));
        }
        seen.push(id.clone());

        let label = preset.label.trim().to_string();
        if label.is_empty() {
            return Err(AppError::new("presets.labelRequired"));
        }
        if label.chars().count() > MAX_PRESET_LABEL_LEN {
            return Err(AppError::with_params(
                "presets.labelTooLong",
                json!({ "limit": MAX_PRESET_LABEL_LEN }),
            ));
        }
        if label.chars().any(char::is_control) {
            return Err(AppError::new("presets.labelControlChars"));
        }

        // Tekst presetu to wpis do MCC, więc obowiązują dokładnie te same zasady
        // co przy wpisywaniu ręcznym. Pusty tekst ma własny kod, bo w edytorze
        // presetów „puste pole” jest czytelniejsze niż „puste wejście”.
        let text = input::validate_input(preset.text.trim()).map_err(|error| {
            if error.code == "input.empty" {
                AppError::new("presets.textRequired")
            } else {
                error
            }
        })?;

        let color = match preset.color {
            Some(raw) if raw.trim().is_empty() => None,
            Some(raw) => {
                let color = raw.trim().to_string();
                if !PRESET_COLORS.contains(&color.as_str()) {
                    return Err(AppError::new("presets.invalidColor"));
                }
                Some(color)
            }
            None => None,
        };

        normalized.push(Preset {
            id,
            label,
            text,
            color,
            enabled: preset.enabled,
        });
    }

    Ok(normalized)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn preset(id: &str, label: &str, text: &str) -> Preset {
        Preset {
            id: id.into(),
            label: label.into(),
            text: text.into(),
            color: None,
            enabled: true,
        }
    }

    #[test]
    fn accepts_a_normal_list() {
        let list = vec![
            Preset {
                color: Some("blue".into()),
                ..preset("p1", "/home", "/send /home")
            },
            Preset {
                color: Some("violet".into()),
                enabled: false,
                ..preset("p2", "/reco", "/send /reco")
            },
        ];
        let validated = validate_presets(list.clone()).expect("poprawna lista");
        assert_eq!(validated, list);
    }

    #[test]
    fn empty_list_is_valid() {
        assert_eq!(validate_presets(Vec::new()).unwrap(), Vec::new());
    }

    #[test]
    fn trims_label_and_text() {
        let validated =
            validate_presets(vec![preset(" p1 ", "  /home  ", "  /send /home  ")]).unwrap();
        assert_eq!(validated[0].id, "p1");
        assert_eq!(validated[0].label, "/home");
        assert_eq!(validated[0].text, "/send /home");
    }

    #[test]
    fn rejects_empty_label() {
        assert_eq!(
            validate_presets(vec![preset("p1", "   ", "/send /home")])
                .unwrap_err()
                .code,
            "presets.labelRequired"
        );
    }

    #[test]
    fn rejects_empty_text() {
        assert_eq!(
            validate_presets(vec![preset("p1", "/home", "")])
                .unwrap_err()
                .code,
            "presets.textRequired"
        );
    }

    #[test]
    fn rejects_empty_id_and_duplicates() {
        assert_eq!(
            validate_presets(vec![preset("", "/home", "/send /home")])
                .unwrap_err()
                .code,
            "presets.invalidId"
        );
        assert_eq!(
            validate_presets(vec![
                preset("same", "A", "/send /a"),
                preset("same", "B", "/send /b"),
            ])
            .unwrap_err()
            .code,
            "presets.duplicateId"
        );
    }

    #[test]
    fn rejects_text_with_control_characters() {
        // Nowa linia w tekście presetu wstrzyknęłaby drugą komendę do MCC.
        assert_eq!(
            validate_presets(vec![preset("p1", "/home", "/send /home\n/quit")])
                .unwrap_err()
                .code,
            "input.controlChars"
        );
    }

    #[test]
    fn rejects_too_long_label_and_text() {
        let long_label = "x".repeat(MAX_PRESET_LABEL_LEN + 1);
        assert_eq!(
            validate_presets(vec![preset("p1", &long_label, "/send /x")])
                .unwrap_err()
                .code,
            "presets.labelTooLong"
        );

        let long_text = "x".repeat(input::MAX_INPUT_LEN + 1);
        assert_eq!(
            validate_presets(vec![preset("p1", "/x", &long_text)])
                .unwrap_err()
                .code,
            "input.tooLong"
        );
    }

    #[test]
    fn rejects_unknown_color_and_treats_blank_as_none() {
        assert_eq!(
            validate_presets(vec![Preset {
                color: Some("#ff00ff".into()),
                ..preset("p1", "/home", "/send /home")
            }])
            .unwrap_err()
            .code,
            "presets.invalidColor"
        );

        let blank = validate_presets(vec![Preset {
            color: Some("  ".into()),
            ..preset("p1", "/home", "/send /home")
        }])
        .unwrap();
        assert_eq!(blank[0].color, None);
    }

    #[test]
    fn rejects_more_than_the_limit() {
        let many: Vec<Preset> = (0..=MAX_PRESETS)
            .map(|index| preset(&format!("p{index}"), "/x", "/send /x"))
            .collect();
        assert_eq!(validate_presets(many).unwrap_err().code, "presets.tooMany");

        let at_limit: Vec<Preset> = (0..MAX_PRESETS)
            .map(|index| preset(&format!("p{index}"), "/x", "/send /x"))
            .collect();
        assert_eq!(validate_presets(at_limit).unwrap().len(), MAX_PRESETS);
    }

    #[test]
    fn serializes_with_camel_case_and_layer_defaults() {
        let raw = r#"[
            { "id": "p1", "label": "/home", "text": "/send /home" },
            { "id": "p2", "label": "/reco", "text": "/send /reco", "color": "red", "enabled": false }
        ]"#;
        let parsed: Vec<Preset> = serde_json::from_str(raw).expect("odczyt presetów");
        assert_eq!(parsed[0].color, None);
        // Brak pola w pliku nie może gasić akcji.
        assert!(parsed[0].enabled);
        assert!(!parsed[1].enabled);

        let json = serde_json::to_string(&parsed[1]).expect("zapis presetu");
        assert!(json.contains("\"enabled\":false"), "JSON: {json}");
        // Stary plik z `sendToAll` wczytuje się bez błędu (pole jest ignorowane),
        // a preset zostaje włączony.
        let legacy: Vec<Preset> = serde_json::from_str(
            r#"[{ "id": "p3", "label": "A", "text": "/send /a", "sendToAll": true }]"#,
        )
        .expect("odczyt starego pliku");
        assert!(legacy[0].enabled);
    }
}

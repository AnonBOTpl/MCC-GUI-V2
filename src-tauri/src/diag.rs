//! Diagnostyka zamrożenia widoku logu - **opcjonalna, wyłączona domyślnie**.
//!
//! Włączana tylko świadomie, zmienną `MCC_GUI_DIAG=1` (albo `true`) w środowisku
//! procesu aplikacji - tak samo w buildzie deweloperskim i w wydaniu. Bez tej
//! zmiennej `enabled()` zwraca `false`, więc nic nie trafia na dysk, a `count_*`
//! są pomijane (koszt zerowy). Po stronie UI frontend pyta o to raz przy starcie
//! (`diag_enabled`) i przy wyłączonej diagnostyce nie zakłada żadnej pętli.
//!
//! Po co: zgłoszenie użytkownika („log odświeża się dopiero po ruszeniu kółkiem
//! myszy”) przeszło już przez dwie hipotezy bez rozstrzygnięcia. Ten moduł zbiera
//! dane, które **jednym przebiegiem** mówią, która warstwa stoi:
//!
//! - czy Rust czyta linie z MCC i wysyła zdarzenia (`linesRead` / `linesEmitted`),
//! - czy frontend je odbiera i dopisuje do bufora (liczniki po stronie UI),
//! - czy przeglądarka produkuje klatki (`frames`, `frameAgeMs`),
//! - czy działają timery (`intervalMs` między raportami),
//! - czy okno było w tym czasie zminimalizowane (`window.minimized`).
//!
//! Raport trafia jako **linia JSON** do `diag.log` w katalogu danych aplikacji.
//! W pliku nie ma treści logów ani niczego związanego z poświadczeniami - tylko
//! liczby i czasy.
//!
//! By ją włączyć, ustaw zmienną przed startem aplikacji (np. w skrypcie uruchomieniowym):
//! `set "MCC_GUI_DIAG=1"`. Powstały plik `diag.log` leży w katalogu danych
//! aplikacji (`%APPDATA%\com.mccgui.v2\`).

use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{Value, json};

/// Nazwa pliku z raportami (w katalogu danych aplikacji).
const FILE_NAME: &str = "diag.log";
/// Po przekroczeniu tego rozmiaru plik leci na `diag.log.1` i zaczynamy od nowa.
const MAX_BYTES: u64 = 4 * 1024 * 1024;

/// Czy diagnostyka jest włączona. Pytamy raz - wynik jest stały dla procesu.
static ENABLED: OnceLock<bool> = OnceLock::new();

/// Linie odebrane z MCC (przed wysłaniem do UI).
static LINES_READ: AtomicU64 = AtomicU64::new(0);
/// Linie wysłane zdarzeniem do UI.
static LINES_EMITTED: AtomicU64 = AtomicU64::new(0);

/// Czy diagnostyka jest włączona: tylko na wyraźne życzenie (`MCC_GUI_DIAG=1`).
///
/// Build deweloperski **nie** włącza jej sam - inaczej `diag.log` rósłby przy
/// każdym uruchomieniu aplikacji, także wtedy, gdy nikt go nie potrzebuje.
pub fn enabled() -> bool {
    *ENABLED.get_or_init(|| std::env::var("MCC_GUI_DIAG").is_ok_and(|value| flag_on(&value)))
}

/// Czy wartość zmiennej `MCC_GUI_DIAG` włącza diagnostykę (`1` albo `true`).
fn flag_on(value: &str) -> bool {
    let value = value.trim();
    value == "1" || value.eq_ignore_ascii_case("true")
}

/// Liczy linię odczytaną z procesu MCC.
pub fn count_read() {
    if enabled() {
        LINES_READ.fetch_add(1, Ordering::Relaxed);
    }
}

/// Liczy linię wysłaną zdarzeniem do UI.
pub fn count_emitted() {
    if enabled() {
        LINES_EMITTED.fetch_add(1, Ordering::Relaxed);
    }
}

/// Liczniki po stronie Rusta - różnica względem liczników UI wskazuje, gdzie
/// giną linie albo że nic nie ginie (i problem jest dalej).
pub fn counters() -> Value {
    json!({
        "linesRead": LINES_READ.load(Ordering::Relaxed),
        "linesEmitted": LINES_EMITTED.load(Ordering::Relaxed),
    })
}

/// Bieżący czas w ms od epoki Unix (znacznik czasu raportu).
pub fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_millis() as i64)
        .unwrap_or_default()
}

/// Ścieżka pliku z raportami w danym katalogu danych.
pub fn file_path(root: &Path) -> PathBuf {
    root.join(FILE_NAME)
}

/// Dopisuje jedną linię raportu do `diag.log` (z rotacją).
///
/// Błąd zapisu zwracamy tylko po to, żeby dało się to przetestować - wołający
/// loguje go przez `tracing` i leci dalej. Diagnostyka nie może wpływać na
/// działanie aplikacji.
pub fn append(root: &Path, value: &Value) -> std::io::Result<()> {
    append_with_limit(root, value, MAX_BYTES)
}

/// Jak `append`, ale z podanym progiem rotacji (dla testów).
fn append_with_limit(root: &Path, value: &Value, max_bytes: u64) -> std::io::Result<()> {
    std::fs::create_dir_all(root)?;
    let path = file_path(root);

    // Rotacja: bez niej długa sesja zapchałaby dysk, a i tak interesuje nas
    // najświeższy fragment.
    if std::fs::metadata(&path).map(|meta| meta.len()).unwrap_or(0) > max_bytes {
        let rotated = root.join(format!("{FILE_NAME}.1"));
        let _ = std::fs::remove_file(&rotated);
        let _ = std::fs::rename(&path, &rotated);
    }

    let mut line = serde_json::to_string(value).unwrap_or_else(|_| "{}".to_string());
    line.push('\n');
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)?;
    file.write_all(line.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_root(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("mcc-diag-{}-{}", name, std::process::id()))
    }

    /// Każde wołanie dopisuje dokładnie jedną linię JSON.
    #[test]
    fn append_writes_one_line_per_report() {
        let root = temp_root("append");
        let _ = std::fs::remove_dir_all(&root);

        append(&root, &json!({"a": 1})).expect("zapis raportu");
        append(&root, &json!({"a": 2})).expect("zapis raportu");

        let content = std::fs::read_to_string(file_path(&root)).expect("odczyt raportu");
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[1].contains("\"a\":2"));

        let _ = std::fs::remove_dir_all(&root);
    }

    /// Po przekroczeniu progu stary plik leci na `diag.log.1`.
    #[test]
    fn append_rotates_when_file_is_too_big() {
        let root = temp_root("rotate");
        let _ = std::fs::remove_dir_all(&root);

        append(&root, &json!({"first": true})).expect("zapis raportu");
        // Próg mniejszy niż jeden raport - drugi zapis musi zrotować plik.
        append_with_limit(&root, &json!({"second": true}), 1).expect("zapis raportu");

        let rotated =
            std::fs::read_to_string(root.join(format!("{FILE_NAME}.1"))).expect("rotacja");
        assert!(rotated.contains("\"first\":true"));
        let current = std::fs::read_to_string(file_path(&root)).expect("odczyt raportu");
        assert!(current.contains("\"second\":true"));
        assert!(!current.contains("\"first\":true"));

        let _ = std::fs::remove_dir_all(&root);
    }

    /// Włącza tylko `1` i `true` - spacje i wielkość liter nie mają znaczenia.
    #[test]
    fn flag_is_opt_in() {
        assert!(flag_on("1"));
        assert!(flag_on("true"));
        assert!(flag_on(" TRUE "));
        assert!(!flag_on(""));
        assert!(!flag_on("0"));
        assert!(!flag_on("yes"));
    }

    /// Liczniki Rusta rosną i są widoczne w raporcie.
    #[test]
    fn counters_are_reported() {
        let before = counters();
        count_read();
        count_emitted();
        let after = counters();

        // Domyślnie diagnostyka jest wyłączona (liczniki stoją), więc porównujemy
        // tylko przy włączonej - `MCC_GUI_DIAG=1` ustawionym dla procesu testu.
        if enabled() {
            assert_ne!(before, after);
        }
    }
}

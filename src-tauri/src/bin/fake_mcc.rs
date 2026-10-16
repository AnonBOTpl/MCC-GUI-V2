//! Fałszywy „MCC" używany w testach menedżera procesów. Nie wymaga prawdziwego
//! klienta ani serwera. Zachowuje się jak prosty proces liniowy:
//!
//! - na start wypisuje gotowość na `stdout` i `stderr`,
//! - `echo: <linia>` dla zwykłego wpisu (wypisuje go z powrotem),
//! - `/quit` kończy proces kodem 0,
//! - `crash` kończy proces kodem 3 (symulacja awarii).
//!
//! Flaga `--delay-ms <n>` opóźnia pierwszą odpowiedź, do testów kolejności.
//!
//! Tryb obciążenia (do sprawdzenia, czy UI wyrabia przy ~200 liniach/s) włącza się
//! zmiennymi środowiskowymi, a nie argumentami, bo aplikacja zawsze uruchamia
//! klienta jako `MinecraftClient.exe settings.ini BasicIO` - nie ma gdzie wcisnąć
//! dodatkowej flagi. Zmienne środowiskowe dziedziczą się do procesu potomnego:
//!
//! - `FAKE_MCC_SPAM` - ile linii na sekundę wypisywać (0 = wyłączone),
//! - `FAKE_MCC_SPAM_SECONDS` - jak długo (domyślnie 60),
//! - `FAKE_MCC_PROMPT` - po starcie wypisz prompt logowania Microsoft (Etap 3),
//!   a po `join` zdanie o udanym połączeniu (symuluje logowanie urządzenia).

use std::io::{self, BufRead, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

/// Odczytuje zmienną środowiskową jako liczbę.
fn env_number(name: &str, fallback: u64) -> u64 {
    std::env::var(name)
        .ok()
        .and_then(|value| value.trim().parse::<u64>().ok())
        .unwrap_or(fallback)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if let Some(pos) = args.iter().position(|arg| arg == "--delay-ms")
        && let Some(value) = args.get(pos + 1).and_then(|v| v.parse::<u64>().ok())
    {
        thread::sleep(Duration::from_millis(value));
    }

    let stdout = io::stdout();
    {
        let mut out = stdout.lock();
        let _ = writeln!(out, "§8[fake] ready");
        let _ = out.flush();
    }
    eprintln!("§8[fake] stderr ready");

    // Symulacja logowania Microsoft. Wyzwalaczem jest plik w katalogu roboczym
    // konta, a nie zmienna środowiskowa - zmienne są wspólne dla całego procesu
    // testowego i wyciekałyby między równoległymi testami.
    if std::path::Path::new("fake-prompt").exists() {
        let mut out = stdout.lock();
        let _ = writeln!(out, "Connecting to Microsoft...");
        let _ = writeln!(
            out,
            "To sign in, open https://www.microsoft.com/link in your browser and enter the code: §e7FRBLM2Y"
        );
        let _ = writeln!(out, "Waiting for authentication to complete...");
        let _ = out.flush();
    }

    // Wątek obciążenia: wypisuje linie w tempie zbliżonym do zadanego.
    let spam_per_second = env_number("FAKE_MCC_SPAM", 0);
    let stopped = Arc::new(AtomicBool::new(false));
    if spam_per_second > 0 {
        let seconds = env_number("FAKE_MCC_SPAM_SECONDS", 60);
        let stopped = Arc::clone(&stopped);
        // `Stdout` w uchwycie: blokadę bierzemy na czas pojedynczego zapisu.
        let stdout = io::stdout();
        thread::spawn(move || {
            let interval = Duration::from_micros(1_000_000 / spam_per_second.max(1));
            let deadline = Instant::now() + Duration::from_secs(seconds);
            let mut index: u64 = 0;
            while !stopped.load(Ordering::Relaxed) && Instant::now() < deadline {
                {
                    let mut out = stdout.lock();
                    let _ = writeln!(out, "§8[spam] §e#{index} §florem ipsum §adolor §bsit");
                    let _ = out.flush();
                }
                index += 1;
                thread::sleep(interval);
            }
        });
    }

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = match line {
            Ok(line) => line,
            Err(_) => break,
        };
        match line.as_str() {
            "/quit" => {
                stopped.store(true, Ordering::Relaxed);
                let mut out = stdout.lock();
                let _ = writeln!(out, "§8[fake] bye");
                let _ = out.flush();
                return;
            }
            "crash" => {
                stopped.store(true, Ordering::Relaxed);
                std::process::exit(3);
            }
            "join" => {
                let mut out = stdout.lock();
                let _ = writeln!(out, "§8[MCC] Server was successfully joined.");
                let _ = writeln!(out, "Type '/quit' to leave the server.");
                let _ = out.flush();
            }
            other => {
                let mut out = stdout.lock();
                let _ = writeln!(out, "echo: {other}");
                let _ = out.flush();
            }
        }
    }
    stopped.store(true, Ordering::Relaxed);
}

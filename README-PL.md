# MCC GUI V2

Lokalna aplikacja desktopowa na Windows 11 do **uruchamiania wielu kont Minecrafta naraz,
każdego na jednym lub kilku serwerach**. Konto to tożsamość logowania z własną sesją MCC;
każda para (konto, serwer) to osobny proces
[Minecraft Console Client](https://mccteam.github.io/guide/usage.html) (MCC) i osobna
zakładka. Aplikacja daje jedno okno do wszystkich: kolorowe logi na żywo, wysyłanie tekstu
do jednego połączenia albo do wszystkich oraz banner logowania dla kont Microsoft.

Silnikiem zostaje MCC — aplikacja sama nie implementuje protokołu Minecrafta i nigdy nie
dołącza, nie pobiera ani nie modyfikuje MCC.

🇬🇧 English version: [README.md](README.md)

## Stan projektu

Prace w toku.

| Etap | Czego dotyczy | Status |
|------|---------------|--------|
| 0 | Szkielet, narzędzia, research MCC | gotowe |
| 1 | Menedżer procesów (Rust) | gotowe |
| 2 | Interfejs: zakładki, log, wpis | gotowe — 200 linii/s utrzymuje ~59 fps |
| 3 | Banner logowania Microsoft i obsługa statusów | gotowe |
| 4 | Trwałość: konta, ustawienia, serwery per konto | gotowe |
| 5 | Dopracowanie przewijania logu | gotowe (filtry logu są poza zakresem) |
| 6 | Szybkie akcje | gotowe |
| 7 | Przełącznik języka, finalne tłumaczenie, pakowanie | do zrobienia — patrz notka niżej |

**Etap 7** ma już przełącznik języka, ikonę i instalator; zostaje przeczytanie polskiego
tłumaczenia, dlatego wiersz powyżej nadal jest `do zrobienia`. Wszystko z etapów 0-6 działa.

**Aktualne ograniczenia:** instalator nie jest podpisany, więc Windows pokaże ostrzeżenie
SmartScreen przed jego uruchomieniem.

## Wymagania

- **Windows 11** (jedyny obsługiwany system).
- **Node.js 24+** i npm.
- **Rust 1.90+** z toolchainem MSVC (`rustup default stable-msvc`). Crate używa edycji 2024.
- **WebView2** — w Windows 11 jest już w systemie.
- Sam **Minecraft Console Client**. To osobny program, który pobierasz i trzymasz w stałym
  miejscu; aplikacja tylko wskazuje na `MinecraftClient.exe`.

## Jak zacząć

```bash
npm install
npm run tauri dev
```

Przy pierwszym uruchomieniu aplikacja otworzy swoje ustawienia, bo nie wie jeszcze, gdzie
jest MCC:

1. Kliknij **Wybierz** i wskaż swój plik `MinecraftClient.exe`.
2. Kliknij **Zapisz**.

Ścieżka zostaje zapisana i jest używana przy kolejnych uruchomieniach. Potem naciśnij
**Dodaj konto** w nagłówku panelu kont po lewej, wybierz typ (Microsoft albo offline), wpisz
login i zatwierdź — konto pojawi się w panelu, a nic się jeszcze nie łączy. Dodaj mu tam
jeden lub więcej serwerów (adres i w razie potrzeby wymuszoną wersję Minecrafta), a potem
naciśnij zielony przycisk **Play** przy serwerze, żeby się połączyć.

Do testów możesz też pominąć okno ustawień i podać plik zmienną `MCC_EXECUTABLE` — zapisana
ścieżka zawsze ma pierwszeństwo:

```bash
MCC_EXECUTABLE="C:/sciezka/do/MinecraftClient.exe" npm run tauri dev
```

Zamykaj aplikację przyciskiem ✕ w oknie. Zamknięcie zatrzymuje wszystkie uruchomione przez
nią procesy MCC (są powiązane z aplikacją przez Windows Job Object), więc w Menedżerze
zadań nie powinien zostać żaden `MinecraftClient.exe`.

## Budowanie wersji release

```bash
npm run tauri build
```

Plik wykonywalny i paczki lądują w `src-tauri/target/release/` oraz
`src-tauri/target/release/bundle/`.

### Instalator

```bash
npm run tauri -- build --bundles nsis
```

Powstaje instalator NSIS dla bieżącego użytkownika:
`src-tauri/target/release/bundle/nsis/MCC GUI V2_2.0.0_x64-setup.exe` (numer wersji w nazwie
bierze się z `version` w `src-tauri/tauri.conf.json`). Nie potrzebuje praw administratora i
instaluje się w profilu użytkownika.

Instalator **nie jest podpisany**, więc przy pierwszym uruchomieniu Windows pokaże
ostrzeżenie SmartScreen — wybierz **Więcej informacji** → **Uruchom mimo to**. Certyfikat do
podpisu jest płatny i zostaje poza zakresem MVP.

Zainstalowana wersja ma ten sam identyfikator co build deweloperski, więc czyta te same pliki
w `%APPDATA%` (patrz niżej) — konta, ustawienia i otwarte zakładki przechodzą z jednej wersji
do drugiej, a w MCC nie logujesz się ponownie.

## Jak to działa

- **Rust `process_manager`** uruchamia jeden `MinecraftClient.exe` na połączenie (konto +
  serwer). Konto ma jeden katalog roboczy współdzielony przez swoje połączenia (jedna sesja
  MCC), a każde połączenie dostaje własny generowany `.ini`; proces startuje jako
  `MinecraftClient.exe settings.ini BasicIO`, dzięki czemu wyjście przychodzi jako czysty
  strumień linii (z zachowanymi kodami kolorów `§`).
- **stdout/stderr** są przekazywane linia po linii do interfejsu razem z identyfikatorem
  konta, strumieniem i znacznikiem czasu. Rust nie zmienia treści komunikatów MCC.
- **Frontend** parsuje kody `§` i ANSI, klasyfikuje linie i trzyma ograniczony bufor logu na
  połączenie, wyświetlany jako lista wirtualizowana — interfejs zostaje płynny przy kilkuset
  liniach na sekundę.
- **Autoprzewijanie jest per zakładka:** przewinięcie w górę w jednym połączeniu zatrzymuje
  tylko tę zakładkę, a każda pamięta własne ustawienie. Uruchomienie albo restart połączenia
  wznawia śledzenie, bo jego log startuje od zera.
- **Jedynym łączem** między Rustem a interfejsem jest kontrakt komend i zdarzeń Tauri
  opisany w `src/api/contract.ts`.
- **Frontend nigdy nie uruchamia procesów ani nie dotyka plików** — to wszystko dzieje się
  w Rustcie.
- **Szybkie akcje** to przyciski-presety na dole połączenia: cienki uchwyt rozwija panel,
  który pokazuje najwyżej trzy linie przycisków (co więcej, przewija się w środku panelu,
  więc log zachowuje swoje miejsce). Klik wysyła tekst presetu dokładnie tak, jakbyś go
  wpisał — z tym samym zasięgiem: do aktywnego połączenia albo do wszystkich działających,
  gdy zaznaczony jest checkbox „Wyślij do wszystkich” pod logiem. `Edytuj` otwiera edytor
  (dodawanie, zmiana nazwy i koloru, kolejność przez przeciąganie uchwytu, usuwanie,
  włączanie i wyłączanie akcji — wyłączona znika z paska, ale zostaje w edytorze),
  a `Przeładuj` wczytuje `presets.json` od nowa po ręcznej edycji pliku.

## Gdzie trafiają Twoje dane

Wszystko leży w `%APPDATA%\com.mccgui.v2\`:

| Ścieżka | Zawartość |
|---------|-----------|
| `settings.json` | Ustawienia aplikacji: ścieżka MCC, język, odstęp „wyślij do wszystkich", rozmiar bufora logu, przełącznik przywracania zakładek, historia serwerów |
| `accounts.json` | Zapisane konta, każde z listą serwerów |
| `presets.json` | Przyciski szybkich akcji: etykieta, dokładna linia wysyłana do MCC, opcjonalny kolor i przełącznik wł./wył. per akcja |
| `accounts\<id>\` | Katalog roboczy MCC dla konta: jeden generowany `.ini` na połączenie i własny cache sesji MCC |

Zainstalowana wersja i build deweloperski korzystają z tego samego katalogu (ten sam
identyfikator), a ścieżkę do MCC biorą z `settings.json` — zmienna `MCC_EXECUTABLE` to tylko
wygoda przy pracy nad kodem.

**Aplikacja nigdy nie zapisuje haseł ani tokenów.** Obsługuje konta Microsoft (bez hasła;
MCC używa logowania kodem urządzenia) i konta offline (bez hasła). Pliki sesji MCC zostają
w katalogu konta — aplikacja ich nie czyta, nie kopiuje i nie eksportuje.

## Komendy deweloperskie

| Komenda | Co robi |
|---------|---------|
| `npm run tauri dev` | Uruchamia aplikację z hot reloadem |
| `npm run dev` | Sam frontend (serwer Vite na porcie 1420) |
| `npm run build` | Sprawdza typy i buduje frontend |
| `npm run tauri build` | Buduje wersję release i paczki |
| `npm run typecheck` | `vue-tsc --noEmit` |
| `npm run lint` | ESLint |
| `npm run test` | Vitest (testy jednostkowe) |
| `npm run test:watch` | Vitest w trybie watch |
| `npm run check:i18n` | Błąd, gdy pliki `en` i `pl` mają różne klucze |

Sprawdzenia po stronie Rusta:

```bash
cd src-tauri
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Wszystko powyższe musi przechodzić, zanim zmiana zostanie uznana za skończoną.

## Języki

Interfejs jest **domyślnie angielski** i tak startuje przy pierwszym uruchomieniu, niezależnie
od języka systemu. Polski włącza się przełącznikiem języka w interfejsie, a wybór jest
pamiętany.

Każdy widoczny tekst przechodzi przez `vue-i18n`. `en` jest źródłem prawdy i fallbackiem,
a `pl` musi mieć wszystkie klucze — pilnuje tego `npm run check:i18n`. Rust nigdy nie wysyła
do interfejsu tekstów dla człowieka: wysyła stabilne kody błędów z parametrami, a frontend je
tłumaczy. Surowy log MCC i serwera pokazujemy dokładnie tak, jak przyszedł, bez tłumaczenia.

## Struktura projektu

```
src/                 Frontend Vue 3 (TypeScript, Pinia, Tailwind, vue-i18n)
  api/               Typy kontraktu Tauri i opakowania invoke/listen
  components/        Komponenty UI (panel kont, pasek tytułu, zakładki, log, banner logowania, ...)
  log/               Parsowanie kolorów, bufor logu, zawijanie wierszy, statusy (z testami jednostkowymi)
  stores/            Store'y Pinia (konta, logi, ustawienia)
  i18n/              Pliki językowe, tłumaczenie kodów błędów
src-tauri/           Backend w Rustcie
  src/connection.rs        Identyfikator połączenia i nazwa pliku ustawień per połączenie
  src/process_manager.rs   Jeden proces MCC na połączenie, sprzątanie przez Job Object
  src/device_code.rs       Wykrywanie promptu logowania Microsoft
  src/settings.rs          Model ustawień i walidacja
  src/store.rs             Atomowy zapis JSON
  src/bin/fake_mcc.rs      Atrapa MCC używana w testach automatycznych
  tests/                   Testy integracyjne na atrapie
scripts/             Skrypty repozytorium (sprawdzanie i18n, inspekcja UI)
```

## Materiał referencyjny

`reference/` zawiera tylko do odczytu źródła MCC oraz `MinecraftClient.exe` do lokalnych
testów. Katalog jest **w .gitignore**, wyłączony z TypeScript, ESLint, Vite i Vitest i nie
jest potrzebny do zbudowania ani uruchomienia aplikacji. Zachowania, na których opiera się
aplikacja, weryfikujemy na prawdziwym pliku wykonywalnym.

## Uwagi i zastrzeżenia

- **Logowanie Microsoft działa przez kod urządzenia.** MCC wypisuje kod i link, a aplikacja
  pokazuje oba w bannerze logowania. To, na które konto Microsoft się zalogujesz, zależy od
  sesji w przeglądarce, a nie od loginu wpisanego w aplikacji — przy kolejnych kontach
  otwórz link w oknie prywatnym albo innym profilu przeglądarki. MCC sam też otwiera
  przeglądarkę.
- **MCC pamięta każde konto** po pierwszym udanym zalogowaniu (jego cache sesji leży
  w katalogu konta), więc kod kopiujesz tylko raz na konto, a każdy serwer tego konta
  korzysta z tego samego logowania.
- Konta offline działają bez logowania do Microsoftu.
- Aplikacja nie wykonuje żadnych własnych zapytań sieciowych. Wszystko działa offline poza
  tym, czego potrzebuje sam MCC.

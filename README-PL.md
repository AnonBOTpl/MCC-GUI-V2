# MCC GUI V2

Lokalna aplikacja desktopowa na Windows 11 do **uruchamiania wielu kont Minecrafta naraz,
każdego na jednym lub kilku serwerach**.

Jedno okno zamiast kilkunastu okien konsoli: każde konto dostaje własną zakładkę z kolorowym
logiem na żywo, czat i komendy wysyłasz do jednego połączenia albo do wszystkich naraz, a
logowanie do Microsoftu sprowadza się do przepisania kodu pokazanego na ekranie — żadne hasło
nie jest wpisywane do aplikacji ani przez nią zapisywane.

Samo połączenie z Minecraftem prowadzi
[Minecraft Console Client](https://mccteam.github.io/guide/usage.html) (MCC) — ten sam
program, który uruchamiałbyś ręcznie. Aplikacja startuje po jednym procesie MCC na połączenie
i pokazuje to, co one wypisują.

🇬🇧 English version: [README.md](README.md)

## Czego potrzebujesz

- **Windows 11.** Aplikacja jest pisana wyłącznie pod Windows.
- **Minecraft Console Client.** Osobny program, który sam pobierasz ze strony
  [mccteam.github.io](https://mccteam.github.io/guide/usage.html) i trzymasz w stałym
  miejscu. Aplikacja nigdy go nie dołącza ani nie pobiera — uruchamia tylko wskazany przez
  Ciebie plik `MinecraftClient.exe`.
- **MCC 1.x.** Aplikacja współpracuje z klasycznym Minecraft Console Client (1.x, wydanie
  z `MinecraftClient.exe`). MCC 2.0 Early Access nie jest jeszcze obsługiwany.
- Nic więcej. Komponent WebView2, z którego korzysta aplikacja, jest już częścią Windows 11.

## Instalacja

1. Pobierz `MCC.GUI.V2_2.0.0_x64-setup.exe` ze strony wydań tego repozytorium (w wydaniu
   nazwa pliku ma kropki zamiast spacji, bo GitHub nie przyjmuje spacji w nazwach plików).
2. Uruchom plik. Windows pokaże ostrzeżenie **SmartScreen**, bo instalator nie jest podpisany
   certyfikatem — wybierz **Więcej informacji** → **Uruchom mimo to**.
3. Instalator **nie potrzebuje uprawnień administratora**; instaluje się w Twoim profilu.

Skoro instalator nie jest podpisany, podpis i tak nie powie Ci, czy pobrany plik jest tym,
który opublikował projekt. Dlatego przy każdym wydaniu w opisie wydania podany jest **SHA256**
instalatora. Porównaj go, zanim uruchomisz plik:

```powershell
Get-FileHash -Algorithm SHA256 ".\MCC.GUI.V2_2.0.0_x64-setup.exe"
```

Jeśli w Twoim systemie tego polecenia nie ma (albo używasz starszej powłoki), ten sam skrót
policzy `certutil`, który jest w każdym Windowsie:

```powershell
certutil -hashfile ".\MCC.GUI.V2_2.0.0_x64-setup.exe" SHA256
```

Jeśli skrót różni się od tego w opisie wydania, nie uruchamiaj instalatora.

## Pierwsze uruchomienie

Przy pierwszym uruchomieniu aplikacja otworzy swoje Ustawienia, bo nie wie jeszcze, gdzie jest
MCC:

1. Kliknij **Wybierz** i wskaż swój plik `MinecraftClient.exe`.
2. Kliknij **Zapisz**.

Ta ścieżka jest zapamiętana i używana przy kolejnych uruchomieniach.

Potem naciśnij **Dodaj konto** w nagłówku panelu kont po lewej, wybierz typ konta, wpisz login
i zatwierdź:

- **Microsoft** — podajesz adres e-mail. Bez hasła: MCC pokazuje kod do przepisania na stronie
  `microsoft.com/link`, a aplikacja wyświetla banner z kodem i przyciskiem, który otwiera tę
  stronę za Ciebie.
- **Offline** — sam nick, bez żadnego logowania.

Konto pojawia się w panelu i **nic się jeszcze nie łączy**. Dodaj mu tam jeden lub więcej
serwerów (adres, a w razie potrzeby wymuszoną wersję Minecrafta), a potem naciśnij zielony
przycisk odtwarzania przy serwerze (podpowiedź: **Uruchom**), żeby się połączyć.

## Jak korzystać z aplikacji

**Panel kont (po lewej).** Każde zapisane konto z listą serwerów i statusem każdego połączenia.
Stąd dodajesz i usuwasz konta oraz serwery, a także uruchamiasz i zatrzymujesz połączenia.
Serwer łączy się dopiero po naciśnięciu jego zielonego przycisku (podpowiedź: **Uruchom**) —
nic nie łączy się samo, a zapisane zakładki wracają zatrzymane.

**Zakładki (u góry).** Jedna zakładka na połączenie (konto + serwer), każda z własnym logiem
i własną kropką statusu. Przycisk zamknięcia zakładki zamyka ją **i zatrzymuje to
połączenie**; serwer zostaje zapisany na koncie i można go uruchomić ponownie.

**Nagłówek połączenia.** Etykieta konta, adres serwera i aktualny status, checkbox
**Autoprzewijanie** oraz przycisk **Uruchom** (gdy połączenie jest zatrzymane albo po awarii)
albo para **Rozłącz** i **Połącz ponownie**.

**Log.** Wyjście tego połączenia dokładnie takie, jakie wypisuje MCC, z oddanymi kolorami
Minecrafta. Długie wiersze zawijają się do szerokości okna. Przy włączonym autoprzewijaniu
widok nadąża za nowymi liniami; gdy przewiniesz w górę, przestaje nadążać, a gdy wrócisz na
dół (albo zaznaczysz **Autoprzewijanie**), znowu nadąża. Wiersze można zaznaczać i kopiować:
klik zaznacza jeden wiersz, `Ctrl`+klik dodaje lub odejmuje kolejny, `Shift`+klik zaznacza
zakres, `Ctrl`+`A` zaznacza wszystko, `Ctrl`+`C` kopiuje, `Esc` czyści zaznaczenie. Prawy
przycisk myszy daje **Kopiuj** i **Kopiuj wszystko**; to samo menu otwiera się z klawiatury
przez `Shift`+`F10` albo klawisz Menu przy pierwszym zaznaczonym wierszu, a strzałki oraz
`Home` / `End` chodzą po jego pozycjach. Skopiowany tekst jest czysty — kody kolorów są
usuwane.

**Pole wpisywania.** Wpisz tekst czatu i naciśnij `Enter` albo wpisz komendę MCC zaczynającą
się od `/`. Zaznacz **Wyślij do wszystkich**, żeby ta sama linia poszła do wszystkich
działających połączeń, z niewielkim odstępem między kontami, żeby serwer nie uznał tego za
spam. Komendy serwera trzeba wysyłać przez `/send`, na przykład `/send /home`.

**Szybkie akcje (na dole).** Twoje własne przyciski z komendami, których używasz najczęściej.
Pasek siedzi zwinięty jako cienka kreska na dole i rozwija się po kliknięciu. Kliknięcie akcji
wysyła jej tekst dokładnie tak, jakbyś go wpisał, z tym samym zasięgiem co pole wpisywania.
**Edytuj** pozwala dodawać, zmieniać nazwy i kolory, przestawiać kolejność i włączać lub
wyłączać akcje; wyłączona akcja znika z paska, ale zostaje w edytorze. Pasek i edytor są
dostępne tylko przy otwartej zakładce połączenia — bez zakładek akcje zostają zapisane
w `presets.json`, ale z poziomu aplikacji nie da się ich zmienić.

**Ustawienia (ikona koła zębatego w pasku tytułu).** Ścieżka do MCC, przywracanie zakładek
przy starcie, pytanie przed zamknięciem przy działających połączeniach, **Odstęp między
kontami** dla „wyślij do wszystkich”, **Liczba linii logu na konto** i ustawienia logów czatu. Obok tytułu aplikacji jest jeszcze
mały przełącznik **EN / PL** do zmiany języka, a wybór jest zapamiętywany.

**Zamykanie.** Zamknięcie okna zatrzymuje wszystkie procesy MCC uruchomione przez aplikację,
więc w tle nie zostaje żaden `MinecraftClient.exe`. Jeśli połączenia jeszcze działają,
aplikacja najpierw pyta — to pytanie możesz wyłączyć w tym samym okienku albo w Ustawieniach.
W zainstalowanej wersji drugie uruchomienie nie otwiera kolejnego okna, tylko wysuwa na
wierzch to już działające.

## Logi czatu

Logi czatu są **domyślnie włączone** i można je wyłączyć w Ustawieniach. Każde połączenie
zapisuje to, co widzi i pisze, do pliku:

```
<logi>\<serwer>\<etykieta konta>-<krótki skrót>\<RRRR-MM-DD>.log
```

Czteroznakowy skrót po etykiecie rozdziela konta, których etykiety różnią się tylko wielkością
liter albo spacjami na końcu (`Work` i `work` to w Windows ten sam folder). Skrót liczy się
z etykiety, więc połączenie zawsze pisze do tego samego folderu — po zmianie nazwy konta
jego logi zaczynają się w nowym.

`<logi>` to `%APPDATA%\com.mccgui.v2\logs`, o ile nie wskażesz innego folderu w Ustawieniach
(**Zmień**, **Otwórz folder**). Wszystkie sesje tego samego dnia dopisują się do jednego
pliku, a każdą z nich oddzielają wiersze `=== session start ... ===` i
`=== session end ... ===`. Po północy (czas lokalny) zapis przechodzi do pliku z nową datą,
także w działającej sesji.

Przełącznik i folder działają od razu: włączenie zapisu zaczyna pisać dla połączeń, które już
działają, a nowy folder dostaje kolejne linie bez restartowania połączeń.

Pliki są w UTF-8, jeden wiersz na linię wyjścia MCC: `[21:03:07] <Steve> hello`, bez kodów
kolorów Minecrafta (`§`) i sekwencji ANSI. To, co MCC pisze na stderr, dostaje przedrostek
`[stderr] `.

**Prywatność:** te pliki zawierają czat z serwera, w tym prywatne wiadomości przysłane przez
innych graczy (serwer dostarcza je do klienta). Nie opuszczają Twojego komputera — nic nie jest
nigdzie wysyłane, a folder to zwykły katalog, który możesz kopiować, przeszukiwać i kasować.
Wiersze promptu logowania (kod urządzenia i link) są odfiltrowane, zanim cokolwiek trafi na
dysk, więc żaden kod nie znajdzie się w żadnym pliku.

**Retencja:** ustawienie **Przechowuj logi czatu przez** (domyślnie 30 dni, `0` = bez limitu).
Czyszczenie działa przy starcie i raz na dobę i usuwa wyłącznie pliki o nazwie
`RRRR-MM-DD.log` na głębokości `<serwer>\<konto>\` (folder konta razem ze skrótem) — wszystko
inne, co trzymasz w folderze logów, zostaje nietknięte.

Zapis odbywa się w tle, więc wolny dysk nigdy nie zatrzyma aplikacji. Jeśli w jednej chwili
przyjdzie bardzo dużo linii, w pliku pojawi się pojedynczy wiersz
`N lines omitted (queue full)`. Błąd zapisu (np. pełny dysk) pokazuje jeden komunikat
i wyłącza zapis dla tego połączenia, zamiast powtarzać go w kółko.

## Gdzie trafiają Twoje dane

Wszystko leży w `%APPDATA%\com.mccgui.v2\`:

| Ścieżka | Zawartość |
|---------|-----------|
| `settings.json` | Ustawienia aplikacji: ścieżka MCC, język, odstęp „wyślij do wszystkich”, rozmiar bufora logu, przełącznik przywracania zakładek, ustawienia logów czatu, historia serwerów |
| `accounts.json` | Zapisane konta, każde z listą serwerów |
| `presets.json` | Przyciski szybkich akcji: etykieta, dokładna linia wysyłana do MCC, opcjonalny kolor i przełącznik wł./wył. per akcja |
| `session.json` | Zakładki otwarte w chwili zamknięcia aplikacji, przywracane przy następnym starcie |
| `logs\` | Logi czatu (patrz wyżej); folder można przenieść w Ustawieniach |
| `accounts\<id>\` | Katalog roboczy MCC dla konta: jeden generowany `.ini` na połączenie i własny cache sesji MCC |

Zainstalowana wersja i build deweloperski korzystają z tego samego katalogu, więc konta,
ustawienia i otwarte zakładki są w obu takie same.

**Aplikacja nigdy nie zapisuje haseł ani tokenów.** Konta Microsoft używają logowania kodem
urządzenia, a konta offline nie mają hasła w ogóle. Pliki sesji MCC zostają w katalogu konta —
aplikacja ich nie czyta, nie kopiuje i nie eksportuje.

## Warto wiedzieć

- **Logowanie Microsoft idzie przez kod urządzenia.** To, które konto Microsoft zostanie
  użyte, rozstrzyga sesja w Twojej przeglądarce, a nie adres wpisany w aplikacji — dla
  kolejnego konta otwórz link w oknie prywatnym albo w innym profilu przeglądarki. MCC sam
  też otwiera przeglądarkę.
- **MCC pamięta każde konto** po pierwszym udanym logowaniu, więc kod przepisujesz tylko raz
  na konto, a wszystkie serwery tego konta korzystają z tego samego logowania.
- **Konta offline** nie potrzebują żadnego logowania do Microsoftu.
- **Aplikacja nie wykonuje własnych połączeń sieciowych.** Wszystko działa offline poza tym,
  czego potrzebuje sam MCC.
- **Brakujące pliki nie zerują ustawień.** Jeśli zapisany `MinecraftClient.exe` zniknął
  (przeniosłeś folder), aplikacja to zgłasza, prosi o plik od nowa i zachowuje pozostałe
  ustawienia. Folder logów, do którego przy starcie nie da się pisać, wraca w ten sam sposób
  do folderu domyślnego — zmienia się tylko to jedno ustawienie.
- **Konto, którego nie udało się zapisać, nie jest dodane.** Jeśli `accounts.json` nie da się
  zapisać (np. nieczytelna kopia została odłożona przy starcie), formularz zostaje otwarty
  z Twoim wpisem i pokazuje powód, zamiast udawać, że konto zostało zapisane.
- **Nic więcej nie jest zbierane.** Żadnej telemetrii, statystyk ani raportów awarii.

## Znane ograniczenia

- Instalator **nie jest podpisany certyfikatem**, więc Windows pokaże ostrzeżenie SmartScreen
  przy pierwszym uruchomieniu.
- Obsługiwany jest tylko **Windows 11**.
- Potrzebujesz własnej kopii Minecraft Console Client — aplikacja jej nie zawiera.
- Wyszukiwania w logu, eksportu logów poza folder logów i powiadomień w tej wersji nie ma;
  zamiast tego rozmowy zapisują się do plików.

## Licencja i oprogramowanie osób trzecich

MCC GUI V2 jest wydane na **licencji MIT** — patrz [LICENSE](LICENSE).
Copyright (c) 2026 AnonBOTpl.

Komponenty osób trzecich, z których korzysta aplikacja, wraz z ich licencjami, są wypisane w
[THIRD-PARTY-NOTICES.md](THIRD-PARTY-NOTICES.md).

Projekt **nie jest powiązany z Mojang Studios, Microsoftem ani zespołem Minecraft Console
Client, nie jest przez nich sponsorowany ani rekomendowany**. „Minecraft” i „Mojang” to znaki
towarowe Mojang Studios. Aplikacja sama nie mówi protokołem Minecrafta — robi to przez MCC.

**Minecraft Console Client nie jest dołączony do aplikacji i nie jest przez nią
rozprowadzany.** MCC pobierasz sam ze strony
[mccteam.github.io](https://mccteam.github.io/guide/usage.html), a aplikacja tylko uruchamia
wskazany przez Ciebie `MinecraftClient.exe` jako osobny proces.

Sam MCC jest objęty licencją **CDDL 1.0**
([LICENSE.md w repozytorium MCC](https://github.com/MCCTeam/Minecraft-Console-Client/blob/master/LICENSE.md)).
To copyleft plikowy, ale jego obowiązki dotyczą plików źródłowych MCC, a ta aplikacja żadnego
z nich nie kopiuje, nie modyfikuje, nie linkuje i nie dołącza — rozmawia z MCC przez
standardowe wejście i wyjście, co licencja wprost dopuszcza („Larger Work"). Dlatego nic
tutaj nie jest objęte CDDL, a projekt może zostać na MIT.

## Dla deweloperów

Wszystko poniżej dotyczy pracy nad aplikacją, a nie korzystania z niej.

### Stan projektu

Wydane jako 2.0.0; wszystkie etapy poniżej są gotowe.

| Etap | Czego dotyczy | Status |
|------|---------------|--------|
| 0 | Szkielet, narzędzia, research MCC | gotowe |
| 1 | Menedżer procesów (Rust) | gotowe |
| 2 | Interfejs: zakładki, log, wpis | gotowe — 200 linii/s utrzymuje ~59 fps |
| 3 | Banner logowania Microsoft i obsługa statusów | gotowe |
| 4 | Trwałość: konta, ustawienia, serwery per konto | gotowe |
| 5 | Dopracowanie przewijania logu | gotowe (filtry logu są poza zakresem) |
| 5b | Niezawodność UI: log nie zamarza przy pełnym buforze | gotowe |
| 6 | Szybkie akcje | gotowe |
| 7 | Przełącznik języka, finalne tłumaczenie, pakowanie | gotowe |

### Środowisko

- **Node.js 24** i npm (wersja, na której projekt jest budowany i testowany).
- **Rust 1.90+** z toolchainem MSVC (`rustup default stable-msvc`). Crate używa edycji 2024.

### Komendy deweloperskie

```bash
npm install
npm run tauri dev
```

W pracy nad kodem możesz pominąć okno ustawień i podać plik zmienną `MCC_EXECUTABLE` —
zapisana ścieżka zawsze ma pierwszeństwo:

```bash
MCC_EXECUTABLE="C:/sciezka/do/MinecraftClient.exe" npm run tauri dev
```

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

### Architektura

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
- **Logi czatu** zapisuje osobne zadanie przez kolejkę o ograniczonej pojemności, więc
  czytanie wyjścia MCC nigdy się nie blokuje; zgubione linie są policzone i widoczne w pliku.
- **Autoprzewijanie jest per zakładka:** przewinięcie w górę w jednym połączeniu zatrzymuje
  tylko tę zakładkę, a każda pamięta własne ustawienie. Uruchomienie albo restart połączenia
  wznawia śledzenie, bo jego log startuje od zera.
- **Jedynym łączem** między Rustem a interfejsem jest kontrakt komend i zdarzeń Tauri
  opisany w `src/api/contract.ts`.
- **Frontend nigdy nie uruchamia procesów ani nie dotyka plików** — to wszystko dzieje się
  w Rustcie.

### Wydania

```bash
npm run tauri build
```

Plik wykonywalny i paczki lądują w `src-tauri/target/release/` oraz
`src-tauri/target/release/bundle/`.

```bash
npm run tauri -- build --bundles nsis
```

Powstaje instalator NSIS dla bieżącego użytkownika:
`src-tauri/target/release/bundle/nsis/MCC GUI V2_2.0.0_x64-setup.exe`. Nie potrzebuje praw
administratora i instaluje się w profilu użytkownika.

Numer wersji ma **jedno źródło: `src-tauri/Cargo.toml`** (`version = "2.0.0"`). Konfiguracja
Tauri celowo nie ma pola `version`, więc Tauri bierze numer z `Cargo.toml` — z niego wynikają
nazwa instalatora oraz metadane pliku wykonywalnego i paczki. `package.json` ma ten sam numer
tylko dlatego, że npm wymaga tam pola `version`; build go nie czyta. Przy wydaniu podnieś
numer w `Cargo.toml` i ustaw taki sam w `package.json`.

### Języki

Interfejs jest **domyślnie angielski** i startuje po angielsku przy pierwszym uruchomieniu,
niezależnie od języka systemu. Polski włącza się przełącznikiem w pasku tytułu, a wybór jest
zapamiętywany.

Każdy widoczny tekst przechodzi przez `vue-i18n`. `en` jest źródłem prawdy i wersją zapasową;
`pl` musi mieć wszystkie klucze, czego pilnuje `npm run check:i18n`. Rust nigdy nie wysyła do
interfejsu tekstów dla człowieka — wysyła stabilne kody błędów z parametrami, a frontend je
tłumaczy. Surowy czat i wyjście MCC są pokazywane dokładnie tak, jak przyszły, i nie są
tłumaczone.

### Struktura projektu

```
src/                 Frontend Vue 3 (TypeScript, Pinia, Tailwind, vue-i18n)
  api/               Typy kontraktu Tauri i opakowania invoke/listen
  components/        Komponenty UI (panel kont, pasek tytułu, zakładki, log, banner logowania, ...)
  log/               Parsowanie kolorów, bufor logu, zawijanie, zaznaczanie, statusy (z testami)
  presets/           Model szybkich akcji, kolejność i paleta kolorów (z testami)
  session/           Które zakładki były otwarte przy zamknięciu i jak wracają (z testami)
  stores/            Store'y Pinia (konta, logi, ustawienia)
  i18n/              Pliki językowe, tłumaczenie kodów błędów
src-tauri/           Backend w Rustcie
  src/chatlog.rs           Logi czatu: ścieżki, oczyszczanie nazw, retencja, kolejka zapisu
  src/connection.rs        Identyfikator połączenia i nazwa pliku ustawień per połączenie
  src/process_manager.rs   Jeden proces MCC na połączenie, sprzątanie przez Job Object
  src/device_code.rs       Wykrywanie promptu logowania Microsoft
  src/settings.rs          Model ustawień i walidacja
  src/store.rs             Atomowy zapis JSON
  src/bin/fake_mcc.rs      Atrapa MCC używana w testach automatycznych
  tests/                   Testy integracyjne na atrapie
scripts/             Skrypty repozytorium (sprawdzanie i18n, inspekcja UI)
```

### Materiał referencyjny

`reference/` zawiera tylko do odczytu źródła MCC oraz `MinecraftClient.exe` do lokalnych
testów. Katalog jest **w .gitignore**, wyłączony z TypeScript, ESLint, Vite i Vitest i nie
jest potrzebny do zbudowania ani uruchomienia aplikacji. Zachowania, na których opiera się
aplikacja, weryfikujemy na prawdziwym pliku wykonywalnym.

// Typy kontraktu Tauri - jedno źródło po stronie frontendu.
// Muszą pozostać zgodne z payloadami z Rusta: `src-tauri/src/events.rs`
// i `src-tauri/src/account.rs`. Rust serializuje pola jako camelCase,
// a status jako snake_case.

export type AccountType = 'microsoft' | 'offline'

export type AccountStatus =
  | 'stopped'
  | 'starting'
  | 'awaiting_login'
  | 'running'
  | 'stopping'
  | 'crashed'

/**
 * Serwer przypisany do konta. `version` wymusza wersję Minecrafta
 * (`MinecraftVersion` w ustawieniach połączenia); `null`/brak = `auto`, czyli
 * normalne wykrywanie przez MCC. Wymuszenie jest potrzebne, gdy serwer nie
 * odpowiada na ping statusu i MCC pyta w konsoli „Server version:".
 */
export interface ServerEntry {
  address: string
  version?: string | null
}

export interface Account {
  id: string
  label: string
  type: AccountType
  login: string
  /** Zapisane serwery konta (jeden login, wiele serwerów). */
  servers: ServerEntry[]
  tabOrder: number
}

/**
 * Połączenie = para (konto, serwer). Jedno połączenie = jeden proces MCC.
 * Wszystkie połączenia konta dzielą jego katalog, więc wspólną sesję MCC.
 */
export interface Connection {
  id: string
  accountId: string
  server: string
}

/**
 * Identyfikator połączenia. **Ta sama reguła jest w Rustcie**
 * (`src-tauri/src/connection.rs` → `connection_id`) - zmiana tutaj wymaga zmiany tam.
 */
export function connectionId(accountId: string, server: string): string {
  return `${accountId}::${server}`
}

/** Buduje obiekt połączenia dla pary (konto, serwer). */
export function toConnection(accountId: string, server: string): Connection {
  return { id: connectionId(accountId, server), accountId, server }
}

/**
 * Zakładka zapisana w sesji (`session.json`): okno połączenia, które było
 * otwarte przy zamknięciu aplikacji. Przywracamy **tylko okna** - procesy MCC
 * startują wyłącznie z decyzji użytkownika.
 */
export interface OpenTab {
  accountId: string
  server: string
}

/**
 * Kolory presetów szybkich akcji. **Ta sama lista jest w Rustcie**
 * (`src-tauri/src/preset.rs` → `PRESET_COLORS`) - backend odrzuca nazwę spoza
 * palety, więc zmiana tutaj wymaga zmiany tam.
 */
export const PRESET_COLORS = ['blue', 'green', 'amber', 'red', 'violet'] as const

export type PresetColor = (typeof PRESET_COLORS)[number]

/**
 * Preset szybkiej akcji. `label` widać na przycisku, a `text` idzie
 * dosłownie do `stdin` MCC - tak jakby został wpisany ręcznie. Dlatego obowiązują
 * go te same zasady (jedna linia, bez znaków sterujących).
 *
 * Zasięg wysyłki nie należy do presetu: bierze się z checkboxa „Wyślij do
 * wszystkich” przy polu wpisywania, wspólnego dla wiadomości i dla akcji.
 */
export interface Preset {
  id: string
  label: string
  text: string
  /** Kolor paska przy etykiecie; `null` = bez paska. */
  color: PresetColor | null
  /** Włączona akcja jest na pasku; wyłączona znika z paska i zostaje w edytorze. */
  enabled: boolean
}

export type LogStream = 'stdout' | 'stderr'

/** Payload zdarzenia `mcc://log`. */
export interface LogPayload {
  accountId: string
  stream: LogStream
  /** Linia bez terminatora, poza tym niezmieniona (kody `§` zostają). */
  line: string
  /** Czas odebrania linii, ms od epoki Unix. */
  ts: number
}

/** Payload zdarzenia `mcc://status`. */
export interface StatusPayload {
  accountId: string
  status: AccountStatus
}

/** Payload zdarzenia `app://error` - stabilny kod i parametry, tłumaczone w UI. */
export interface ErrorPayload {
  code: string
  params: Record<string, unknown>
}

/** Payload zdarzenia `mcc://device-code` - prompt logowania Microsoft. */
export interface DeviceCodePayload {
  accountId: string
  /** Adres do otwarcia (tylko `https://`, walidowany w Rustcie). */
  url: string
  /** Kod, który użytkownik przepisuje w przeglądarce. */
  code: string
}

/** Payload zdarzenia `mcc://joined` - klient połączył się z serwerem. */
export interface JoinedPayload {
  accountId: string
}

export const EVENTS = {
  log: 'mcc://log',
  status: 'mcc://status',
  error: 'app://error',
  deviceCode: 'mcc://device-code',
  joined: 'mcc://joined',
} as const

/**
 * Ustawienia aplikacji. Muszą zgadzać się z `AppSettings`
 * w `src-tauri/src/settings.rs` (pola `camelCase`).
 */
export interface AppSettings {
  /** Ścieżka do `MinecraftClient.exe`; `null` = jeszcze nie wybrana. */
  mccPath: string | null
  language: LocaleCode
  sendAllDelayMs: number
  logBufferSize: number
  /** Czy przywracać zapisane zakładki przy starcie. */
  restoreTabs: boolean
  /** Historia adresów serwerów, najnowsze pierwsze. */
  serverHistory: string[]
}

/**
 * Raport diagnostyczny UI (tylko przy `MCC_GUI_DIAG=1` - patrz `src-tauri/src/diag.rs`).
 *
 * Rust dokłada do niego własne liczniki i stan okna. Nie ma tu treści logów ani
 * niczego związanego z poświadczeniami - tylko liczby i czasy.
 */
export interface DiagReport {
  /** Ile ms minęło od poprzedniego raportu (dłużej niż 5 s = timer nie odpalał). */
  intervalMs: number
  /** Liczba klatek od poprzedniego raportu; `-1`, gdy środowisko nie ma `requestAnimationFrame`. */
  frames: number
  /** Najdłuższy odstęp między klatkami; `-1`, gdy brak `requestAnimationFrame`. */
  frameGapMaxMs: number
  /** Ile ms minęło od ostatniej klatki - rośnie, gdy klatki stanęły. */
  frameAgeMs: number
  visibility: DocumentVisibilityState
  focused: boolean
  /** Liczniki z `src/log/diag.ts`. */
  counters: {
    received: number
    appended: number
    flushFrame: number
    flushTimer: number
    flushNow: number
    wake: number
    renders: number
  }
  /** Stan widoku logu; `null`, gdy żaden widok nie jest zamontowany. */
  view: {
    bufferSize: number
    rowsInDom: number
    scrollHeight: number
    scrollTop: number
    clientHeight: number
    follow: boolean
  } | null
}

/** Kody języków obsługiwane przez backend (`en` jest źródłem prawdy). */
export type LocaleCode = 'en' | 'pl'

/** Konto tworzone w UI: `tabOrder` i `servers` nadaje store. */
export type NewAccount = Omit<Account, 'tabOrder' | 'servers'>

/** Adres serwera w zapisanej liście konta. */
export function serverAddresses(account: Account): string[] {
  return account.servers.map((entry) => entry.address)
}

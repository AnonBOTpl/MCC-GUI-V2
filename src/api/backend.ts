// Jedyne miejsce w frontendzie, które rozmawia z Tauri.
// Komendy to RPC do Rusta, zdarzenia to strumień z Rusta do UI.

import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import {
  EVENTS,
  type Account,
  type AppSettings,
  type CloseRequestedPayload,
  type DeviceCodePayload,
  type DiagReport,
  type ErrorPayload,
  type JoinedPayload,
  type LogPayload,
  type OpenTab,
  type Preset,
  type StatusPayload,
} from './contract'

/** Sprawdza, czy obiekt jest kształtem błędu z Rusta (`{ code, params }`). */
function asAppError(value: unknown): ErrorPayload | null {
  if (typeof value !== 'object' || value === null) return null
  const candidate = value as { code?: unknown; params?: unknown }
  if (typeof candidate.code !== 'string') return null
  return {
    code: candidate.code,
    params:
      typeof candidate.params === 'object' && candidate.params !== null
        ? (candidate.params as Record<string, unknown>)
        : {},
  }
}

/**
 * Wywołuje komendę Tauri. Rust odrzuca komendę stabilnym błędem `{ code, params }`;
 * jeśli odrzucenie nie ma tego kształtu (np. błąd samego Tauri), zamieniamy je na
 * kod `app.unexpectedError`, żeby UI nigdy nie dostało surowego tekstu.
 */
export async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(command, args)
  } catch (raw) {
    throw asAppError(raw) ?? { code: 'app.unexpectedError', params: {} }
  }
}

export const backend = {
  addAccount: (account: Account) => call<void>('add_account', { account }),
  /** Konta zapisane na dysku (`accounts.json`), bez uruchamiania procesów. */
  listAccounts: () => call<Account[]>('list_accounts'),
  /**
   * Rejestruje konta w Rustcie (tworzy ich katalogi) **bez zapisu pliku**.
   * Używane przy starcie: `accounts.json` jest już źródłem tej listy, więc jeden
   * niepoprawny wpis nie może jej nadpisać ani wyczyścić (R-01).
   */
  registerAccounts: (accounts: Account[]) =>
    call<void>('register_accounts', { accounts }),
  /**
   * Zapisuje całą listę kont i rejestruje je w menedżerze jako `stopped`
   * (bez uruchamiania procesów). Rust waliduje listę.
   */
  saveAccounts: (accounts: Account[]) => call<void>('save_accounts', { accounts }),
  removeAccount: (id: string) => call<void>('remove_account', { id }),
  /**
   * Dopisuje serwer do konta (bez uruchamiania procesu). Ten sam adres
   * aktualizuje wymuszoną wersję Minecrafta (`null` = `auto`).
   */
  addServer: (accountId: string, server: string, version: string | null) =>
    call<void>('add_server', { accountId, server, version }),
  /** Usuwa serwer z konta, zatrzymując jego połączenie, jeśli działa. */
  removeServer: (accountId: string, server: string) =>
    call<void>('remove_server', { accountId, server }),
  /** Uruchamia połączenie (konto + serwer). */
  startConnection: (accountId: string, server: string) =>
    call<void>('start_connection', { accountId, server }),
  /** Zatrzymuje połączenie po jego identyfikatorze (`konto::serwer`). */
  stopConnection: (id: string) => call<void>('stop_connection', { id }),
  /** Ponownie uruchamia połączenie. */
  restartConnection: (accountId: string, server: string) =>
    call<void>('restart_connection', { accountId, server }),
  sendInput: (id: string, text: string) => call<void>('send_input', { id, text }),
  sendInputAll: (text: string) => call<string[]>('send_input_all', { text }),
  /** Otwiera adres logowania w przeglądarce. Walidacja adresu jest w Rustcie. */
  openLoginUrl: (url: string) => call<void>('open_login_url', { url }),
  /** Kopiuje kod logowania do schowka (przez Rusta, bez okna uprawnień WebView2). */
  copyLoginCode: (code: string) => call<void>('copy_login_code', { code }),
  /**
   * Kopiuje zaznaczone wiersze logu do schowka (Paczka 3). Tekst waliduje Rust
   * (`src-tauri/src/clipboard.rs`): długość, znaki sterujące i końce linii.
   */
  copyText: (text: string) => call<void>('copy_text', { text }),
  /** Presety szybkich akcji z `presets.json` (start jest łagodny: brak pliku = pusta lista). */
  listPresets: () => call<Preset[]>('list_presets'),
  /**
   * Zapisuje presety; Rust waliduje je i zwraca wersję znormalizowaną, więc UI
   * nigdy nie pokazuje danych, których backend nie przyjął.
   */
  savePresets: (presets: Preset[]) => call<Preset[]>('save_presets', { presets }),
  /**
   * Wczytuje `presets.json` ponownie (przycisk `Reload`). W przeciwieństwie do
   * startu jest ścisłe: uszkodzony plik zwraca błąd `presets.readFailed`.
   */
  reloadPresets: () => call<Preset[]>('reload_presets'),
  /**
   * Zakładki z poprzedniej sesji (`session.json`). Rust odcedza wpisy bez konta
   * albo serwera, więc UI dostaje tylko zakładki, które da się otworzyć.
   */
  getOpenTabs: () => call<OpenTab[]>('get_open_tabs'),
  /**
   * Zapisuje otwarte zakładki. Zwraca listę po odcedzeniu - tak samo jak
   * `saveSettings` oddaje wersję znormalizowaną.
   */
  saveOpenTabs: (tabs: OpenTab[]) => call<OpenTab[]>('save_open_tabs', { tabs }),
  /**
   * Zamyka aplikację po potwierdzeniu przez użytkownika. Rust najpierw łagodnie
   * zatrzymuje połączenia (`/quit`), a dopiero potem zamyka okno.
   */
  closeApp: () => call<void>('close_app'),
  /** Bieżące ustawienia aplikacji (Rust wysyła je znormalizowane). */
  getSettings: () => call<AppSettings>('get_settings'),
  /** Zapisuje ustawienia; Rust waliduje je i zwraca wersję znormalizowaną. */
  saveSettings: (settings: AppSettings) => call<AppSettings>('save_settings', { settings }),
  /**
   * Otwiera natywne okno wyboru `MinecraftClient.exe`.
   * Zwraca wybraną, zwalidowaną ścieżkę albo `null`, gdy użytkownik anulował.
   */
  pickMccExecutable: () => call<string | null>('pick_mcc_executable'),
  /** Efektywny folder logów czatu (własny użytkownika albo domyślny). */
  chatLogDir: () => call<string>('chat_log_dir'),
  /**
   * Otwiera natywne okno wyboru folderu logów czatu. Tytuł okna przychodzi
   * z warstwy i18n (Rust nie trzyma tekstów dla człowieka), a ścieżkę bierze
   * z Rusta (walidacja: pełna ścieżka, folder do utworzenia, zapisywalny).
   * `null` oznacza anulowanie.
   */
  pickChatLogFolder: (title: string) => call<string | null>('pick_chat_log_folder', { title }),
  /** Otwiera folder logów czatu w Eksploratorze (ścieżka pochodzi z ustawień). */
  openChatLogFolder: () => call<void>('open_chat_log_folder'),
  /** Czy diagnostyka zamrożenia widoku logu jest włączona (`MCC_GUI_DIAG=1`). */
  diagEnabled: () => call<boolean>('diag_enabled'),
  /**
   * Dopisuje raport diagnostyczny do `diag.log` (opcjonalny, `MCC_GUI_DIAG=1`).
   * Bez treści logów - tylko liczniki, czasy i stan widoku.
   */
  diagReport: (report: DiagReport) => call<void>('diag_report', { report }),
}

export const events = {
  onLog: (handler: (payload: LogPayload) => void): Promise<UnlistenFn> =>
    listen<LogPayload>(EVENTS.log, (event) => handler(event.payload)),
  onStatus: (handler: (payload: StatusPayload) => void): Promise<UnlistenFn> =>
    listen<StatusPayload>(EVENTS.status, (event) => handler(event.payload)),
  onError: (handler: (payload: ErrorPayload) => void): Promise<UnlistenFn> =>
    listen<ErrorPayload>(EVENTS.error, (event) => handler(event.payload)),
  onDeviceCode: (handler: (payload: DeviceCodePayload) => void): Promise<UnlistenFn> =>
    listen<DeviceCodePayload>(EVENTS.deviceCode, (event) => handler(event.payload)),
  onJoined: (handler: (payload: JoinedPayload) => void): Promise<UnlistenFn> =>
    listen<JoinedPayload>(EVENTS.joined, (event) => handler(event.payload)),
  /**
   * Użytkownik zamknął okno, a działają połączenia: Rust wstrzymał zamknięcie
   * i czeka, aż UI zapyta. Odpowiedzią jest `backend.closeApp`.
   */
  onCloseRequested: (handler: (payload: CloseRequestedPayload) => void): Promise<UnlistenFn> =>
    listen<CloseRequestedPayload>(EVENTS.closeRequested, (event) => handler(event.payload)),
}

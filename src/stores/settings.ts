// Store ustawień: język interfejsu (lokalny) oraz ustawienia aplikacji z Rusta
// (ścieżka MCC, opóźnienie „wyślij do wszystkich", przywracanie zakładek).
//
// Ustawienia z Rusta wczytujemy raz na starcie (`load()`), a każda zmiana idzie
// przez `save()`, które wysyła cały obiekt - Rust waliduje i odsyła wersję
// znormalizowaną, więc UI nigdy nie trzyma wartości, której backend nie przyjął.

import { defineStore } from 'pinia'
import { ref } from 'vue'
import { backend } from '../api/backend'
import type { AppSettings, ErrorPayload } from '../api/contract'
import { DEFAULT_LOCALE, i18n, SUPPORTED_LOCALES, type Locale } from '../i18n'
import { useAccountsStore } from './accounts'
import { useLogsStore } from './logs'

const STORAGE_KEY = 'mcc.locale'

/** Wartości na czas przed pierwszym odczytem z Rusta. */
const FALLBACK_SETTINGS: AppSettings = {
  mccPath: null,
  language: 'en',
  sendAllDelayMs: 250,
  logBufferSize: 5000,
  restoreTabs: true,
  confirmClose: true,
  chatLogEnabled: true,
  chatLogDir: null,
  chatLogRetentionDays: 30,
  serverHistory: [],
}

function isLocale(value: unknown): value is Locale {
  return typeof value === 'string' && (SUPPORTED_LOCALES as readonly string[]).includes(value)
}

function readStoredLocale(): Locale {
  if (typeof localStorage === 'undefined') return DEFAULT_LOCALE
  const stored = localStorage.getItem(STORAGE_KEY)
  return isLocale(stored) ? stored : DEFAULT_LOCALE
}

/**
 * `<html lang>` mówi czytnikom ekranu i sprawdzaniu pisowni, w jakim języku jest
 * strona. Bez tego zostawało `en` z `index.html` także po przełączeniu na polski.
 */
function applyDocumentLang(next: Locale): void {
  if (typeof document === 'undefined') return
  document.documentElement.lang = next
}

export const useSettingsStore = defineStore('settings', () => {
  const locale = ref<Locale>(readStoredLocale())
  i18n.global.locale.value = locale.value
  // Język z poprzedniej sesji obowiązuje od pierwszego rysowania okna.
  applyDocumentLang(locale.value)

  const appSettings = ref<AppSettings>({ ...FALLBACK_SETTINGS })
  /** Czy udało się wczytać ustawienia z Rusta (potrzebne do pierwszego uruchomienia). */
  const loaded = ref(false)
  /**
   * Efektywny folder logów czatu wg Rusta (domyślny, gdy nie wybrano własnego).
   * Trzymamy go osobno, bo przy `chatLogDir = null` UI i tak ma pokazać ścieżkę.
   */
  const chatLogDir = ref<string | null>(null)

  function setLocale(next: Locale): void {
    locale.value = next
    i18n.global.locale.value = next
    applyDocumentLang(next)
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem(STORAGE_KEY, next)
    }
  }

  /** Błędy pokazujemy wspólnym mechanizmem toastów ze store kont. */
  function pushError(payload: ErrorPayload): void {
    useAccountsStore().pushError(payload)
  }

  /** Rozmiar bufora logu jest ustawieniem, ale działa po stronie store logów. */
  function applyLogBufferSize(settings: AppSettings): void {
    useLogsStore().setLimit(settings.logBufferSize)
  }

  /** Wczytuje ustawienia z Rusta. Wywoływane raz przy starcie aplikacji. */
  async function load(): Promise<void> {
    try {
      appSettings.value = await backend.getSettings()
      loaded.value = true
      applyLogBufferSize(appSettings.value)
      await loadChatLogDir()

      // Język z ustawień jest autorytatywny, jeśli lokalnie nic nie zapisano.
      if (typeof localStorage === 'undefined' || localStorage.getItem(STORAGE_KEY) === null) {
        if (isLocale(appSettings.value.language) && appSettings.value.language !== locale.value) {
          setLocale(appSettings.value.language)
        }
      }
    } catch (error) {
      pushError(error as ErrorPayload)
    }
  }

  /**
   * Zapis czeka na poprzedni. Bez tego dwa szybkie zapisy wysyłały dwie migawki
   * policzone z tej samej, starej wersji: drugi nadpisywał zmiany pierwszego
   * (R-37). Migawkę policzymy dopiero w chwili wysyłki, więc zawiera już wynik
   * poprzedniego zapisu.
   */
  let saveQueue: Promise<unknown> = Promise.resolve()

  /** Wysyła jedną migawkę i podstawia wersję znormalizowaną przez Rusta. */
  async function pushSettings(patch: Partial<AppSettings>): Promise<boolean> {
    try {
      appSettings.value = await backend.saveSettings({ ...appSettings.value, ...patch })
      applyLogBufferSize(appSettings.value)
      // Folder logów mógł się zmienić - pokazujemy efektywną ścieżkę z Rusta.
      await loadChatLogDir()
      return true
    } catch (error) {
      pushError(error as ErrorPayload)
      return false
    }
  }

  /** Zapisuje zmiany (częściowe) i podstawia wersję znormalizowaną z Rusta. */
  async function save(patch: Partial<AppSettings>): Promise<boolean> {
    const pending = saveQueue.then(() => pushSettings(patch))
    // Kolejka nie może paść po błędzie zapisu - inaczej następny zapis nigdy nie ruszy.
    saveQueue = pending.catch(() => undefined)
    return pending
  }

  /**
   * Dopisuje adres serwera do historii (najnowsze pierwsze, bez duplikatów).
   * Limit i deduplikację egzekwuje też `normalize` w Rustcie.
   */
  async function rememberServer(server: string): Promise<void> {
    const trimmed = server.trim()
    if (trimmed === '') return
    const history = [trimmed, ...appSettings.value.serverHistory.filter((item) => item !== trimmed)]
    await save({ serverHistory: history.slice(0, 30) })
  }

  /**
   * Zamyka aplikację po potwierdzeniu przez użytkownika (`close_app`); Rust
   * najpierw łagodnie zatrzymuje połączenia. `false` oznacza, że zamknięcie się
   * nie udało - okno zostaje otwarte, a błąd ląduje w toastach.
   */
  async function closeApp(): Promise<boolean> {
    try {
      await backend.closeApp()
      return true
    } catch (error) {
      pushError(error as ErrorPayload)
      return false
    }
  }

  /**
   * Otwiera natywny wybór pliku i zwraca zwalidowaną ścieżkę do MCC.
   * Nic nie zapisuje - zapis należy do modala ustawień.
   * `null` oznacza anulowanie albo błąd (błąd ląduje w toastach).
   */
  async function pickMccExecutable(): Promise<string | null> {
    try {
      return await backend.pickMccExecutable()
    } catch (error) {
      pushError(error as ErrorPayload)
      return null
    }
  }

  /** Odświeża efektywny folder logów czatu (po starcie i po każdej zmianie ustawień). */
  async function loadChatLogDir(): Promise<void> {
    try {
      chatLogDir.value = await backend.chatLogDir()
    } catch (error) {
      pushError(error as ErrorPayload)
    }
  }

  /**
   * Otwiera natywny wybór folderu logów czatu. Nic nie zapisuje - zapis należy
   * do modala ustawień. `null` oznacza anulowanie albo błąd (błąd ląduje w toastach).
   *
   * Tytuł okna podaje komponent z vue-i18n: natywny dialog nie ma własnej
   * warstwy tłumaczeń, a tekst widoczny dla użytkownika nie może stać w Rustcie (S-06).
   */
  async function pickChatLogFolder(title: string): Promise<string | null> {
    try {
      return await backend.pickChatLogFolder(title)
    } catch (error) {
      pushError(error as ErrorPayload)
      return null
    }
  }

  /** Otwiera folder logów czatu w Eksploratorze. */
  async function openChatLogFolder(): Promise<void> {
    try {
      await backend.openChatLogFolder()
    } catch (error) {
      pushError(error as ErrorPayload)
    }
  }

  return {
    locale,
    appSettings,
    loaded,
    chatLogDir,
    setLocale,
    load,
    save,
    closeApp,
    rememberServer,
    pickMccExecutable,
    loadChatLogDir,
    pickChatLogFolder,
    openChatLogFolder,
  }
})

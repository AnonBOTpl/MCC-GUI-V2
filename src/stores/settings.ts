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

export const useSettingsStore = defineStore('settings', () => {
  const locale = ref<Locale>(readStoredLocale())
  i18n.global.locale.value = locale.value

  const appSettings = ref<AppSettings>({ ...FALLBACK_SETTINGS })
  /** Czy udało się wczytać ustawienia z Rusta (potrzebne do pierwszego uruchomienia). */
  const loaded = ref(false)

  function setLocale(next: Locale): void {
    locale.value = next
    i18n.global.locale.value = next
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

  /** Zapisuje zmiany (częściowe) i podstawia wersję znormalizowaną z Rusta. */
  async function save(patch: Partial<AppSettings>): Promise<boolean> {
    try {
      appSettings.value = await backend.saveSettings({ ...appSettings.value, ...patch })
      applyLogBufferSize(appSettings.value)
      return true
    } catch (error) {
      pushError(error as ErrorPayload)
      return false
    }
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

  return {
    locale,
    appSettings,
    loaded,
    setLocale,
    load,
    save,
    rememberServer,
    pickMccExecutable,
  }
})

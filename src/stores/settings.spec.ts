import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { backend } from '../api/backend'
import type { AppSettings } from '../api/contract'
import { i18n } from '../i18n'
import { useSettingsStore } from './settings'

// Backend podmieniamy w całości: te testy sprawdzają kolejkowanie zapisów
// i język dokumentu, a nie Tauri.
vi.mock('../api/backend', () => ({
  backend: {
    getSettings: vi.fn(),
    saveSettings: vi.fn(),
    chatLogDir: vi.fn(),
  },
  events: {},
}))

const SETTINGS: AppSettings = {
  mccPath: 'C:/MCC/MinecraftClient.exe',
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

describe('store ustawień', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    localStorage.clear()
    document.documentElement.lang = 'en'
    i18n.global.locale.value = 'en'
    vi.mocked(backend.getSettings).mockResolvedValue(SETTINGS)
    vi.mocked(backend.saveSettings).mockImplementation(async (next) => next)
    vi.mocked(backend.chatLogDir).mockResolvedValue('C:/logs')
  })

  afterEach(() => {
    vi.clearAllMocks()
    localStorage.clear()
    document.documentElement.lang = 'en'
    i18n.global.locale.value = 'en'
  })

  /**
   * R-37: dwa zapisy uruchomione jeden po drugim (np. dwa szybkie kliknięcia)
   * wysyłały dwie migawki policzone z tej samej wersji, więc drugi gubił zmiany
   * pierwszego. Teraz migawka powstaje dopiero przy wysyłce, po poprzednim zapisie.
   */
  it('drugi zapis czeka na pierwszy i nie gubi jego zmian (R-37)', async () => {
    const settings = useSettingsStore()
    const sent: AppSettings[] = []
    let releaseFirst: (() => void) | undefined
    vi.mocked(backend.saveSettings).mockImplementation(async (next) => {
      sent.push(next)
      if (sent.length === 1) {
        await new Promise<void>((resolve) => {
          releaseFirst = resolve
        })
      }
      return next
    })

    const first = settings.save({ sendAllDelayMs: 100 })
    const second = settings.save({ logBufferSize: 9000 })

    // Dopóki pierwszy zapis nie wrócił, drugi nie wysyła niczego.
    await new Promise((resolve) => setTimeout(resolve, 0))
    expect(sent).toHaveLength(1)

    releaseFirst?.()
    await Promise.all([first, second])

    expect(sent).toHaveLength(2)
    // Druga migawka zawiera już zmianę z pierwszego zapisu.
    expect(sent[1].sendAllDelayMs).toBe(100)
    expect(sent[1].logBufferSize).toBe(9000)
    expect(settings.appSettings).toMatchObject({ sendAllDelayMs: 100, logBufferSize: 9000 })
  })

  it('nieudany zapis nie blokuje kolejki (R-37)', async () => {
    const settings = useSettingsStore()
    vi.mocked(backend.saveSettings)
      .mockRejectedValueOnce({ code: 'settings.writeFailed', params: {} })
      .mockImplementation(async (next) => next)

    const failed = settings.save({ sendAllDelayMs: 100 })
    const next = settings.save({ logBufferSize: 9000 })

    await expect(failed).resolves.toBe(false)
    await expect(next).resolves.toBe(true)
    expect(settings.appSettings.logBufferSize).toBe(9000)
  })

  /** R-28: `<html lang>` musi mówić, w jakim języku naprawdę jest interfejs. */
  it('przełączenie języka zmienia <html lang> (R-28)', () => {
    const settings = useSettingsStore()

    settings.setLocale('pl')
    expect(document.documentElement.lang).toBe('pl')
    expect(i18n.global.locale.value).toBe('pl')

    settings.setLocale('en')
    expect(document.documentElement.lang).toBe('en')
  })

  it('język z poprzedniej sesji obowiązuje od pierwszego rysowania (R-28)', () => {
    localStorage.setItem('mcc.locale', 'pl')

    const settings = useSettingsStore()

    expect(settings.locale).toBe('pl')
    expect(document.documentElement.lang).toBe('pl')
  })
})

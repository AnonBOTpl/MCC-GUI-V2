import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { backend } from '../api/backend'
import type { Preset } from '../api/contract'
import { useAccountsStore } from './accounts'
import { usePresetsStore } from './presets'

// Backend podmieniamy w całości: te testy sprawdzają logikę store'u, a nie Tauri.
vi.mock('../api/backend', () => ({
  backend: {
    listPresets: vi.fn(),
    savePresets: vi.fn(),
    reloadPresets: vi.fn(),
    sendInput: vi.fn(),
    sendInputAll: vi.fn(),
  },
  events: {},
}))

function preset(id: string, text: string, enabled = true): Preset {
  return { id, label: text, text, color: null, enabled }
}

/** Otwarte i aktywne połączenie - potrzebne, żeby było gdzie wysyłać. */
function openActiveConnection(): void {
  const accounts = useAccountsStore()
  accounts.connections = [{ id: 'acc::srv', accountId: 'acc', server: 'srv' }]
  accounts.activeConnectionId = 'acc::srv'
}

describe('store presetów', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.mocked(backend.listPresets).mockResolvedValue([])
    vi.mocked(backend.savePresets).mockImplementation(async (list) => list)
    vi.mocked(backend.reloadPresets).mockResolvedValue([])
    vi.mocked(backend.sendInput).mockResolvedValue(undefined)
    vi.mocked(backend.sendInputAll).mockResolvedValue([])
  })

  afterEach(() => {
    vi.useRealTimers()
    vi.clearAllMocks()
  })

  it('wczytuje plik i czyści wpisy z ręcznej edycji', async () => {
    vi.mocked(backend.listPresets).mockResolvedValue([
      { id: 'p1', label: '/home', text: '/send /home', color: 'magenta', enabled: false },
      { id: '', label: 'zepsuty', text: '/send /x', color: null, enabled: true },
    ] as unknown as Preset[])

    const presets = usePresetsStore()
    await presets.load()

    expect(presets.loaded).toBe(true)
    expect(presets.presets).toEqual([
      { id: 'p1', label: '/home', text: '/send /home', color: null, enabled: false },
    ])
  })

  it('raportuje błąd wczytania i nadal kończy start', async () => {
    vi.mocked(backend.listPresets).mockRejectedValue({ code: 'presets.readFailed', params: {} })

    const presets = usePresetsStore()
    const accounts = useAccountsStore()
    await presets.load()

    expect(presets.loaded).toBe(true)
    expect(accounts.errors.map((error) => error.code)).toEqual(['presets.readFailed'])
  })

  it('po zapisie podstawia listę znormalizowaną przez Rusta', async () => {
    const normalized = [preset('p1', '/send /home')]
    vi.mocked(backend.savePresets).mockResolvedValue(normalized)

    const presets = usePresetsStore()
    const draft = [{ ...preset('p1', '/send /home'), label: '  /home  ' }]
    const saved = await presets.save(draft)

    expect(saved).toBe(true)
    expect(backend.savePresets).toHaveBeenCalledWith(draft)
    expect(presets.presets).toEqual(normalized)
  })

  it('przy błędzie zapisu zostawia poprzednią listę i pokazuje błąd', async () => {
    vi.mocked(backend.savePresets).mockRejectedValue({ code: 'presets.duplicateId', params: {} })

    const presets = usePresetsStore()
    presets.presets = [preset('p1', '/send /home')]
    const saved = await presets.save([preset('x', '/send /x'), preset('x', '/send /x')])

    expect(saved).toBe(false)
    expect(presets.presets).toEqual([preset('p1', '/send /home')])
    expect(useAccountsStore().errors.map((error) => error.code)).toEqual(['presets.duplicateId'])
  })

  it('Reload wczytuje plik ponownie i potwierdza liczbę akcji', async () => {
    vi.mocked(backend.reloadPresets).mockResolvedValue([
      preset('p1', '/send /home'),
      preset('p2', '/send /bal'),
    ])

    const presets = usePresetsStore()
    const reloaded = await presets.reload()

    expect(reloaded).toBe(true)
    expect(presets.presets).toHaveLength(2)
    expect(presets.notice).toEqual({ kind: 'reloaded', count: 2 })
  })

  it('Reload zgłasza zepsuty plik, ale nie czyści paska', async () => {
    vi.mocked(backend.reloadPresets).mockRejectedValue({ code: 'presets.readFailed', params: {} })

    const presets = usePresetsStore()
    presets.presets = [preset('p1', '/send /home')]
    const reloaded = await presets.reload()

    expect(reloaded).toBe(false)
    expect(presets.presets).toEqual([preset('p1', '/send /home')])
    expect(presets.notice).toBeNull()
    expect(useAccountsStore().errors.map((error) => error.code)).toEqual(['presets.readFailed'])
  })

  it('klik presetu wysyła tekst do aktywnego połączenia', async () => {
    openActiveConnection()
    const presets = usePresetsStore()

    await presets.run(preset('p1', '/send /home'))

    expect(backend.sendInput).toHaveBeenCalledWith('acc::srv', '/send /home')
    expect(backend.sendInputAll).not.toHaveBeenCalled()
    expect(presets.notice).toBeNull()
  })

  it('bez zaznaczonego „Wyślij do wszystkich” preset nie idzie do wszystkich', async () => {
    openActiveConnection()
    const presets = usePresetsStore()

    await presets.run(preset('p1', '/send /home'))

    expect(backend.sendInputAll).not.toHaveBeenCalled()
  })

  it('bez aktywnego połączenia nic nie wysyła i mówi o tym', async () => {
    const presets = usePresetsStore()

    await presets.run(preset('p1', '/send /home'))

    expect(backend.sendInput).not.toHaveBeenCalled()
    expect(presets.notice).toEqual({ kind: 'noActive' })
  })

  it('przy zaznaczonym „Wyślij do wszystkich” preset idzie do wszystkich', async () => {
    vi.mocked(backend.sendInputAll).mockResolvedValue(['a::1', 'b::2'])
    const accounts = useAccountsStore()
    accounts.sendToAll = true
    const presets = usePresetsStore()

    await presets.run(preset('p1', '/send /reco'))

    expect(backend.sendInputAll).toHaveBeenCalledWith('/send /reco')
    expect(backend.sendInput).not.toHaveBeenCalled()
    expect(presets.notice).toBeNull()
  })

  it('gdy nic nie działa, komunikat zamiast cichego niepowodzenia', async () => {
    const accounts = useAccountsStore()
    accounts.sendToAll = true
    const presets = usePresetsStore()

    await presets.run(preset('p1', '/send /reco'))

    expect(presets.notice).toEqual({ kind: 'noneRunning' })
  })

  it('akcja wyłączona nie wysyła niczego', async () => {
    openActiveConnection()
    const presets = usePresetsStore()

    await presets.run(preset('p1', '/send /home', false))

    expect(backend.sendInput).not.toHaveBeenCalled()
    expect(backend.sendInputAll).not.toHaveBeenCalled()
    expect(presets.notice).toBeNull()
  })

  it('komunikat znika sam po kilku sekundach', async () => {
    vi.useFakeTimers()
    const presets = usePresetsStore()

    await presets.run(preset('p1', '/send /home'))
    expect(presets.notice).toEqual({ kind: 'noActive' })

    vi.advanceTimersByTime(3000)
    expect(presets.notice).toBeNull()
  })
})

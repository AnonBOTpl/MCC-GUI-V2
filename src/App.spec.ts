import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { i18n } from './i18n'
import { backend } from './api/backend'
import type { Account, AppSettings } from './api/contract'
import App from './App.vue'

// Ekran główny dotyka sieci tylko przez `backend` - w testach podstawiamy całość,
// żeby żadne wywołanie nie próbowało rozmawiać z Tauri.
vi.mock('./api/backend', () => {
  const unlisten = (): Promise<() => void> => Promise.resolve(() => {})
  return {
    backend: {
      getSettings: vi.fn(),
      chatLogDir: vi.fn(),
      listAccounts: vi.fn(),
      registerAccounts: vi.fn(),
      getOpenTabs: vi.fn(),
      listPresets: vi.fn(),
      saveAccounts: vi.fn(),
      saveOpenTabs: vi.fn(),
      saveSettings: vi.fn(),
      savePresets: vi.fn(),
    },
    events: {
      onLog: vi.fn(unlisten),
      onStatus: vi.fn(unlisten),
      onError: vi.fn(unlisten),
      onDeviceCode: vi.fn(unlisten),
      onJoined: vi.fn(unlisten),
      onCloseRequested: vi.fn(unlisten),
    },
  }
})

// Pasek tytułu rozmawia z oknem Tauri; w jsdom nie ma tych API.
vi.mock('@tauri-apps/api/window', () => ({
  getCurrentWindow: () => ({
    minimize: vi.fn(async () => {}),
    close: vi.fn(async () => {}),
    toggleMaximize: vi.fn(async () => {}),
    isMaximized: vi.fn(async () => false),
    onResized: vi.fn(async () => () => {}),
  }),
}))

const SETTINGS: AppSettings = {
  mccPath: 'C:/mcc/MinecraftClient.exe',
  language: 'en',
  sendAllDelayMs: 200,
  logBufferSize: 5000,
  restoreTabs: true,
  confirmClose: true,
  chatLogEnabled: true,
  chatLogDir: null,
  chatLogRetentionDays: 30,
  serverHistory: [],
}

const ACCOUNT: Account = {
  id: 'acc-1',
  label: 'AnonBOT',
  type: 'offline',
  login: 'AnonBOT',
  servers: [{ address: 'play.example.com', version: null }],
  tabOrder: 0,
}

describe('App - pusty stan i panel zakładki', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.stubGlobal(
      'ResizeObserver',
      class {
        observe(): void {}
        unobserve(): void {}
        disconnect(): void {}
      },
    )
    vi.mocked(backend.getSettings).mockResolvedValue(SETTINGS)
    vi.mocked(backend.chatLogDir).mockResolvedValue('C:/logs')
    vi.mocked(backend.listPresets).mockResolvedValue([])
    vi.mocked(backend.listAccounts).mockResolvedValue([])
    vi.mocked(backend.getOpenTabs).mockResolvedValue([])
    vi.mocked(backend.registerAccounts).mockResolvedValue(undefined)
    vi.mocked(backend.saveAccounts).mockResolvedValue(undefined)
    vi.mocked(backend.saveOpenTabs).mockResolvedValue([])
  })

  async function mountApp() {
    const wrapper = mount(App, { global: { plugins: [i18n] } })
    await flushPromises()
    await flushPromises()
    return wrapper
  }

  it('bez kont podpowiada dodanie konta (R-29)', async () => {
    const wrapper = await mountApp()
    expect(wrapper.text()).toContain(i18n.global.t('empty.hint'))
    expect(wrapper.text()).not.toContain(i18n.global.t('empty.chooseHint'))
  })

  it('z kontami, ale bez zakładki, podpowiada kliknięcie serwera (R-29)', async () => {
    vi.mocked(backend.listAccounts).mockResolvedValue([ACCOUNT])
    const wrapper = await mountApp()

    expect(wrapper.text()).toContain(i18n.global.t('empty.chooseTitle'))
    expect(wrapper.text()).toContain(i18n.global.t('empty.chooseHint'))
    expect(wrapper.text()).not.toContain(i18n.global.t('empty.hint'))
  })

  it('otwarta zakładka ma panel ARIA powiązany ze swoją kartą (R-27)', async () => {
    vi.mocked(backend.listAccounts).mockResolvedValue([ACCOUNT])
    vi.mocked(backend.getOpenTabs).mockResolvedValue([{ accountId: 'acc-1', server: 'play.example.com' }])
    const wrapper = await mountApp()

    const connectionId = 'acc-1::play.example.com'
    const panel = wrapper.get('[role="tabpanel"]')
    expect(panel.attributes('id')).toBe(`panel-${connectionId}`)
    expect(panel.attributes('aria-labelledby')).toBe(`tab-${connectionId}`)

    const tab = wrapper.get(`[role="tab"][id="tab-${connectionId}"]`)
    expect(tab.attributes('aria-controls')).toBe(`panel-${connectionId}`)
    expect(tab.attributes('aria-selected')).toBe('true')
  })
})

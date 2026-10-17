import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { nextTick } from 'vue'
import { i18n } from '../i18n'
import { backend } from '../api/backend'
import type { AppSettings } from '../api/contract'
import CloseConfirmModal from './CloseConfirmModal.vue'

// Modal rozmawia wyłącznie ze storem ustawień, a ten z backendem - podmieniamy
// backend w całości, żeby żaden test nie próbował wołać Tauri.
vi.mock('../api/backend', () => ({
  backend: {
    closeApp: vi.fn(),
    saveSettings: vi.fn(),
  },
  events: {},
}))

/** Ustawienia w wersji, jaką oddaje Rust po zapisie. */
const SAVED_SETTINGS: AppSettings = {
  mccPath: null,
  language: 'en',
  sendAllDelayMs: 250,
  logBufferSize: 5000,
  restoreTabs: true,
  confirmClose: false,
  chatLogEnabled: true,
  chatLogDir: null,
  chatLogRetentionDays: 30,
  serverHistory: [],
}

describe('CloseConfirmModal', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.mocked(backend.closeApp).mockResolvedValue(undefined)
    vi.mocked(backend.saveSettings).mockResolvedValue(SAVED_SETTINGS)
  })

  afterEach(() => {
    vi.clearAllMocks()
    document.body.innerHTML = ''
  })

  /** Modal w aplikacji jest zamontowany na stałe i dopiero otwierany. */
  async function openModal(active: number) {
    const wrapper = mount(CloseConfirmModal, {
      props: { open: false, active },
      global: { plugins: [i18n] },
      attachTo: document.body,
    })
    await wrapper.setProps({ open: true, active })
    await nextTick()
    return wrapper
  }

  it('zamknięty nic nie pokazuje', () => {
    const wrapper = mount(CloseConfirmModal, {
      props: { open: false, active: 2 },
      global: { plugins: [i18n] },
    })

    expect(wrapper.text()).toBe('')
  })

  it('pokazuje liczbę połączeń i odmienia ją po angielsku', async () => {
    expect((await openModal(1)).text()).toContain('1 connection')
    expect((await openModal(3)).text()).toContain('3 connections')
  })

  it('Esc zostawia okno otwarte i nic nie zapisuje', async () => {
    const wrapper = await openModal(2)

    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }))
    await nextTick()

    expect(wrapper.emitted('close')).toHaveLength(1)
    expect(backend.closeApp).not.toHaveBeenCalled()
    expect(backend.saveSettings).not.toHaveBeenCalled()
  })

  it('klik poza modalem zostawia okno otwarte', async () => {
    const wrapper = await openModal(2)

    await wrapper.get('.fixed').trigger('click')

    expect(wrapper.emitted('close')).toHaveLength(1)
    expect(backend.closeApp).not.toHaveBeenCalled()
  })

  it('potwierdzenie zamyka aplikację bez zapisywania ustawienia', async () => {
    const wrapper = await openModal(2)

    await wrapper.findAll('button').at(-1)?.trigger('click')
    await flushPromises()

    expect(backend.closeApp).toHaveBeenCalledTimes(1)
    expect(backend.saveSettings).not.toHaveBeenCalled()
  })

  it('„nie pytaj więcej” zapisuje ustawienie **przed** zamknięciem', async () => {
    // Kolejność sprawdzamy własnym licznikiem, bo to jedyny sposób, żeby
    // porównać wywołania dwóch różnych atrap.
    const calls: string[] = []
    vi.mocked(backend.saveSettings).mockImplementation(async () => {
      calls.push('saveSettings')
      return SAVED_SETTINGS
    })
    vi.mocked(backend.closeApp).mockImplementation(async () => {
      calls.push('closeApp')
    })

    const wrapper = await openModal(2)
    await wrapper.get('input[type="checkbox"]').setValue(true)

    await wrapper.findAll('button').at(-1)?.trigger('click')
    await flushPromises()

    expect(vi.mocked(backend.saveSettings).mock.calls[0][0]).toMatchObject({ confirmClose: false })
    expect(calls).toEqual(['saveSettings', 'closeApp'])
  })

  it('ma semantykę dialogu, a Tab nie wypuszcza fokusu poza modal', async () => {
    const wrapper = await openModal(2)
    // Pole wpisywania komend jest poza modalem: bez pułapki fokusu `Tab` do niego
    // wracał, a `Enter` w tym formularzu wysyłał wiadomość pod nakładką.
    const inputBar = document.createElement('input')
    inputBar.type = 'text'
    document.body.append(inputBar)

    const dialog = wrapper.get('[role="dialog"]')
    expect(dialog.attributes('aria-modal')).toBe('true')
    expect(dialog.element.contains(document.activeElement)).toBe(true)

    inputBar.focus()
    document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Tab', bubbles: true }))

    expect(document.activeElement).not.toBe(inputBar)
    expect(dialog.element.contains(document.activeElement)).toBe(true)
  })

  it('nieudane zamknięcie zostawia modal otwarty i pozwala spróbować ponownie', async () => {
    vi.mocked(backend.closeApp).mockRejectedValue({ code: 'app.closeFailed', params: {} })
    const wrapper = await openModal(1)

    await wrapper.findAll('button').at(-1)?.trigger('click')
    await flushPromises()
    await nextTick()

    const again = wrapper.findAll('button').at(-1)
    expect(again?.attributes('disabled')).toBeUndefined()
  })
})

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { nextTick } from 'vue'
import { i18n } from '../i18n'
import { backend } from '../api/backend'
import SettingsModal from './SettingsModal.vue'

// Ustawienia czytają z Rusta tylko efektywny folder logów czatu - reszta idzie
// przez store, a store przez podmieniony backend.
vi.mock('../api/backend', () => ({
  backend: {
    chatLogDir: vi.fn(),
    pickMccExecutable: vi.fn(),
    pickChatLogFolder: vi.fn(),
    openChatLogFolder: vi.fn(),
    saveSettings: vi.fn(),
  },
  events: {},
}))

describe('SettingsModal', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.mocked(backend.chatLogDir).mockResolvedValue('C:/logs')
  })

  afterEach(() => {
    vi.clearAllMocks()
    document.body.innerHTML = ''
  })

  async function openModal() {
    const wrapper = mount(SettingsModal, {
      props: { open: false },
      global: { plugins: [i18n] },
      attachTo: document.body,
    })
    await wrapper.setProps({ open: true })
    await nextTick()
    await nextTick()
    return wrapper
  }

  it('jest dialogiem z nazwą, a fokus siedzi w panelu (R-06)', async () => {
    const wrapper = await openModal()

    const dialog = wrapper.get('[role="dialog"]')
    expect(dialog.attributes('aria-modal')).toBe('true')
    expect(dialog.attributes('aria-label')).toBe('Settings')
    expect(dialog.element.contains(document.activeElement)).toBe(true)

    // `Tab` z ostatniego elementu nie ucieka do paska tytułu pod nakładką.
    const outside = document.createElement('button')
    document.body.append(outside)
    outside.focus()
    document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Tab', bubbles: true }))
    expect(dialog.element.contains(document.activeElement)).toBe(true)
  })
})

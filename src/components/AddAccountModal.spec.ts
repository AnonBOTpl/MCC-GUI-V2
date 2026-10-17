import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { nextTick } from 'vue'
import { i18n } from '../i18n'
import { backend } from '../api/backend'
import { useAccountsStore } from '../stores/accounts'
import AddAccountModal from './AddAccountModal.vue'

// Modal tworzy konto przez store, a store woła backend - podmieniamy go w całości,
// żeby żaden test nie próbował rozmawiać z Tauri.
vi.mock('../api/backend', () => ({
  backend: {
    addAccount: vi.fn(),
    saveAccounts: vi.fn(),
  },
  events: {},
}))

describe('AddAccountModal', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.mocked(backend.addAccount).mockResolvedValue(undefined)
    vi.mocked(backend.saveAccounts).mockResolvedValue(undefined)
  })

  afterEach(() => {
    vi.clearAllMocks()
    document.body.innerHTML = ''
  })

  /** Modal w aplikacji jest zamontowany na stałe i dopiero otwierany. */
  async function openModal() {
    const wrapper = mount(AddAccountModal, {
      props: { open: false },
      global: { plugins: [i18n] },
      attachTo: document.body,
    })
    await wrapper.setProps({ open: true })
    await nextTick()
    return wrapper
  }

  const loginField = (wrapper: Awaited<ReturnType<typeof openModal>>) =>
    wrapper.get('input[type="text"]')

  it('zamknięty nic nie pokazuje', () => {
    const wrapper = mount(AddAccountModal, {
      props: { open: false },
      global: { plugins: [i18n] },
    })

    expect(wrapper.find('form').exists()).toBe(false)
  })

  it('jest dialogiem z nazwą, ale fokus zostaje w polu loginu (R-06)', async () => {
    const wrapper = await openModal()

    const dialog = wrapper.get('[role="dialog"]')
    expect(dialog.attributes('aria-modal')).toBe('true')
    expect(dialog.attributes('aria-label')).toBe('Add account')
    expect(document.activeElement).toBe(loginField(wrapper).element)
  })

  it('po otwarciu ustawia fokus na loginie i blokuje zapis do czasu wpisania loginu', async () => {
    const wrapper = await openModal()

    expect(document.activeElement).toBe(loginField(wrapper).element)

    const submit = wrapper.get('button[type="submit"]')
    expect(submit.attributes('disabled')).toBeDefined()

    // Sam biały znak to nie login.
    await loginField(wrapper).setValue('   ')
    expect(submit.attributes('disabled')).toBeDefined()

    await loginField(wrapper).setValue('bot@example.com')
    expect(submit.attributes('disabled')).toBeUndefined()
  })

  it('konto Microsoft bez nazwy dostaje login jako nazwę', async () => {
    const wrapper = await openModal()

    await loginField(wrapper).setValue('  bot@example.com  ')
    await wrapper.get('form').trigger('submit')
    await flushPromises()

    expect(backend.addAccount).toHaveBeenCalledTimes(1)
    const account = vi.mocked(backend.addAccount).mock.calls[0][0]
    expect(account.login).toBe('bot@example.com')
    expect(account.label).toBe('bot@example.com')
    expect(account.type).toBe('microsoft')
    expect(account.id).not.toBe('')
    expect(account.servers).toEqual([])
    // Zapis kont na dysk idzie przez store.
    expect(backend.saveAccounts).toHaveBeenCalledTimes(1)
    // Po dodaniu modal się zamyka.
    expect(wrapper.emitted('close')).toHaveLength(1)
  })

  it('konto offline: podpowiedź to ksywka, a wpisana nazwa wygrywa z loginem', async () => {
    const wrapper = await openModal()

    await wrapper.get('[role="radio"][aria-checked="false"]').trigger('click')
    await nextTick()
    expect(loginField(wrapper).attributes('placeholder')).toBe('PlayerName')

    const nameField = wrapper.findAll('input[type="text"]')[1]
    await loginField(wrapper).setValue('Bot123')
    await nameField.setValue('Kopalnia')
    await wrapper.get('form').trigger('submit')
    await flushPromises()

    const account = vi.mocked(backend.addAccount).mock.calls[0][0]
    expect(account.type).toBe('offline')
    expect(account.login).toBe('Bot123')
    expect(account.label).toBe('Kopalnia')
  })

  it('dwa konta pod rząd dostają różne identyfikatory', async () => {
    const wrapper = await openModal()

    await loginField(wrapper).setValue('pierwsze@example.com')
    await wrapper.get('form').trigger('submit')
    await flushPromises()
    await wrapper.setProps({ open: true })

    await loginField(wrapper).setValue('drugie@example.com')
    await wrapper.get('form').trigger('submit')
    await flushPromises()

    const ids = vi.mocked(backend.addAccount).mock.calls.map((call) => call[0].id)
    expect(ids).toHaveLength(2)
    expect(new Set(ids).size).toBe(2)
  })

  it('Esc zamyka bez dodawania konta', async () => {
    const wrapper = await openModal()

    await loginField(wrapper).setValue('bot@example.com')
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }))
    await nextTick()

    expect(wrapper.emitted('close')).toHaveLength(1)
    expect(backend.addAccount).not.toHaveBeenCalled()
  })

  it('gdy Rust odrzuci konto, modal zostaje otwarty i pokazuje błąd', async () => {
    vi.mocked(backend.addAccount).mockRejectedValue({ code: 'account.alreadyExists', params: {} })
    const wrapper = await openModal()

    await loginField(wrapper).setValue('bot@example.com')
    await wrapper.get('form').trigger('submit')
    await flushPromises()

    expect(wrapper.emitted('close')).toBeUndefined()
    // Toast opisuje kod błędu z Rusta, więc żaden surowy tekst nie idzie do UI.
    expect(useAccountsStore().errors[0]?.code).toBe('account.alreadyExists')
  })
})

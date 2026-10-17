import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { i18n } from '../i18n'
import { backend } from '../api/backend'
import type { Account, AccountStatus } from '../api/contract'
import { useAccountsStore } from '../stores/accounts'
import AccountSidebar from './AccountSidebar.vue'

// Panel czyta tylko store'y, ale przycisk stopu woła backend - więc podmieniamy go
// w całości, żeby żaden test nie próbował rozmawiać z Tauri.
vi.mock('../api/backend', () => ({
  backend: {
    stopConnection: vi.fn(),
    startConnection: vi.fn(),
    saveOpenTabs: vi.fn(),
  },
  events: {},
}))

const ACCOUNT: Account = {
  id: 'acc-1',
  label: 'AnonBOT',
  type: 'microsoft',
  login: 'bot@example.com',
  servers: [{ address: 'play.example.com', version: '1.21' }],
  tabOrder: 0,
}

const CONNECTION = 'acc-1::play.example.com'

describe('AccountSidebar - przycisk połączenia', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.mocked(backend.stopConnection).mockResolvedValue(undefined)
    vi.mocked(backend.startConnection).mockResolvedValue(undefined)
    vi.mocked(backend.saveOpenTabs).mockResolvedValue([])
  })

  afterEach(() => {
    vi.clearAllMocks()
  })

  function mountSidebar(status: AccountStatus) {
    const accounts = useAccountsStore()
    accounts.accounts = [ACCOUNT]
    accounts.connections = [{ id: CONNECTION, accountId: 'acc-1', server: 'play.example.com' }]
    accounts.statuses = { [CONNECTION]: status }
    return mount(AccountSidebar, { global: { plugins: [i18n] } })
  }

  it('zatrzymany serwer ma zieloną strzałkę startu', () => {
    const wrapper = mountSidebar('stopped')

    const start = wrapper.get(`button[title="${i18n.global.t('account.start')}"]`)
    expect(start.classes()).toContain('text-green-400')
    expect(wrapper.findAll(`button[title="${i18n.global.t('account.disconnect')}"]`)).toHaveLength(0)
  })

  it('połączony serwer ma czerwony kwadrat, który rozłącza zamiast startować', async () => {
    const wrapper = mountSidebar('running')

    const stop = wrapper.get(`button[title="${i18n.global.t('account.disconnect')}"]`)
    expect(stop.classes()).toContain('text-red-400')
    expect(stop.find('svg').exists()).toBe(true)
    expect(wrapper.findAll(`button[title="${i18n.global.t('account.start')}"]`)).toHaveLength(0)

    await stop.trigger('click')
    await flushPromises()

    expect(backend.stopConnection).toHaveBeenCalledWith(CONNECTION)
    expect(backend.startConnection).not.toHaveBeenCalled()
  })

  it('startujący i czekający na login też pokazują kwadrat (można przerwać)', () => {
    for (const status of ['starting', 'awaiting_login'] as AccountStatus[]) {
      const wrapper = mountSidebar(status)
      expect(
        wrapper.findAll(`button[title="${i18n.global.t('account.disconnect')}"]`),
      ).toHaveLength(1)
      wrapper.unmount()
    }
  })

  it('w trakcie zatrzymywania kwadrat jest nieaktywny', () => {
    const wrapper = mountSidebar('stopping')

    const stop = wrapper.get(`button[title="${i18n.global.t('account.disconnect')}"]`)
    expect(stop.attributes('disabled')).toBeDefined()
  })

  it('po awarii znowu jest start, nie stop', () => {
    const wrapper = mountSidebar('crashed')

    const start = wrapper.get(`button[title="${i18n.global.t('account.start')}"]`)
    expect(start.classes()).toContain('text-green-400')
    expect(wrapper.findAll(`button[title="${i18n.global.t('account.disconnect')}"]`)).toHaveLength(0)
  })

  /**
   * Regresja z testu ręcznego B: po pierwszym kliknięciu Start w tym samym
   * miejscu stoi już kwadrat rozłączania, więc drugi klik dwukliku zatrzymywał
   * świeżo uruchomione połączenie.
   */
  it('drugi klik dwukliku nie zatrzymuje świeżo uruchomionego połączenia', async () => {
    const wrapper = mountSidebar('stopped')

    await wrapper.get(`button[title="${i18n.global.t('account.start')}"]`).trigger('click')
    await flushPromises()

    wrapper
      .get(`button[title="${i18n.global.t('account.disconnect')}"]`)
      .element.dispatchEvent(new MouseEvent('click', { bubbles: true, detail: 2 }))
    await flushPromises()

    expect(backend.startConnection).toHaveBeenCalledTimes(1)
    expect(backend.stopConnection).not.toHaveBeenCalled()
  })
})

describe('AccountSidebar - dodawanie konta', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('przycisk w nagłówku panelu prosi o otwarcie modala', async () => {
    const wrapper = mount(AccountSidebar, { global: { plugins: [i18n] } })

    const button = wrapper.findAll('button').find((item) => item.text().includes('Add account'))
    expect(button).toBeDefined()

    await button!.trigger('click')

    expect(wrapper.emitted('addAccount')).toHaveLength(1)
  })

  it('pusty panel odsyła do przycisku w nagłówku', () => {
    const wrapper = mount(AccountSidebar, { global: { plugins: [i18n] } })

    expect(wrapper.text()).toContain('No accounts yet')
    expect(wrapper.text()).toContain('“Add account”')
  })
})

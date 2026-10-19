import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { i18n } from '../i18n'
import { backend } from '../api/backend'
import type { Account, AccountStatus } from '../api/contract'
import { useAccountsStore } from '../stores/accounts'
import AccountTabs from './AccountTabs.vue'

// Zakładki wołają backend przy zamykaniu - podmieniamy go w całości.
vi.mock('../api/backend', () => ({
  backend: {
    stopConnection: vi.fn(),
    saveOpenTabs: vi.fn(),
  },
  events: {},
}))

const ACCOUNT: Account = {
  id: 'acc-1',
  label: 'AnonBOT',
  type: 'microsoft',
  login: 'bot@example.com',
  servers: [{ address: 'play.example.com', version: null }],
  tabOrder: 0,
}

const CONNECTION = 'acc-1::play.example.com'

describe('AccountTabs - zamykanie zakładki (R-44)', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.mocked(backend.stopConnection).mockResolvedValue(undefined)
    vi.mocked(backend.saveOpenTabs).mockResolvedValue([])
  })

  afterEach(() => {
    vi.clearAllMocks()
  })

  function mountTabs(status: AccountStatus) {
    const accounts = useAccountsStore()
    accounts.accounts = [ACCOUNT]
    accounts.connections = [{ id: CONNECTION, accountId: 'acc-1', server: 'play.example.com' }]
    accounts.activeConnectionId = CONNECTION
    accounts.statuses = { [CONNECTION]: status }
    return mount(AccountTabs, { global: { plugins: [i18n] } })
  }

  function closeButton() {
    return `button[title="${i18n.global.t('tabs.close')}"]`
  }

  it.each(['running', 'starting', 'awaiting_login'] as AccountStatus[])(
    'działa lub startuje (%s) - najpierw pyta',
    async (status) => {
      const wrapper = mountTabs(status)

      await wrapper.get(closeButton()).trigger('click')
      await flushPromises()

      expect(wrapper.find('[role="dialog"]').exists()).toBe(true)
      expect(wrapper.text()).toContain(i18n.global.t('confirm.closeTabTitle'))
      expect(backend.stopConnection).not.toHaveBeenCalled()
      expect(useAccountsStore().connections).toHaveLength(1)

      await wrapper.get('[role="dialog"] footer button:last-child').trigger('click')
      await flushPromises()

      expect(backend.stopConnection).toHaveBeenCalledWith(CONNECTION)
      expect(useAccountsStore().connections).toHaveLength(0)
    },
  )

  it('zatrzymana i po awarii zamyka się bez pytania', async () => {
    for (const status of ['stopped', 'crashed'] as AccountStatus[]) {
      const wrapper = mountTabs(status)

      await wrapper.get(closeButton()).trigger('click')
      await flushPromises()

      expect(wrapper.find('[role="dialog"]').exists()).toBe(false)
      expect(backend.stopConnection).toHaveBeenCalledWith(CONNECTION)
      wrapper.unmount()
      vi.clearAllMocks()
    }
  })

  it('anulowanie pytania zostawia zakładkę i proces', async () => {
    const wrapper = mountTabs('running')

    await wrapper.get(closeButton()).trigger('click')
    await flushPromises()
    await wrapper.get('[role="dialog"] footer button:first-child').trigger('click')
    await flushPromises()

    expect(backend.stopConnection).not.toHaveBeenCalled()
    expect(useAccountsStore().connections).toHaveLength(1)
    expect(wrapper.find('[role="dialog"]').exists()).toBe(false)
  })
})

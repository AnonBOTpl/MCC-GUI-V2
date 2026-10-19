import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { nextTick } from 'vue'
import { createPinia, setActivePinia } from 'pinia'
import { i18n } from '../i18n'
import { backend } from '../api/backend'
import { useAccountsStore } from '../stores/accounts'
import type { AccountStatus, Connection } from '../api/contract'
import AccountHeader from './AccountHeader.vue'

vi.mock('../api/backend', () => ({
  backend: {
    startConnection: vi.fn(),
    stopConnection: vi.fn(),
    restartConnection: vi.fn(),
  },
  events: {},
}))

const CONNECTION: Connection = { id: 'acc-1::srv', accountId: 'acc-1', server: 'srv' }

function mountHeader(
  status: AccountStatus,
  connection: Connection = CONNECTION,
  accountLabel = 'A',
) {
  setActivePinia(createPinia())
  const accounts = useAccountsStore()
  // Nagłówek steruje połączeniem z listy store'u, więc musi ono tam być.
  accounts.connections = [connection]
  accounts.activeConnectionId = connection.id
  accounts.statuses = { [connection.id]: status }
  const wrapper = mount(AccountHeader, {
    props: { connection, accountLabel, follow: true },
    global: { plugins: [i18n] },
  })
  return wrapper
}

/** Przycisk akcji po widocznym tekście (etykieta idzie z i18n). */
function action(wrapper: ReturnType<typeof mountHeader>, label: string) {
  const button = wrapper
    .findAll('button')
    .find((node) => node.text().includes(i18n.global.t(label)))
  if (!button) throw new Error(`brak przycisku ${label}`)
  return button
}

/** Drugi klik dwukliku - przeglądarka ustawia wtedy `detail = 2`. */
async function doubleClickSecond(
  wrapper: ReturnType<typeof mountHeader>,
  label: string,
): Promise<void> {
  action(wrapper, label).element.dispatchEvent(
    new MouseEvent('click', { bubbles: true, detail: 2 }),
  )
  await flushPromises()
}

describe('AccountHeader - długi adres serwera (R-26)', () => {
  it('adres i etykieta się skracają, a grupa akcji nie zawija się', () => {
    const longAddress = `${'a'.repeat(110)}.example.com`
    const connection: Connection = { id: 'acc-1::long', accountId: 'acc-1', server: longAddress }
    const wrapper = mountHeader('running', connection, 'Bardzo długa nazwa konta gracza')

    const address = wrapper.get(`span[title="${longAddress}"]`)
    expect(address.classes()).toEqual(expect.arrayContaining(['min-w-0', 'flex-1', 'truncate']))

    const label = wrapper.get('span[title="Bardzo długa nazwa konta gracza"]')
    expect(label.classes()).toContain('truncate')

    // Przyciski akcji mają `shrink-0`, więc nie ściskają się przy długim adresie.
    expect(wrapper.get('header > div').classes()).toContain('shrink-0')
  })
})

describe('AccountHeader - dwuklik na Start', () => {
  beforeEach(() => {
    vi.mocked(backend.startConnection).mockResolvedValue(undefined)
    vi.mocked(backend.stopConnection).mockResolvedValue(undefined)
    vi.mocked(backend.restartConnection).mockResolvedValue(undefined)
  })

  it('drugi klik dwukliku nie uruchamia „Połącz ponownie”', async () => {
    const wrapper = mountHeader('stopped')

    // Pierwszy klik: start. Po nim nagłówek pokazuje Rozłącz + Połącz ponownie.
    await action(wrapper, 'account.start').trigger('click')
    await nextTick()
    await flushPromises()

    // Drugi klik dwukliku trafia w przycisk, który pojawił się w tym miejscu.
    await doubleClickSecond(wrapper, 'account.reconnect')

    expect(backend.startConnection).toHaveBeenCalledTimes(1)
    expect(backend.restartConnection).not.toHaveBeenCalled()
    expect(backend.stopConnection).not.toHaveBeenCalled()
  })

  it('zwykły klik w Rozłącz nadal działa', async () => {
    const wrapper = mountHeader('running')

    await action(wrapper, 'account.disconnect').trigger('click')
    await flushPromises()

    expect(backend.stopConnection).toHaveBeenCalledWith(CONNECTION.id)
  })

  it('drugi klik dwukliku nie rozłącza pojedynczym klikiem', async () => {
    const wrapper = mountHeader('running')

    await doubleClickSecond(wrapper, 'account.disconnect')

    expect(backend.stopConnection).not.toHaveBeenCalled()
  })
})

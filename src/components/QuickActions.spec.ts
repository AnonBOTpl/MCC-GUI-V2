import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { i18n } from '../i18n'
import { useAccountsStore } from '../stores/accounts'
import { usePresetsStore } from '../stores/presets'
import type { Preset } from '../api/contract'
import QuickActions from './QuickActions.vue'
import { backend } from '../api/backend'

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

describe('QuickActions', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.mocked(backend.listPresets).mockResolvedValue([])
    vi.mocked(backend.savePresets).mockImplementation(async (list) => list)
    vi.mocked(backend.reloadPresets).mockResolvedValue([])
    vi.mocked(backend.sendInput).mockResolvedValue(undefined)
    vi.mocked(backend.sendInputAll).mockResolvedValue([])
  })

  function mountBar(list: Preset[]) {
    const presets = usePresetsStore()
    presets.presets = list
    presets.loaded = true
    return mount(QuickActions, { global: { plugins: [i18n] } })
  }

  function openActiveConnection(): void {
    const accounts = useAccountsStore()
    accounts.connections = [{ id: 'acc::srv', accountId: 'acc', server: 'srv' }]
    accounts.activeConnectionId = 'acc::srv'
  }

  it('startuje zwinięty i rozwija się po kliknięciu uchwytu', async () => {
    const wrapper = mountBar([preset('p1', '/send /home')])
    const handle = wrapper.get('button[aria-controls="quick-actions-panel"]')

    expect(handle.attributes('aria-expanded')).toBe('false')

    await handle.trigger('click')
    expect(handle.attributes('aria-expanded')).toBe('true')

    await handle.trigger('click')
    expect(handle.attributes('aria-expanded')).toBe('false')
  })

  it('limit trzech linii: panel ma własne przewijanie i maksymalną wysokość', () => {
    const wrapper = mountBar([preset('p1', '/send /home')])
    const scroll = wrapper.get('.qa-scroll')

    // 3 linie akcji = 104 px, wyżej pasek się już nie rozciąga.
    expect(scroll.classes()).toContain('max-h-[104px]')
    expect(scroll.classes()).toContain('overflow-y-auto')
    expect(scroll.classes()).toContain('overscroll-contain')
  })

  it('klik pigułki wysyła jej tekst do aktywnego połączenia', async () => {
    openActiveConnection()
    const wrapper = mountBar([preset('p1', '/send /home')])

    await wrapper.get('button[title="Sends: /send /home"]').trigger('click')

    expect(backend.sendInput).toHaveBeenCalledWith('acc::srv', '/send /home')
    expect(backend.sendInputAll).not.toHaveBeenCalled()
  })

  it('przy zaznaczonym „Send to all” preset idzie do wszystkich i tak też brzmi podpowiedź', async () => {
    vi.mocked(backend.sendInputAll).mockResolvedValue(['acc::srv'])
    const accounts = useAccountsStore()
    accounts.sendToAll = true
    const wrapper = mountBar([{ id: 'p1', label: '/reco', text: '/send /reco', color: null, enabled: true }])

    await wrapper.get('button[title="Sends to all running connections: /send /reco"]').trigger('click')

    expect(backend.sendInputAll).toHaveBeenCalledWith('/send /reco')
    expect(backend.sendInput).not.toHaveBeenCalled()
  })

  it('wyłączona akcja nie trafia na pasek', async () => {
    const wrapper = mountBar([
      preset('p1', '/send /home'),
      preset('p2', '/send /bal', false),
    ])

    expect(wrapper.findAll('button[title^="Sends"]')).toHaveLength(1)
    expect(wrapper.text()).not.toContain('/send /bal')
    // Licznik pokazuje to, co widać na pasku.
    expect(wrapper.get('button[aria-controls="quick-actions-panel"]').text()).toContain('1 action')
  })

  it('gdy wszystkie akcje są wyłączone, pasek mówi o tym i daje edytor', async () => {
    const wrapper = mountBar([preset('p1', '/send /home', false)])

    expect(wrapper.text()).toContain('Every action is switched off')
    expect(wrapper.text()).not.toContain('No actions yet')
    expect(wrapper.findAll('button').some((button) => button.text() === 'Edit')).toBe(true)
  })

  it('bez aktywnego połączenia nic nie wysyła i pokazuje komunikat', async () => {
    const wrapper = mountBar([preset('p1', '/send /home')])

    await wrapper.get('button[title="Sends: /send /home"]').trigger('click')
    // Wysyłka jest asynchroniczna, a komunikat ustawia się dopiero po jej wyniku.
    await flushPromises()

    expect(backend.sendInput).not.toHaveBeenCalled()
    expect(wrapper.text()).toContain('Open a connection first')
  })

  it('przy pustej liście pokazuje podpowiedź i przycisk dodawania akcji', async () => {
    const wrapper = mountBar([])

    expect(wrapper.text()).toContain('No actions yet')
    // Przy braku akcji zamiast „Edit” jest wprost „Add actions”.
    expect(wrapper.text()).not.toContain('Reload')

    const addButton = wrapper.findAll('button').find((button) => button.text() === 'Add actions')
    expect(addButton).toBeDefined()
    await addButton!.trigger('click')

    // Edytor podpowiada składnię `/send` trzema gotowymi przykładami.
    expect(wrapper.text()).toContain('Saved to presets.json')
    expect(wrapper.text()).toContain('Examples')
  })

  it('„Edit” otwiera edytor z bieżącymi presetami', async () => {
    const wrapper = mountBar([preset('p1', '/send /home')])

    const edit = wrapper.findAll('button').find((button) => button.text() === 'Edit')
    expect(edit).toBeDefined()
    await edit!.trigger('click')

    const texts = wrapper.findAll('input[aria-label="Text sent to the client"]')
    expect(texts.some((input) => (input.element as HTMLInputElement).value === '/send /home')).toBe(
      true,
    )
  })

  it('„Reload” wczytuje plik ponownie i potwierdza to przy pasku', async () => {
    vi.mocked(backend.reloadPresets).mockResolvedValue([
      preset('p1', '/send /home'),
      preset('p2', '/send /bal'),
    ])
    const wrapper = mountBar([preset('p1', '/send /home')])

    const reload = wrapper.findAll('button').find((button) => button.text() === 'Reload')
    await reload!.trigger('click')
    await flushPromises()

    expect(backend.reloadPresets).toHaveBeenCalled()
    expect(wrapper.text()).toContain('Reloaded 2 actions')
  })

  it('kolor presetu daje pasek przy etykiecie', () => {
    const wrapper = mountBar([
      { id: 'p1', label: '/home', text: '/send /home', color: 'green', enabled: true },
    ])

    const stripe = wrapper.get('button[title="Sends: /send /home"] span')
    expect(stripe.classes()).toContain('bg-green-500')
  })
})

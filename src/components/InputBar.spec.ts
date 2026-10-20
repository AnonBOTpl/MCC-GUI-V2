import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { nextTick } from 'vue'
import { backend } from '../api/backend'
import { i18n } from '../i18n'
import { useAccountsStore } from '../stores/accounts'
import InputBar from './InputBar.vue'

vi.mock('../api/backend', () => ({
  backend: {
    sendInput: vi.fn(),
    sendInputAll: vi.fn(),
  },
  events: {},
}))

describe('InputBar', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.mocked(backend.sendInput).mockResolvedValue(undefined)
    vi.mocked(backend.sendInputAll).mockResolvedValue([])
  })

  afterEach(() => {
    vi.clearAllMocks()
    vi.useRealTimers()
  })

  function mountBar() {
    return mount(InputBar, { global: { plugins: [i18n] } })
  }

  /** Aktywne połączenie - domyślny cel wysyłki. */
  function openConnection(): void {
    const accounts = useAccountsStore()
    accounts.connections = [{ id: 'acc::srv', accountId: 'acc', server: 'srv' }]
    accounts.activeConnectionId = 'acc::srv'
  }

  it('wysyła wpis i czyści pole po udanej wysyłce', async () => {
    openConnection()
    const wrapper = mountBar()

    await wrapper.get('input[type="text"]').setValue('/say hej')
    await wrapper.get('form').trigger('submit')
    await flushPromises()

    expect(backend.sendInput).toHaveBeenCalledWith('acc::srv', '/say hej')
    expect((wrapper.get('input[type="text"]').element as HTMLInputElement).value).toBe('')
    expect(wrapper.text()).not.toContain('not sent')
  })

  /**
   * R-24: przy braku odbiorców tekst był kasowany z pola, więc użytkownik tracił
   * wpis bez żadnej informacji. Teraz zostaje w polu, a nad nim pojawia się powód.
   */
  it('bez otwartego połączenia zostawia tekst w polu i mówi dlaczego (R-24)', async () => {
    const wrapper = mountBar()

    await wrapper.get('input[type="text"]').setValue('/say hej')
    await wrapper.get('form').trigger('submit')
    await flushPromises()

    expect(backend.sendInput).not.toHaveBeenCalled()
    expect((wrapper.get('input[type="text"]').element as HTMLInputElement).value).toBe('/say hej')
    expect(wrapper.text()).toContain('Open a connection first')
  })

  it('przy „Send to all” bez działających kont też nie gubi wpisu (R-24)', async () => {
    const accounts = useAccountsStore()
    accounts.sendToAll = true
    openConnection()
    const wrapper = mountBar()

    await wrapper.get('input[type="text"]').setValue('/say hej')
    await wrapper.get('form').trigger('submit')
    await flushPromises()

    expect(backend.sendInputAll).toHaveBeenCalledWith('/say hej')
    expect((wrapper.get('input[type="text"]').element as HTMLInputElement).value).toBe('/say hej')
    expect(wrapper.text()).toContain('Nothing is running')
  })

  it('komunikat znika sam po kilku sekundach (R-24)', async () => {
    vi.useFakeTimers()
    const wrapper = mountBar()

    await wrapper.get('input[type="text"]').setValue('/say hej')
    await wrapper.get('form').trigger('submit')
    await flushPromises()
    expect(wrapper.text()).toContain('Open a connection first')

    vi.advanceTimersByTime(3000)
    await nextTick()

    expect(wrapper.text()).not.toContain('Open a connection first')
    // Wpis zostaje - komunikat tylko informował.
    expect((wrapper.get('input[type="text"]').element as HTMLInputElement).value).toBe('/say hej')
  })

  it('nie kasuje tekstu wpisanego już w trakcie wysyłki', async () => {
    openConnection()
    let release: (() => void) | undefined
    vi.mocked(backend.sendInput).mockImplementation(
      () =>
        new Promise<void>((resolve) => {
          release = resolve
        }),
    )
    const wrapper = mountBar()

    await wrapper.get('input[type="text"]').setValue('pierwszy')
    await wrapper.get('form').trigger('submit')
    // Użytkownik zdążył wpisać kolejną wiadomość, zanim pierwsza doszła do klienta.
    await wrapper.get('input[type="text"]').setValue('drugi')

    release?.()
    await flushPromises()

    expect((wrapper.get('input[type="text"]').element as HTMLInputElement).value).toBe('drugi')
  })

  /**
   * S-01: pole czyścimy po odpowiedzi Rusta, więc drugi submit w trakcie
   * wysyłki widział jeszcze ten sam tekst i wysyłał go drugi raz.
   */
  it('drugi submit w trakcie wysyłki nie wysyła wpisu ponownie (S-01)', async () => {
    openConnection()
    let release: (() => void) | undefined
    vi.mocked(backend.sendInput).mockImplementation(
      () =>
        new Promise<void>((resolve) => {
          release = resolve
        }),
    )
    const wrapper = mountBar()

    await wrapper.get('input[type="text"]').setValue('/say hej')
    await wrapper.get('form').trigger('submit')
    // Drugi Enter, zanim pierwsza wysyłka doszła do backendu.
    await wrapper.get('form').trigger('submit')

    expect(backend.sendInput).toHaveBeenCalledTimes(1)

    release?.()
    await flushPromises()

    expect(backend.sendInput).toHaveBeenCalledTimes(1)
    expect((wrapper.get('input[type="text"]').element as HTMLInputElement).value).toBe('')
  })

  it('udana wysyłka kasuje wcześniejszy komunikat (R-24)', async () => {
    const accounts = useAccountsStore()
    const wrapper = mountBar()

    // Najpierw wysyłka bez celu - komunikat się pojawia.
    await wrapper.get('input[type="text"]').setValue('/say hej')
    await wrapper.get('form').trigger('submit')
    await flushPromises()
    expect(wrapper.text()).toContain('Open a connection first')

    // Potem otwieramy połączenie i wysyłamy ponownie.
    openConnection()
    accounts.sendToAll = false
    await wrapper.get('form').trigger('submit')
    await flushPromises()

    expect(wrapper.text()).not.toContain('Open a connection first')
    expect((wrapper.get('input[type="text"]').element as HTMLInputElement).value).toBe('')
  })
})

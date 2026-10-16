import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount, type VueWrapper } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { defineComponent, h, nextTick, ref } from 'vue'
import { backend } from '../api/backend'
import { i18n } from '../i18n'
import { useLogsStore } from '../stores/logs'
import LogView from './LogView.vue'

// Schowek zapisujemy po stronie Rusta (`copy_text`), więc w testach podstawiamy
// tylko tę jedną komendę.
vi.mock('../api/backend', () => ({
  backend: { copyText: vi.fn() },
  events: {},
}))

/**
 * Okno aplikacji trzyma `follow` i podaje je przez `v-model`. Własny komponent
 * jest potrzebny, żeby odtworzyć prawdziwą kolejność: kliknięcie przycisku
 * najpierw aktualizuje stan rodzica (i uruchamia przerysowanie), a dociągnięcie
 * na dół idzie dopiero po nim.
 */
const Host = defineComponent({
  setup() {
    const follow = ref(true)
    return () =>
      h(LogView, {
        accountId: 'acc::srv',
        follow: follow.value,
        'onUpdate:follow': (value: boolean) => {
          follow.value = value
        },
      })
  },
})

// jsdom nie liczy układu (wszystkie wysokości są zerowe), więc podstawiamy
// geometrię przewijania: `scrollTop` z ograniczeniem do rzeczywistego zakresu.
function stubGeometry(element: HTMLElement, viewportHeight: number, contentHeight: number): void {
  let top = 0
  Object.defineProperty(element, 'clientHeight', {
    configurable: true,
    get: () => viewportHeight,
  })
  Object.defineProperty(element, 'scrollHeight', {
    configurable: true,
    get: () => contentHeight,
  })
  Object.defineProperty(element, 'scrollTop', {
    configurable: true,
    get: () => top,
    set: (value: number) => {
      top = Math.max(0, Math.min(value, contentHeight - viewportHeight))
    },
  })
}

/** Czeka, aż store odświeży widok (klatka albo pilnowacz) i Vue przerysuje DOM. */
async function settle(): Promise<void> {
  await new Promise((resolve) => setTimeout(resolve, 250))
  await flushPromises()
  await flushPromises()
}

describe('LogView - zaznaczanie i kopiowanie', () => {
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
    vi.mocked(backend.copyText).mockReset()
    vi.mocked(backend.copyText).mockResolvedValue(undefined)
  })

  afterEach(() => {
    vi.unstubAllGlobals()
  })

  /**
   * Bufor z pięcioma liniami (z kodem koloru `§a`, żeby sprawdzić, że kopiowany
   * tekst jest czysty). Wiersze trafiają do DOM bez stubowania geometrii, bo
   * widok startuje z górnej partii bufora.
   */
  async function setup(count = 5) {
    const wrapper = mount(LogView, {
      props: { accountId: 'acc::srv', follow: true },
      global: { plugins: [i18n] },
    })
    const logs = useLogsStore()
    logs.append(
      'acc::srv',
      Array.from({ length: count }, (_, index) => ({
        stream: 'stdout' as const,
        raw: `\u00a7alinia ${index + 1}`,
        ts: index,
      })),
      'acc::srv',
    )
    logs.flushNow()
    await nextTick()

    const viewport = wrapper.find('.log-text.overflow-auto')
    return { wrapper, logs, viewport }
  }

  /** Wiersze widoczne w DOM (w kolejności bufora). */
  function rows(wrapper: VueWrapper) {
    return wrapper.findAll('[data-seq]')
  }

  function selected(wrapper: VueWrapper): string[] {
    return wrapper.findAll('.log-row-selected').map((node) => node.text())
  }

  it('klik zaznacza jeden wiersz i wyłącza autoprzewijanie', async () => {
    const { wrapper } = await setup()

    await rows(wrapper)[1].trigger('click')

    expect(selected(wrapper)).toEqual(['linia 2'])
    expect(wrapper.emitted('update:follow')).toEqual([[false]])
  })

  it('Ctrl+klik dodaje wiersz do zaznaczenia, a kolejny Ctrl+klik go odejmuje', async () => {
    const { wrapper } = await setup()

    await rows(wrapper)[0].trigger('click')
    await rows(wrapper)[2].trigger('click', { ctrlKey: true })
    expect(selected(wrapper)).toEqual(['linia 1', 'linia 3'])

    await rows(wrapper)[2].trigger('click', { ctrlKey: true })
    expect(selected(wrapper)).toEqual(['linia 1'])
  })

  it('Shift+klik zaznacza zakres od kotwicy, zastępując poprzednie zaznaczenie', async () => {
    const { wrapper } = await setup()

    await rows(wrapper)[3].trigger('click')
    await rows(wrapper)[1].trigger('click', { shiftKey: true })

    // Zakres od kotwicy (wiersz 4) w górę do klikniętego wiersza 2.
    expect(selected(wrapper)).toEqual(['linia 2', 'linia 3', 'linia 4'])
  })

  it('Ctrl+Shift+klik dodaje zakres do istniejącego zaznaczenia', async () => {
    const { wrapper } = await setup()

    await rows(wrapper)[0].trigger('click')
    await rows(wrapper)[2].trigger('click', { ctrlKey: true }) // kotwica = wiersz 3
    await rows(wrapper)[4].trigger('click', { ctrlKey: true, shiftKey: true })

    expect(selected(wrapper)).toEqual(['linia 1', 'linia 3', 'linia 4', 'linia 5'])
  })

  it('Shift+klik nie uruchamia natywnego zaznaczania tekstu', async () => {
    const { wrapper } = await setup()

    const shiftClick = new MouseEvent('mousedown', { shiftKey: true, cancelable: true })
    rows(wrapper)[2].element.dispatchEvent(shiftClick)
    expect(shiftClick.defaultPrevented).toBe(true)

    // Zwykły klik zostawia natywne zaznaczanie tekstu w spokoju.
    const plainClick = new MouseEvent('mousedown', { cancelable: true })
    rows(wrapper)[2].element.dispatchEvent(plainClick)
    expect(plainClick.defaultPrevented).toBe(false)
  })

  it('Ctrl+C kopiuje zaznaczone wiersze jako czysty tekst', async () => {
    const { wrapper, viewport } = await setup()

    await rows(wrapper)[0].trigger('click')
    await rows(wrapper)[2].trigger('click', { shiftKey: true })
    await viewport.trigger('keydown', { key: 'c', ctrlKey: true })
    await flushPromises()

    expect(backend.copyText).toHaveBeenCalledWith('linia 1\nlinia 2\nlinia 3')
    // Po skopiowaniu zaznaczenie znika, a autoprzewijanie wraca.
    expect(selected(wrapper)).toHaveLength(0)
    expect(wrapper.emitted('update:follow')?.at(-1)).toEqual([true])
  })

  it('po obcięciu bufora zaznaczenie obejmuje tylko to, co zostało', async () => {
    const { wrapper, viewport, logs } = await setup()

    await rows(wrapper)[0].trigger('click')
    await rows(wrapper)[2].trigger('click', { shiftKey: true })
    await rows(wrapper)[4].trigger('click', { ctrlKey: true })
    // Bufor obcina wszystko oprócz ostatniej linii (wiersze 1-4 wypadają).
    logs.setLimit(1)
    logs.flushNow()
    await nextTick()

    await viewport.trigger('keydown', { key: 'c', ctrlKey: true })
    await flushPromises()

    expect(backend.copyText).toHaveBeenCalledWith('linia 5')
  })

  it('gdy obcięto wszystkie zaznaczone linie, kopiowanie nic nie wysyła', async () => {
    const { wrapper, viewport, logs } = await setup()

    await rows(wrapper)[0].trigger('click')
    logs.clear('acc::srv')
    logs.flushNow()
    await nextTick()

    await viewport.trigger('keydown', { key: 'c', ctrlKey: true })
    await flushPromises()

    expect(backend.copyText).not.toHaveBeenCalled()
  })

  it('nieudane kopiowanie zostawia zaznaczenie w spokoju', async () => {
    vi.mocked(backend.copyText).mockRejectedValue({ code: 'clipboard.writeFailed', params: {} })
    const { wrapper, viewport } = await setup()

    await rows(wrapper)[0].trigger('click')
    await viewport.trigger('keydown', { key: 'c', ctrlKey: true })
    await flushPromises()

    expect(selected(wrapper)).toEqual(['linia 1'])
    expect(wrapper.emitted('update:follow')?.at(-1)).toEqual([false])
  })

  it('Ctrl+C bez zaznaczenia nie przechwytuje kopiowania', async () => {
    const { viewport } = await setup()

    await viewport.trigger('keydown', { key: 'c', ctrlKey: true })
    await flushPromises()

    expect(backend.copyText).not.toHaveBeenCalled()
  })

  it('Ctrl+A zaznacza cały bufor, Esc czyści zaznaczenie i wraca do autoprzewijania', async () => {
    const { wrapper, viewport } = await setup()

    await viewport.trigger('keydown', { key: 'a', ctrlKey: true })
    expect(selected(wrapper)).toHaveLength(5)

    await viewport.trigger('keydown', { key: 'Escape' })
    expect(selected(wrapper)).toHaveLength(0)
    expect(wrapper.emitted('update:follow')?.at(-1)).toEqual([true])
  })

  it('Esc dociąga widok do końca logu', async () => {
    const { wrapper } = await setup()
    const element = wrapper.find('.log-text.overflow-auto').element as HTMLElement
    stubGeometry(element, 200, 2000)
    await flushPromises()

    // Widok zjeżdża na dół przy starcie; przewijamy go „ręką” i zaznaczamy wiersz.
    element.dispatchEvent(new WheelEvent('wheel', { bubbles: true, deltaY: -100 }))
    element.scrollTop = 400
    await rows(wrapper)[0].trigger('click')
    expect(element.scrollTop).toBe(400)

    await wrapper.find('.log-text.overflow-auto').trigger('keydown', { key: 'Escape' })
    await flushPromises()
    await flushPromises()

    expect(element.scrollTop).toBe(1800) // koniec logu = scrollHeight - clientHeight
  })

  it('skróty nie działają poza widokiem logu', async () => {
    const { wrapper } = await setup()

    // Zdarzenie poza elementem widoku (np. pole wpisywania komend) nie może
    // zaznaczać logu pod spodem.
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'a', ctrlKey: true }))
    await nextTick()

    expect(selected(wrapper)).toHaveLength(0)
  })

  it('menu kontekstowe: „Kopiuj” nieaktywne bez zaznaczenia, „Kopiuj wszystko” kopiuje bufor', async () => {
    const { wrapper, viewport } = await setup()

    await viewport.trigger('contextmenu', { clientX: 40, clientY: 40 })
    const items = wrapper.findAll('[role="menuitem"]')
    expect(items).toHaveLength(2)
    expect(items[0].attributes('disabled')).toBeDefined()

    await items[1].trigger('click')
    await flushPromises()

    expect(backend.copyText).toHaveBeenCalledWith(
      'linia 1\nlinia 2\nlinia 3\nlinia 4\nlinia 5',
    )
    // Menu zamyka się po użyciu.
    expect(wrapper.findAll('[role="menuitem"]')).toHaveLength(0)
  })

  it('menu kontekstowe: „Kopiuj” kopiuje tylko zaznaczone wiersze', async () => {
    const { wrapper, viewport } = await setup()

    await rows(wrapper)[1].trigger('click')
    await viewport.trigger('contextmenu', { clientX: 40, clientY: 40 })
    const items = wrapper.findAll('[role="menuitem"]')
    expect(items[0].attributes('disabled')).toBeUndefined()

    await items[0].trigger('click')
    await flushPromises()

    expect(backend.copyText).toHaveBeenCalledWith('linia 2')
    // Kopiowanie z menu też czyści zaznaczenie i wraca do autoprzewijania.
    expect(selected(wrapper)).toHaveLength(0)
    expect(wrapper.emitted('update:follow')?.at(-1)).toEqual([true])
  })
})

describe('LogView - aktualizacja widoku', () => {
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
  })

  afterEach(() => {
    vi.unstubAllGlobals()
  })

  /**
   * Regresja: bufor linii mutuje tę samą tablicę
   * w miejscu, więc `computed` widoku, który zwracał ją bez kopii, przy każdej
   * dawce linii zwracał **tę samą tożsamość**. Vue porównuje wartości przez
   * `Object.is`, więc jego wersyjne sprawdzanie zmian uznawało, że nic się nie
   * zmieniło, i pomijało aktualizację widoku (`instance.job =
   * effect.runIfDirty.bind(effect)` w `@vue/runtime-core`). Log stał zamrożony do
   * najbliższego przerysowania wywołanego czymś innym - np. przewinięciem.
   * Ten test tego nie przepuszcza: nie ma tu żadnego zdarzenia wejścia ani ruchu
   * przewijania, jest tylko dopisanie linii.
   */
  it('pokazuje nowe linie po każdej dawce, bez zdarzeń wejścia i przewijania', async () => {
    const wrapper = mount(LogView, {
      props: { accountId: 'acc::srv', follow: true },
      global: { plugins: [i18n] },
    })
    const logs = useLogsStore()

    // Bufor musi być **zapełniony**: dopóki rośnie, liczba linii też się zmienia
    // i sama unieważnia widok. Zamrożenie widać dopiero, gdy rozmiar bufora stoi.
    logs.setLimit(3)
    for (const raw of ['linia 1', 'linia 2', 'linia 3']) {
      logs.append('acc::srv', [{ stream: 'stdout', raw, ts: 1 }], 'acc::srv')
      logs.flushNow()
      await nextTick()
    }
    expect(wrapper.text()).toContain('linia 3')

    // Czwarta linia wypycha najstarszą - rozmiar bufora się nie zmienia.
    logs.append('acc::srv', [{ stream: 'stdout', raw: 'linia 4', ts: 2 }], 'acc::srv')
    logs.flushNow()
    await nextTick()

    expect(wrapper.text()).toContain('linia 4')
    expect(wrapper.text()).not.toContain('linia 1')

    wrapper.unmount()
  })
})

describe('LogView - autoprzewijanie', () => {
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
  })

  afterEach(() => {
    vi.unstubAllGlobals()
  })

  async function setup(follow: boolean) {
    const wrapper = mount(LogView, {
      props: { accountId: 'acc::srv', follow },
      global: { plugins: [i18n] },
    })
    const viewport = wrapper.find('.log-text.overflow-auto')
    stubGeometry(viewport.element as HTMLElement, 200, 2000)
    // Widok startuje z dociągnięciem do dołu (`onMounted`) - odczekujemy je,
    // zanim udamy przewinięcie użytkownika.
    await flushPromises()
    return { wrapper, viewport }
  }

  /** Przewinięcie „ręką”: kółko unieważnia znacznik naszego własnego przewinięcia. */
  function userScroll(element: HTMLElement, top: number): void {
    element.dispatchEvent(new WheelEvent('wheel', { bubbles: true, deltaY: -100 }))
    element.scrollTop = top
    element.dispatchEvent(new Event('scroll'))
  }

  it('nie rusza śledzenia, gdy widok stoi na dole', async () => {
    const { wrapper, viewport } = await setup(true)
    userScroll(viewport.element as HTMLElement, 1800)
    expect(wrapper.emitted('update:follow')).toBeUndefined()
  })

  it('zatrzymuje śledzenie po przewinięciu w górę', async () => {
    const { wrapper, viewport } = await setup(true)
    userScroll(viewport.element as HTMLElement, 1000)
    expect(wrapper.emitted('update:follow')).toEqual([[false]])
  })

  it('wznawia śledzenie po powrocie na dole', async () => {
    const { wrapper, viewport } = await setup(false)
    userScroll(viewport.element as HTMLElement, 1800)
    expect(wrapper.emitted('update:follow')).toEqual([[true]])
  })

  it('chowa przycisk śledzenia po kliknięciu, gdy widok dojechał na dół', async () => {
    const wrapper = mount(Host, { global: { plugins: [i18n] } })
    const element = wrapper.find('.log-text.overflow-auto').element as HTMLElement
    stubGeometry(element, 200, 2000)
    await flushPromises()

    userScroll(element, 1000)
    await flushPromises()
    expect(wrapper.findAll('button')).toHaveLength(1) // przycisk „Following” widoczny

    await wrapper.find('button').trigger('click')
    await settle()

    expect(element.scrollTop).toBe(1800)
    expect(wrapper.findAll('button')).toHaveLength(0)
  })

  it('pokazuje zaległość na pierwszym zdarzeniu wejścia, bez czekania na pilnowacz', async () => {
    // Klatki nie ma (okno zminimalizowane albo zasłonięte) - zostaje pilnowacz.
    vi.stubGlobal('requestAnimationFrame', () => 0)
    const { wrapper, viewport } = await setup(true)
    const logs = useLogsStore()

    logs.append('acc::srv', [{ stream: 'stdout', raw: 'po powrocie', ts: 1 }], 'acc::srv')
    // Tylko mikro-zadania: pilnowacz (200 ms) jeszcze nie zdążył, więc widok stoi.
    await nextTick()
    expect(wrapper.text()).not.toContain('po powrocie')

    // Pierwszy klawisz wpisanej komendy domyka odświeżenie i dociąga widok na dół.
    window.dispatchEvent(new KeyboardEvent('keydown'))
    await flushPromises()

    expect(wrapper.text()).toContain('po powrocie')
    expect((viewport.element as HTMLElement).scrollTop).toBe(1800)
    wrapper.unmount()
  })

  it('nie wyłącza śledzenia przez własne dociągnięcie do dołu', async () => {
    const { wrapper, viewport } = await setup(true)
    const logs = useLogsStore()

    // Nowa linia: widok sam zjeżdża na dół (nasze przewinięcie).
    logs.append('acc::srv', [{ stream: 'stdout', raw: 'linia', ts: 1 }], 'acc::srv')
    await settle()

    // Przeglądarka podnosi zdarzenie `scroll` także dla naszego przewinięcia.
    ;(viewport.element as HTMLElement).dispatchEvent(new Event('scroll'))
    expect(wrapper.emitted('update:follow')).toBeUndefined()

    // Znacznik nie może zostać na stałe: kolejne przewinięcie ręką działa.
    userScroll(viewport.element as HTMLElement, 200)
    expect(wrapper.emitted('update:follow')).toEqual([[false]])
  })
})

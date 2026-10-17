import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { i18n } from '../i18n'
import { backend } from '../api/backend'
import type { Preset } from '../api/contract'
import { usePresetsStore } from '../stores/presets'
import PresetEditor from './PresetEditor.vue'

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

function preset(id: string, label: string, text: string, enabled = true): Preset {
  return { id, label, text, color: null, enabled }
}

/**
 * jsdom nie liczy układu (wszystkie prostokąty są zerowe), więc podstawiamy
 * geometrię wierszy - bez tego nie da się policzyć, nad którym wierszem jest
 * kursor. Ten sam trik jest w `LogView.spec.ts`.
 */
function stubRowGeometry(wrapper: ReturnType<typeof mount>): void {
  wrapper.findAll('[data-preset-row]').forEach((row, index) => {
    const top = index * 50
    Object.defineProperty(row.element, 'getBoundingClientRect', {
      configurable: true,
      value: () => ({
        top,
        bottom: top + 40,
        left: 0,
        right: 300,
        width: 300,
        height: 40,
        x: 0,
        y: top,
        toJSON: () => ({}),
      }),
    })
  })
}

/** Teksty wierszy listy (bez pola formularza „nowa akcja” na dole). */
function fieldValues(wrapper: ReturnType<typeof mount>): string[] {
  return wrapper
    .findAll('[data-preset-row] input[aria-label="Text sent to the client"]')
    .map((input) => (input.element as HTMLInputElement).value)
}

describe('PresetEditor', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.mocked(backend.listPresets).mockResolvedValue([])
    vi.mocked(backend.savePresets).mockImplementation(async (list) => list)
    vi.mocked(backend.reloadPresets).mockResolvedValue([])
    vi.mocked(backend.sendInput).mockResolvedValue(undefined)
    vi.mocked(backend.sendInputAll).mockResolvedValue([])
  })

  function setup(list: Preset[]) {
    const presets = usePresetsStore()
    presets.presets = list
    return mount(PresetEditor, { props: { open: true }, global: { plugins: [i18n] } })
  }

  it('jest dialogiem (R-06)', () => {
    const wrapper = setup([])

    expect(wrapper.get('[role="dialog"]').attributes('aria-modal')).toBe('true')
    expect(wrapper.get('[role="dialog"]').attributes('aria-label')).toBe('Quick actions')
  })

  it('dodaje nową akcję z etykietą i tekstem', async () => {
    const wrapper = setup([])

    await wrapper.get('input[aria-label="Label"]').setValue('/home')
    const textField = wrapper.findAll('input[aria-label="Text sent to the client"]').at(-1)!
    await textField.setValue('/send /home')
    const add = wrapper.findAll('button').find((button) => button.text() === 'Add')
    await add!.trigger('click')

    // Wiersz listy jest pierwszym polem tekstowym (drugie to formularz nowej akcji).
    expect(fieldValues(wrapper)[0]).toBe('/send /home')
  })

  it('przy pustej liście podaje gotowe przykłady do przyjęcia jednym kliknięciem', async () => {
    const wrapper = setup([])

    expect(wrapper.text()).toContain('Examples')
    const addExample = wrapper.findAll('button').filter((button) => button.text() === 'Add')
    // Trzy przykłady + przycisk formularza nowej akcji.
    expect(addExample.length).toBe(4)

    await addExample[0].trigger('click')
    expect(fieldValues(wrapper)).toEqual(['/send /home'])
  })

  it('edytuje istniejącą akcję i zapisuje zmianę', async () => {
    const wrapper = setup([preset('p1', '/home', '/send /home')])

    await wrapper.get('[data-preset-row] input[aria-label="Label"]').setValue('/dom')
    const textField = wrapper.get('[data-preset-row] input[aria-label="Text sent to the client"]')
    await textField.setValue('/send /dom')
    const save = wrapper.findAll('button').find((button) => button.text() === 'Save')
    await save!.trigger('click')
    await flushPromises()

    expect(backend.savePresets).toHaveBeenCalledWith([
      preset('p1', '/dom', '/send /dom'),
    ])
  })

  it('przełącznikiem wyłącza akcję (i potrafi ją z powrotem włączyć)', async () => {
    const wrapper = setup([preset('p1', '/reco', '/send /reco')])
    const toggle = wrapper.get('[data-preset-toggle]')

    expect(toggle.attributes('aria-checked')).toBe('true')
    expect(toggle.attributes('title')).toBe('Turn this action off')

    await toggle.trigger('click')
    expect(toggle.attributes('aria-checked')).toBe('false')
    expect(toggle.attributes('title')).toBe('Turn this action on')

    const save = wrapper.findAll('button').find((button) => button.text() === 'Save')
    await save!.trigger('click')
    await flushPromises()

    expect(vi.mocked(backend.savePresets).mock.calls[0][0][0].enabled).toBe(false)
  })

  it('edytor pokazuje też akcje wyłączone (one tylko znikają z paska)', () => {
    const wrapper = setup([preset('p1', '/home', '/send /home', false)])

    expect(fieldValues(wrapper)).toEqual(['/send /home'])
    expect(wrapper.get('[data-preset-toggle]').attributes('aria-checked')).toBe('false')
  })

  it('usuwa akcję z listy', async () => {
    const wrapper = setup([
      preset('p1', '/home', '/send /home'),
      preset('p2', '/bal', '/send /bal'),
    ])

    const removeButtons = wrapper.findAll('button[title="Delete"]')
    expect(removeButtons).toHaveLength(2)
    await removeButtons[0].trigger('click')

    expect(fieldValues(wrapper)).toEqual(['/send /bal'])
  })

  it('zmienia kolejność przeciąganiem uchwytu i zapisuje nowy porządek', async () => {
    const wrapper = setup([
      preset('p1', '/home', '/send /home'),
      preset('p2', '/bal', '/send /bal'),
    ])
    stubRowGeometry(wrapper)

    // jsdom nie ma `PointerEvent`, a VTU nie potrafi ustawić `button` na
    // `MouseEvent`, więc zdarzenia wysyłamy ręcznie (typ zdarzenia jest dowolny).
    const grips = wrapper.findAll('[data-preset-grip]')
    grips[1].element.dispatchEvent(new MouseEvent('pointerdown', { bubbles: true, button: 0 }))
    await flushPromises()
    // Przeciąganie idzie po zdarzeniach wskaźnika na `window` (uchwyt trzyma
    // przechwycenie wskaźnika, więc można wyjść poza niego).
    window.dispatchEvent(new MouseEvent('pointermove', { clientY: 20 }))
    await flushPromises()

    expect(fieldValues(wrapper)).toEqual(['/send /bal', '/send /home'])

    window.dispatchEvent(new MouseEvent('pointerup'))
    await flushPromises()
    // Po puszczeniu nic już nie jest „przeciągane”.
    expect(wrapper.findAll('[class*="border-dashed"]')).toHaveLength(0)

    const save = wrapper.findAll('button').find((button) => button.text() === 'Save')
    await save!.trigger('click')
    await flushPromises()

    expect(vi.mocked(backend.savePresets).mock.calls[0][0].map((item) => item.text)).toEqual([
      '/send /bal',
      '/send /home',
    ])
  })

  it('„Cancel” porzuca zmiany i nic nie zapisuje', async () => {
    const wrapper = setup([preset('p1', '/home', '/send /home')])

    await wrapper
      .get('[data-preset-row] input[aria-label="Text sent to the client"]')
      .setValue('/send /zmiana')
    await wrapper.findAll('button').find((button) => button.text() === 'Cancel')!.trigger('click')

    expect(backend.savePresets).not.toHaveBeenCalled()
    expect(wrapper.emitted('close')).toBeTruthy()

    // Kolejne otwarcie startuje z listy ze store'u, bez porzuconej zmiany.
    const reopened = setup([preset('p1', '/home', '/send /home')])
    expect(fieldValues(reopened)).toEqual(['/send /home'])
  })

  /**
   * R-38: edytor pozwalał dodać 61. akcję, a Rust odrzucał cały zapis przy
   * `Save`. Dziś przycisk „Dodaj” jest zablokowany na limicie, a licznik przy
   * nagłówku pokazuje, ile jeszcze zostało.
   */
  it('blokuje dodanie akcji powyżej limitu i pokazuje licznik (R-38)', async () => {
    const full = Array.from({ length: 60 }, (_, index) =>
      preset(`p${index}`, `/akcja${index}`, `/send /akcja${index}`),
    )
    const wrapper = setup(full)

    expect(wrapper.text()).toContain('60 / 60')
    expect(wrapper.text()).toContain('Limit reached')
    const add = wrapper.findAll('button').find((button) => button.text() === 'Add')
    expect((add!.element as HTMLButtonElement).disabled).toBe(true)

    // Nawet po wpisaniu treści przycisk zostaje zablokowany.
    // Pola formularza „nowa akcja” są **na końcu** listy, nie pierwsze.
    await wrapper.findAll('input[aria-label="Label"]').at(-1)!.setValue('/nowa')
    const textField = wrapper.findAll('input[aria-label="Text sent to the client"]').at(-1)!
    await textField.setValue('/send /nowa')
    expect((add!.element as HTMLButtonElement).disabled).toBe(true)
    expect(fieldValues(wrapper)).toHaveLength(60)
  })

  it('przy jednym wolnym miejscu dodaje akcję, a potem pokazuje limit (R-38)', async () => {
    const almost = Array.from({ length: 59 }, (_, index) =>
      preset(`p${index}`, `/akcja${index}`, `/send /akcja${index}`),
    )
    const wrapper = setup(almost)

    await wrapper.findAll('input[aria-label="Label"]').at(-1)!.setValue('/ostatnia')
    const textField = wrapper.findAll('input[aria-label="Text sent to the client"]').at(-1)!
    await textField.setValue('/send /ostatnia')
    const add = wrapper.findAll('button').find((button) => button.text() === 'Add')
    expect((add!.element as HTMLButtonElement).disabled).toBe(false)
    await add!.trigger('click')

    expect(fieldValues(wrapper)).toHaveLength(60)
    expect(wrapper.text()).toContain('Limit reached')
    expect((add!.element as HTMLButtonElement).disabled).toBe(true)
  })

  it('blokuje zapis, gdy wiersz nie ma etykiety albo tekstu', async () => {
    const wrapper = setup([preset('p1', '/home', '/send /home')])

    await wrapper.get('[data-preset-row] input[aria-label="Label"]').setValue('   ')
    const save = wrapper.findAll('button').find((button) => button.text() === 'Save')

    expect(save!.attributes('disabled')).toBeDefined()
    expect(wrapper.text()).toContain('Every action needs a label and a text.')
  })
})

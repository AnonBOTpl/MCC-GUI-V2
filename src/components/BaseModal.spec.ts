import { afterEach, describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import { h, nextTick } from 'vue'
import BaseModal from './BaseModal.vue'

// Wspólny szkielet modali (R-06): semantyka dialogu, pułapka fokusu, `Esc`,
// klik poza panelem i powrót fokusu. Modale aplikacji nie mają własnych
// nasłuchów klawiatury, więc to jest jedyne miejsce, które je ma.
describe('BaseModal', () => {
  afterEach(() => {
    document.body.innerHTML = ''
  })

  const slots = {
    default: () => [h('input', { id: 'pole' }), h('button', { id: 'srodek' }, 'środek')],
    footer: () => h('button', { id: 'stopka' }, 'stopka'),
  }

  function mountModal(props: Record<string, unknown> = {}) {
    return mount(BaseModal, {
      props: { open: true, label: 'Etykieta', closeLabel: 'Zamknij', ...props },
      slots,
      attachTo: document.body,
    })
  }

  const dialog = (wrapper: ReturnType<typeof mountModal>) => wrapper.get('[role="dialog"]')
  const closeButton = (wrapper: ReturnType<typeof mountModal>) =>
    wrapper.get('button[title="Zamknij"]')

  it('zamknięty nie renderuje dialogu', () => {
    const wrapper = mountModal({ open: false })

    expect(wrapper.find('[role="dialog"]').exists()).toBe(false)
  })

  it('otwarty ma role dialog, aria-modal i nazwę', () => {
    const wrapper = mountModal()

    expect(dialog(wrapper).attributes('role')).toBe('dialog')
    expect(dialog(wrapper).attributes('aria-modal')).toBe('true')
    expect(dialog(wrapper).attributes('aria-label')).toBe('Etykieta')
  })

  it('po otwarciu fokus wchodzi do panelu, a po zamknięciu wraca na element, który go otworzył', async () => {
    const opener = document.createElement('button')
    document.body.append(opener)
    opener.focus()

    const wrapper = mountModal({ open: false })
    await wrapper.setProps({ open: true })
    await nextTick()
    await nextTick()

    expect(document.activeElement).toBe(closeButton(wrapper).element)

    await wrapper.setProps({ open: false })
    await nextTick()

    expect(document.activeElement).toBe(opener)
  })

  it('Tab z ostatniego elementu wraca na pierwszy, a Shift+Tab z pierwszego idzie na ostatni', async () => {
    const wrapper = mountModal()
    await nextTick()

    const footerButton = () => wrapper.get<HTMLButtonElement>('#stopka').element
    footerButton().focus()
    document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Tab', bubbles: true }))
    expect(document.activeElement).toBe(closeButton(wrapper).element)

    footerButton().focus()
    document.dispatchEvent(
      new KeyboardEvent('keydown', { key: 'Tab', shiftKey: true, bubbles: true }),
    )
    expect(document.activeElement).toBe(footerButton())
  })

  it('Tab wciąga fokus z powrotem, gdy jest poza dialogiem', async () => {
    const wrapper = mountModal()
    const outside = document.createElement('button')
    document.body.append(outside)
    outside.focus()

    document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Tab', bubbles: true }))

    expect(document.activeElement).toBe(closeButton(wrapper).element)
  })

  it('Esc zamyka i nie wysyła nic dalej', async () => {
    const wrapper = mountModal()
    const event = new KeyboardEvent('keydown', { key: 'Escape', cancelable: true, bubbles: true })

    document.dispatchEvent(event)
    await nextTick()

    expect(wrapper.emitted('close')).toHaveLength(1)
    expect(event.defaultPrevented).toBe(true)
  })

  it('przy dismissible=false Esc i klik poza panelem nic nie robią', async () => {
    const wrapper = mountModal({ dismissible: false })

    document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }))
    await wrapper.trigger('click')
    await nextTick()

    expect(wrapper.emitted('close')).toBeUndefined()
  })

  it('klik w tło zamyka, a klik w panel nie', async () => {
    const wrapper = mountModal()
    // Tło to korzeń modala; w testach `Transition` jest stubem, więc szukamy
    // nakładki po klasie, a nie przez `wrapper.element`.
    const backdrop = wrapper.get('div.fixed')

    await dialog(wrapper).trigger('click')
    expect(wrapper.emitted('close')).toBeUndefined()

    await backdrop.trigger('click')
    expect(wrapper.emitted('close')).toHaveLength(1)
  })

  it('as="form" zamienia wysłanie formularza na zdarzenie submit', async () => {
    const wrapper = mountModal({ as: 'form' })

    await wrapper.get('form').trigger('submit')

    expect(wrapper.emitted('submit')).toHaveLength(1)
  })
})

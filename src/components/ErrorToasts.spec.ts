import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { i18n } from '../i18n'
import { useAccountsStore } from '../stores/accounts'
import ErrorToasts from './ErrorToasts.vue'

vi.mock('../api/backend', () => ({ backend: {}, events: {} }))

describe('ErrorToasts', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  function mountToasts() {
    return mount(ErrorToasts, { global: { plugins: [i18n] } })
  }

  /**
   * R-20: karty błędów i modale miały to samo `z-50`, a `ErrorToasts` jest
   * wcześniej w DOM (`App.vue`), więc modal malował się nad komunikatem i błąd
   * zapisu w oknie dialogowym był niewidoczny. Dziś toast ma wyższy poziom.
   *
   * jsdom nie liczy Tailwinda (arkusz nie jest wczytany), więc sprawdzamy samą
   * klasę poziomu - to ona decyduje o kolejności malowania w aplikacji.
   */
  it('jest malowany nad modalami (R-20)', () => {
    const container = mountToasts().get('div')

    expect(container.classes()).toContain('z-[60]')
  })

  it('pokazuje kartę błędu i daje ją ukryć', async () => {
    const wrapper = mountToasts()
    const accounts = useAccountsStore()
    accounts.pushError({ code: 'connection.notRunning', params: {} })
    await wrapper.vm.$nextTick()

    expect(wrapper.findAll('[role="alert"]')).toHaveLength(1)

    await wrapper.get('button').trigger('click')
    expect(wrapper.findAll('[role="alert"]')).toHaveLength(0)
  })
})

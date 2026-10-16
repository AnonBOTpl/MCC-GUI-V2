import { describe, expect, it, vi } from 'vitest'
import { WAKE_EVENTS, registerWakeHandlers } from './wake'

// Nasłuch zdarzeń budzących jest jedynym sposobem na domknięcie zaległego
// odświeżenia, gdy Chromium wstrzyma klatki dla okna uznanego za zasłonięte.
// Test pilnuje, że łapiemy wszystkie te zdarzenia i że nie zostawiamy nasłuchu
// po odmontowaniu widoku.

describe('registerWakeHandlers', () => {
  it('woła obsługę na każdym zdarzeniu budzącym', () => {
    const target = new EventTarget()
    const handler = vi.fn()
    registerWakeHandlers(target, handler)

    for (const event of WAKE_EVENTS) {
      target.dispatchEvent(new Event(event))
    }

    expect(handler).toHaveBeenCalledTimes(WAKE_EVENTS.length)
  })

  it('odpinanie zdejmuje wszystkie nasłuchy', () => {
    const target = new EventTarget()
    const handler = vi.fn()
    const dispose = registerWakeHandlers(target, handler)

    dispose()
    for (const event of WAKE_EVENTS) {
      target.dispatchEvent(new Event(event))
    }

    expect(handler).not.toHaveBeenCalled()
  })

  it('łapie zdarzenia z fazy przechwytywania, nie zmieniając ich biegu', () => {
    const target = new EventTarget()
    const own = vi.fn()
    const order: string[] = []
    target.addEventListener('wheel', () => order.push('wlasny'))
    registerWakeHandlers(target, () => order.push('budzenie'))

    target.dispatchEvent(new Event('wheel'))

    expect(order).toEqual(['budzenie', 'wlasny'])
    expect(own).not.toHaveBeenCalled()
  })
})

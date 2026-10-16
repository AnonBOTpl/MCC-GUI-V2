import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useLogsStore } from './logs'

// Store logów zbiera zmiany i pokazuje je widokowi raz na klatkę. Testujemy tu
// przypadek, w którym **klatki nie ma**: okno w tle, zminimalizowane albo
// zasłonięte nie produkuje klatek, więc `requestAnimationFrame` nie woła
// callbacku. Wtedy o odświeżenie musi zadbać pilnowacz (timer) - inaczej log
// zamarza do pierwszej klatki, czyli do momentu, gdy użytkownik ruszy oknem.

describe('useLogsStore - odświeżanie widoku', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.useFakeTimers()
  })

  afterEach(() => {
    vi.useRealTimers()
    vi.unstubAllGlobals()
  })

  it('odświeża widok, gdy okno nie produkuje klatek', () => {
    // Klatka nigdy nie przychodzi - jak w oknie w tle.
    vi.stubGlobal('requestAnimationFrame', () => 0)
    const logs = useLogsStore()
    const before = logs.activeRevision

    logs.append('acc::srv', [{ stream: 'stdout', raw: 'linia', ts: 1 }], 'acc::srv')
    expect(logs.activeRevision).toBe(before)

    vi.advanceTimersByTime(200)
    expect(logs.activeRevision).toBeGreaterThan(before)
    expect(logs.sizeFor('acc::srv')).toBe(1)
  })

  it('scala wiele linii w jedno odświeżenie', () => {
    vi.stubGlobal('requestAnimationFrame', () => 0)
    const logs = useLogsStore()

    logs.append('a', [{ stream: 'stdout', raw: '1', ts: 1 }], 'a')
    logs.append('a', [{ stream: 'stdout', raw: '2', ts: 2 }], 'a')
    logs.append('a', [{ stream: 'stdout', raw: '3', ts: 3 }], 'a')
    vi.advanceTimersByTime(1_000)

    expect(logs.sizeFor('a')).toBe(3)
    expect(logs.activeRevision).toBe(1)
  })

  it('domyka odświeżenie natychmiast na żądanie (powrót do okna)', () => {
    // Okno zminimalizowane/zasłonięte: klatki nie przychodzą, pilnowacz czeka.
    vi.stubGlobal('requestAnimationFrame', () => 0)
    const logs = useLogsStore()
    const before = logs.activeRevision

    logs.append('a', [{ stream: 'stdout', raw: 'x', ts: 1 }], 'a')
    expect(logs.activeRevision).toBe(before)

    // Pierwsze zdarzenie wejścia po powrocie: dane są w buforze, widok ma je dostać od razu.
    logs.flushNow()
    expect(logs.activeRevision).toBe(before + 1)

    // Zaplanowany pilnowacz i klatka nie robią drugiego odświeżenia.
    vi.advanceTimersByTime(1_000)
    expect(logs.activeRevision).toBe(before + 1)
  })

  it('nie odświeża niczego, gdy nie ma zaległości', () => {
    vi.stubGlobal('requestAnimationFrame', () => 0)
    const logs = useLogsStore()
    const before = logs.activeRevision

    logs.flushNow()
    expect(logs.activeRevision).toBe(before)
  })

  it('nie gubi odświeżenia, gdy timer zdąży przed klatką', () => {
    // Klatka przychodzi później niż pilnowacz - drugie wywołanie jest ignorowane.
    let frame: (() => void) | undefined
    vi.stubGlobal('requestAnimationFrame', (callback: () => void) => {
      frame = callback
      return 1
    })
    const logs = useLogsStore()
    const before = logs.activeRevision

    logs.append('a', [{ stream: 'stdout', raw: 'x', ts: 1 }], 'a')
    vi.advanceTimersByTime(200)
    expect(logs.activeRevision).toBe(before + 1)

    frame?.()
    expect(logs.activeRevision).toBe(before + 1)
  })
})

import { describe, expect, it } from 'vitest'
import { enabledPresets, moveItem, newPresetId, rowIndexAt, sanitizePresets } from './model'
import type { Preset } from '../api/contract'

describe('newPresetId', () => {
  it('daje unikalne identyfikatory nawet w tej samej milisekundzie', () => {
    const first = newPresetId(1000)
    const second = newPresetId(1000)
    expect(first).not.toBe(second)
  })
})

describe('moveItem', () => {
  const list = ['a', 'b', 'c', 'd']

  it('przenosi element w dół', () => {
    expect(moveItem(list, 0, 2)).toEqual(['b', 'c', 'a', 'd'])
  })

  it('przenosi element w górę', () => {
    expect(moveItem(list, 3, 1)).toEqual(['a', 'd', 'b', 'c'])
  })

  it('nie zmienia oryginalnej listy', () => {
    const copy = [...list]
    moveItem(list, 0, 3)
    expect(list).toEqual(copy)
  })

  it('ignoruje pozycje spoza zakresu i przenoszenie na siebie', () => {
    expect(moveItem(list, 1, 1)).toEqual(list)
    expect(moveItem(list, -1, 2)).toEqual(list)
    expect(moveItem(list, 0, 9)).toEqual(list)
    expect(moveItem([], 0, 1)).toEqual([])
  })
})

describe('enabledPresets', () => {
  function preset(id: string, enabled: boolean): Preset {
    return { id, label: id, text: '/send /x', color: null, enabled }
  }

  it('przepuszcza tylko akcje włączone', () => {
    const list = [preset('a', true), preset('b', false), preset('c', true)]
    expect(enabledPresets(list).map((item) => item.id)).toEqual(['a', 'c'])
  })

  it('zachowuje kolejność i nie zmienia oryginału', () => {
    const list = [preset('a', false), preset('b', true)]
    expect(enabledPresets(list).map((item) => item.id)).toEqual(['b'])
    expect(list).toHaveLength(2)
  })
})

describe('rowIndexAt', () => {
  const rows = [
    { top: 0, bottom: 30 },
    { top: 40, bottom: 70 },
    { top: 80, bottom: 110 },
  ]

  it('trafia w wiersz pod kursorem', () => {
    expect(rowIndexAt(rows, 0)).toBe(0)
    expect(rowIndexAt(rows, 25)).toBe(0)
    expect(rowIndexAt(rows, 55)).toBe(1)
    expect(rowIndexAt(rows, 110)).toBe(2)
  })

  it('w przerwie wybiera najbliższy wiersz', () => {
    expect(rowIndexAt(rows, 35)).toBe(1)
    expect(rowIndexAt(rows, 39)).toBe(1)
    expect(rowIndexAt(rows, 75)).toBe(2)
  })

  it('poza listą przykleja do pierwszego albo ostatniego wiersza', () => {
    expect(rowIndexAt(rows, -50)).toBe(0)
    expect(rowIndexAt(rows, 500)).toBe(2)
  })

  it('bez wierszy nic nie zwraca', () => {
    expect(rowIndexAt([], 10)).toBeNull()
  })
})

describe('sanitizePresets', () => {
  it('sprowadza poprawny wpis do znanego kształtu', () => {
    expect(
      sanitizePresets([
        { id: 'p1', label: ' /home ', text: ' /send /home ', color: 'green', enabled: false },
      ]),
    ).toEqual([
      { id: 'p1', label: '/home', text: '/send /home', color: 'green', enabled: false },
    ])
  })

  it('uzupełnia brakujące pola, a brak `enabled` nie gasi akcji', () => {
    expect(sanitizePresets([{ id: 'p1', label: 'A', text: '/send /a' }])).toEqual([
      { id: 'p1', label: 'A', text: '/send /a', color: null, enabled: true },
    ])
  })

  it('nieznany kolor nie daje paska', () => {
    const [preset] = sanitizePresets([
      { id: 'p1', label: 'A', text: '/send /a', color: '#ff00ff' },
    ])
    expect(preset.color).toBeNull()
  })

  it('pusta etykieta spada do tekstu, żeby przycisk nie był niewidzialny', () => {
    const [preset] = sanitizePresets([{ id: 'p1', label: '   ', text: '/send /a' }])
    expect(preset.label).toBe('/send /a')
  })

  it('odrzuca wpisy bez identyfikatora albo bez tekstu', () => {
    expect(
      sanitizePresets([
        { id: '', label: 'A', text: '/send /a' },
        { id: 'p2', label: 'B', text: '   ' },
        null,
        'nonsens',
        { id: 'p3', label: 'C', text: '/send /c' },
      ]),
    ).toHaveLength(1)
  })

  it('radzi sobie z danymi, które nie są listą', () => {
    expect(sanitizePresets(null)).toEqual([])
    expect(sanitizePresets({ id: 'p1' })).toEqual([])
  })
})

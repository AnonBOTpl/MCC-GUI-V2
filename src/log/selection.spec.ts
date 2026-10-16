import { describe, expect, it } from 'vitest'
import type { LogLine } from './buffer'
import {
  addRangeFromAnchor,
  allLinesText,
  clearSelection,
  pruneSelection,
  createSelection,
  isEmpty,
  selectAll,
  selectRangeFromAnchor,
  selectRow,
  selectionSize,
  selectionText,
  toggleRow,
} from './selection'

/** Bufor testowy: linie 10..20 (wcześniejsze już wypadły). */
const BOUNDS = { min: 10, max: 20 }

function line(seq: number, raw: string): LogLine {
  return { seq, stream: 'stdout', raw, ts: seq }
}

/** Linie 10..20 z czytelnym tekstem, żeby sprawdzić kolejność kopiowania. */
function lines(): LogLine[] {
  const result: LogLine[] = []
  for (let seq = BOUNDS.min; seq <= BOUNDS.max; seq += 1) {
    result.push(line(seq, `linia ${seq}`))
  }
  return result
}

/** Zaznaczone numery w kolejności rosnącej (do porównań w testach). */
function selectedNumbers(selection: { selected: Set<number> }): number[] {
  return [...selection.selected].sort((a, b) => a - b)
}

describe('zaznaczanie wierszy logu', () => {
  it('klik zastępuje zaznaczenie i ustawia kotwicę', () => {
    const selection = createSelection()
    selectRow(selection, 12)
    selectRow(selection, 15)

    expect(selectedNumbers(selection)).toEqual([15])
    expect(selection.anchor).toBe(15)
  })

  it('Ctrl+klik dodaje i odejmuje wiersz, a kotwicą zostaje kliknięty wiersz', () => {
    const selection = createSelection()
    selectRow(selection, 10)

    toggleRow(selection, 12)
    toggleRow(selection, 14)
    expect(selectedNumbers(selection)).toEqual([10, 12, 14])
    expect(selection.anchor).toBe(14)

    toggleRow(selection, 12)
    expect(selectedNumbers(selection)).toEqual([10, 14])
    expect(selection.anchor).toBe(12)
  })

  it('Shift+klik zaznacza zakres od kotwicy, zastępując zaznaczenie', () => {
    const selection = createSelection()
    selectRow(selection, 13)
    toggleRow(selection, 10) // kotwica = 10

    selectRangeFromAnchor(selection, 15, BOUNDS)

    expect(selectedNumbers(selection)).toEqual([10, 11, 12, 13, 14, 15])
    // Kotwica zostaje na miejscu, więc kolejny Shift+klik rozciąga ten sam zakres.
    expect(selection.anchor).toBe(10)
    selectRangeFromAnchor(selection, 12, BOUNDS)
    expect(selectedNumbers(selection)).toEqual([10, 11, 12])
  })

  it('Shift+klik w górę też działa (zakres jest niezależny od kierunku)', () => {
    const selection = createSelection()
    selectRow(selection, 18)
    selectRangeFromAnchor(selection, 15, BOUNDS)
    expect(selectedNumbers(selection)).toEqual([15, 16, 17, 18])
  })

  it('bez kotwicy Shift+klik zaznacza sam kliknięty wiersz', () => {
    const selection = createSelection()
    selectRangeFromAnchor(selection, 16, BOUNDS)
    expect(selectedNumbers(selection)).toEqual([16])
  })

  it('Ctrl+Shift+klik dodaje zakres do istniejącego zaznaczenia', () => {
    const selection = createSelection()
    selectRow(selection, 10)
    toggleRow(selection, 14) // kotwica = 14

    addRangeFromAnchor(selection, 16, BOUNDS)

    // Zaznaczenie z Ctrl+klik (10 i 14) plus zakres 14..16.
    expect(selectedNumbers(selection)).toEqual([10, 14, 15, 16])
    expect(selection.anchor).toBe(14)
  })

  it('przycina zakres, gdy kotwica wypadła już z bufora', () => {
    const selection = createSelection()
    // Kotwica to linia 2, której w buforze już nie ma (min = 10).
    selectRow(selection, 2)
    selectRangeFromAnchor(selection, 14, BOUNDS)

    expect(selectedNumbers(selection)).toEqual([10, 11, 12, 13, 14])
  })

  it('Ctrl+A zaznacza cały bufor, Esc czyści zaznaczenie', () => {
    const selection = createSelection()
    selectAll(
      selection,
      lines().map((item) => item.seq),
    )

    expect(selectionSize(selection)).toBe(11)
    expect(selection.anchor).toBe(10)
    expect(isEmpty(selection)).toBe(false)

    clearSelection(selection)
    expect(isEmpty(selection)).toBe(true)
    expect(selection.anchor).toBeNull()
  })

  it('kopiuje zaznaczone linie w kolejności bufora, bez kodów kolorów', () => {
    const selection = createSelection()
    const withColors = [
      line(10, '\u00a7aHello \u00a7bworld'),
      line(11, '\u001b[31mczerwony\u001b[0m'),
      line(12, 'zwykła linia'),
    ]

    // Zaznaczamy w odwrotnej kolejności, niż leżą w buforze.
    selectRow(selection, 12)
    toggleRow(selection, 10)

    expect(selectionText(withColors, selection)).toBe('Hello world\nzwykła linia')
  })

  it('po obcięciu bufora kopiuje to, co zostało', () => {
    const selection = createSelection()
    selectAll(
      selection,
      lines().map((item) => item.seq),
    )

    // Bufor przytracił najstarsze linie - wypadły z niego 10..14.
    const trimmed = lines().slice(5)
    expect(selectionText(trimmed, selection)).toBe(
      'linia 15\nlinia 16\nlinia 17\nlinia 18\nlinia 19\nlinia 20',
    )
    // Zaznaczenie nadal pamięta obcięte numery - czyszczenie usuwa je zbiorem.
    expect(selectionSize(selection)).toBe(11)
  })

  it('usuwa z zaznaczenia linie, które wypadły z bufora', () => {
    const selection = createSelection()
    selectRow(selection, 10) // kotwica
    toggleRow(selection, 13)
    toggleRow(selection, 17)

    // Bufor przytracił 10 i 17 (nowe 23, czyli kotwica też wypadła).
    pruneSelection(selection, lines().filter((item) => item.seq === 13))

    expect(selectedNumbers(selection)).toEqual([13])
    expect(selection.anchor).toBeNull()

    // Gdy wypadło wszystko, zaznaczenie staje się puste.
    pruneSelection(selection, [])
    expect(isEmpty(selection)).toBe(true)
  })

  it('puste zaznaczenie daje pusty tekst do schowka', () => {
    expect(selectionText(lines(), createSelection())).toBe('')
  })

  it('„Kopiuj wszystko” zwraca cały bufor bez kodów kolorów', () => {
    const all = [line(1, '\u00a7cstart'), line(2, '\u001b[32mśrodek\u001b[0m'), line(3, 'koniec')]
    expect(allLinesText(all)).toBe('start\nśrodek\nkoniec')
    expect(allLinesText([])).toBe('')
  })
})

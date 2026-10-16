import { describe, expect, it } from 'vitest'
import { displayWidth, wrapRowCount } from './wrap'

describe('displayWidth', () => {
  it('liczy zwykłe znaki jako jedną kolumnę', () => {
    expect(displayWidth('abc')).toBe(3)
    expect(displayWidth('Hello, world!')).toBe(13)
  })

  it('liczy znaki CJK jako dwie kolumny', () => {
    expect(displayWidth('你好')).toBe(4)
    expect(displayWidth('a你b')).toBe(4)
  })

  it('liczy emoji spoza BMP jako dwie kolumny', () => {
    expect(displayWidth('😀')).toBe(2)
  })

  it('ignoruje znaki łączące i zerowej szerokości', () => {
    expect(displayWidth('a\u0301')).toBe(1)
    expect(displayWidth('a\u200Bb')).toBe(2)
  })

  it('zwraca zero dla pustego tekstu', () => {
    expect(displayWidth('')).toBe(0)
  })
})

describe('wrapRowCount', () => {
  it('pusty tekst zajmuje jeden wiersz', () => {
    expect(wrapRowCount('', 10)).toBe(1)
  })

  it('tekst mieszczący się w oknie zajmuje jeden wiersz', () => {
    expect(wrapRowCount('abc', 10)).toBe(1)
    expect(wrapRowCount('a'.repeat(10), 10)).toBe(1)
  })

  it('zawija tekst dłuższy niż okno', () => {
    expect(wrapRowCount('a'.repeat(11), 10)).toBe(2)
    expect(wrapRowCount('a'.repeat(20), 10)).toBe(2)
    expect(wrapRowCount('a'.repeat(21), 10)).toBe(3)
  })

  it('uwzględnia podwójną szerokość znaków', () => {
    // 4 znaki CJK = 8 kolumn; przy 3 kolumnach to 3 wiersze.
    expect(wrapRowCount('你好世界', 3)).toBe(3)
  })

  it('bezpiecznie obsługuje zerową liczbę kolumn', () => {
    expect(wrapRowCount('abc', 0)).toBe(1)
  })
})

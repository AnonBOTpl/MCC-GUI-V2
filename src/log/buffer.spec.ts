import { describe, expect, it } from 'vitest'
import { DEFAULT_LOG_LIMIT, LogBuffer, pruneMeasured } from './buffer'

describe('pruneMeasured (R-12)', () => {
  it('usuwa pomiary linii, których bufor już nie ma', () => {
    const measured = new Map([
      [0, 18],
      [1, 36],
      [2, 18],
    ])
    expect(pruneMeasured(measured, 1)).toBe(1)
    expect([...measured.keys()]).toEqual([1, 2])
  })

  it('pusty bufor czyści całą mapę', () => {
    const measured = new Map([[3, 18]])
    expect(pruneMeasured(measured, null)).toBe(1)
    expect(measured.size).toBe(0)
  })

  it('nie rusza mapy, gdy nic nie wypadło', () => {
    const measured = new Map([
      [2, 18],
      [3, 18],
    ])
    expect(pruneMeasured(measured, 2)).toBe(0)
    expect(measured.size).toBe(2)
  })
})

describe('LogBuffer', () => {
  it('dopisuje linie i numeruje je rosnąco', () => {
    const buffer = new LogBuffer(10)
    expect(buffer.push('stdout', 'a', 1)).toBe(0)
    expect(buffer.push('stdout', 'b', 2)).toBe(1)
    expect(buffer.size).toBe(2)
    expect(buffer.lastSeq).toBe(1)
  })

  it('pushMany zwraca numery wszystkich linii partii', () => {
    const buffer = new LogBuffer(10)
    const seqs = buffer.pushMany([
      { stream: 'stdout', raw: 'a', ts: 1 },
      { stream: 'stderr', raw: 'b', ts: 2 },
    ])
    expect(seqs).toEqual([0, 1])
    expect(buffer.all().map((line) => line.raw)).toEqual(['a', 'b'])
    expect(buffer.all()[1].stream).toBe('stderr')
  })

  it('nie przekracza limitu i usuwa najstarsze linie', () => {
    const buffer = new LogBuffer(3)
    buffer.pushMany(
      Array.from({ length: 5 }, (_, index) => ({ stream: 'stdout' as const, raw: `l${index}`, ts: index })),
    )
    expect(buffer.size).toBe(3)
    expect(buffer.all().map((line) => line.raw)).toEqual(['l2', 'l3', 'l4'])
  })

  it('zachowuje logiczną numerację po obcięciu', () => {
    const buffer = new LogBuffer(2)
    buffer.pushMany(
      Array.from({ length: 5 }, (_, index) => ({ stream: 'stdout' as const, raw: `l${index}`, ts: index })),
    )
    expect(buffer.firstSeq).toBe(3)
    expect(buffer.lastSeq).toBe(4)
    expect(buffer.all()[0].seq).toBe(3)
  })

  it('slice zwraca wycinek do wirtualizacji', () => {
    const buffer = new LogBuffer(100)
    buffer.pushMany(
      Array.from({ length: 10 }, (_, index) => ({ stream: 'stdout' as const, raw: `l${index}`, ts: index })),
    )
    expect(buffer.slice(2, 4).map((line) => line.raw)).toEqual(['l2', 'l3'])
    expect(buffer.slice(20, 30)).toEqual([])
  })

  it('clear czyści linie, ale nie numeracji globalnej', () => {
    const buffer = new LogBuffer(10)
    buffer.push('stdout', 'a', 1)
    buffer.clear()
    expect(buffer.size).toBe(0)
    expect(buffer.push('stdout', 'b', 2)).toBe(1)
  })

  it('odrzuca niedodatni limit', () => {
    expect(() => new LogBuffer(0)).toThrow()
  })

  it('setLimit obcina nadmiarowe linie i zachowuje najnowsze', () => {
    const buffer = new LogBuffer(10)
    buffer.pushMany(
      Array.from({ length: 8 }, (_, index) => ({
        stream: 'stdout' as const,
        raw: `line ${index}`,
        ts: index,
      })),
    )
    buffer.setLimit(3)
    expect(buffer.size).toBe(3)
    expect(buffer.all().map((line) => line.raw)).toEqual(['line 5', 'line 6', 'line 7'])
    expect(buffer.firstSeq).toBe(5)
    expect(buffer.lastSeq).toBe(7)
  })

  it('setLimit ignoruje wartość niepoprawną', () => {
    const buffer = new LogBuffer(10)
    buffer.setLimit(0)
    expect(buffer.size).toBe(0)
    buffer.push('stdout', 'a', 1)
    expect(buffer.limitSize).toBe(10)
  })

  it('domyślny limit wynosi 5000 linii', () => {
    expect(DEFAULT_LOG_LIMIT).toBe(5000)
  })

  it('przetwarza 200 linii/s przez minutę bez przekroczenia limitu', () => {
    const buffer = new LogBuffer(DEFAULT_LOG_LIMIT)
    for (let second = 0; second < 60; second += 1) {
      buffer.pushMany(
        Array.from({ length: 200 }, (_, index) => ({
          stream: 'stdout' as const,
          raw: `tick ${second}:${index}`,
          ts: second * 1000 + index,
        })),
      )
    }
    expect(buffer.size).toBe(DEFAULT_LOG_LIMIT)
    expect(buffer.lastSeq).toBe(12_000 - 1)
    expect(buffer.firstSeq).toBe(12_000 - DEFAULT_LOG_LIMIT)
  })
})

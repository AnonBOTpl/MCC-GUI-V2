// Ograniczony bufor linii logu jednego konta.
//
// Priorytet: przy ~200 liniach/s na konto UI musi zostać płynne,
// a pamięć nie może rosnąć bez końca. Bufor trzyma ostatnie N linii; po
// przekroczeniu limitu najstarsze są usuwane partiami, żeby nie przesuwać
// tablicy przy każdej linii.

import type { LogStream } from '../api/contract'

/** Jedna linia logu po stronie frontendu. */
export interface LogLine {
  /** Rosnący identyfikator w obrębie konta - klucz wirtualnej listy. */
  seq: number
  stream: LogStream
  /** Surowa linia z MCC (z kodami `§`) - formatowanie dopiero przy renderowaniu. */
  raw: string
  ts: number
}

/** Domyślny limit linii na konto (`logBufferSize`, domyślnie 5000). */
export const DEFAULT_LOG_LIMIT = 5000

/**
 * Bufor linii. `push`/`pushMany` przyjmują już gotowe linie, więc bufor nie zna
 * źródła danych i można go testować bez Tauri.
 */
export class LogBuffer {
  private lines: LogLine[] = []
  private nextSeq = 0
  /** Ile linii wypadło z bufora w sumie - pozycja logiczna pierwszej linii. */
  private dropped = 0

  constructor(private limit: number = DEFAULT_LOG_LIMIT) {
    if (limit <= 0) {
      throw new Error('limit bufora logu musi być dodatni')
    }
  }

  /**
   * Zmienia limit (np. po zmianie ustawienia) i od razu obcina nadmiarowe linie.
   * Wartości niepoprawne ignorujemy - bufor ma działać dalej, a poprawność
   * ustawienia i tak sprawdza Rust.
   */
  setLimit(limit: number): void {
    if (!Number.isFinite(limit) || limit <= 0) return
    this.limit = limit
    this.trim()
  }

  /** Bieżący limit linii. */
  get limitSize(): number {
    return this.limit
  }

  /** Dopisuje jedną linię i zwraca jej numer. */
  push(stream: LogStream, raw: string, ts: number): number {
    return this.pushMany([{ stream, raw, ts }])[0]
  }

  /**
   * Dopisuje wiele linii naraz (jedna aktualizacja zamiast wielu - mniej
   * przerysowań UI). Zwraca numery dopisanych linii.
   */
  pushMany(batch: ReadonlyArray<{ stream: LogStream; raw: string; ts: number }>): number[] {
    const seqs: number[] = []
    for (const item of batch) {
      const seq = this.nextSeq
      this.nextSeq += 1
      this.lines.push({ seq, stream: item.stream, raw: item.raw, ts: item.ts })
      seqs.push(seq)
    }
    this.trim()
    return seqs
  }

  /** Usuwa najstarsze linie, gdy przekroczymy limit. */
  private trim(): void {
    const overflow = this.lines.length - this.limit
    if (overflow <= 0) return
    this.lines.splice(0, overflow)
    this.dropped += overflow
  }

  /** Wszystkie linie w buforze (od najstarszej). */
  all(): readonly LogLine[] {
    return this.lines
  }

  /** Linie od indeksu włącznie - używane przez wirtualizowaną listę. */
  slice(start: number, end: number): LogLine[] {
    return this.lines.slice(start, end)
  }

  /** Liczba linii w buforze. */
  get size(): number {
    return this.lines.length
  }

  /** Numer logiczny pierwszej linii w buforze (linie wcześniejsze wypadły). */
  get firstSeq(): number {
    return this.dropped
  }

  /** Numer logiczny ostatniej linii albo -1, gdy bufor jest pusty. */
  get lastSeq(): number {
    return this.nextSeq - 1
  }

  /** Czyści bufor (np. po rozłączeniu konta). */
  clear(): void {
    this.lines = []
  }
}

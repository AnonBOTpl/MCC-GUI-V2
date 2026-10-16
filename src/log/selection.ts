// Zaznaczanie wierszy logu (Paczka 3).
//
// Zaznaczamy **wiersze po numerze `seq`**, a nie natywnym zaznaczaniem tekstu,
// bo lista wirtualizowana trzyma w DOM tylko widoczny wycinek - natywne
// zaznaczenie znikałoby przy przewijaniu. Numer `seq` rośnie monotonicznie
// w obrębie połączenia i nie przesuwa się przy obcinaniu bufora
// (`src/log/buffer.ts`), więc zaznaczenie przeżywa dopisywanie linii.
//
// Moduł jest czysty (bez Vue i bez Tauri), żeby dało się go testować osobno.

import type { LogLine } from './buffer'
import { stripFormatting } from './color'

/** Zakres numerów linii, które **są jeszcze w buforze**. */
export interface SeqBounds {
  min: number
  max: number
}

/** Zaznaczenie jednego połączenia. */
export interface RowSelection {
  /** Wiersz, od którego liczą się zakresy (`Shift+klik`). */
  anchor: number | null
  /** Numery wybranych wierszy. */
  selected: Set<number>
}

/** Puste zaznaczenie. */
export function createSelection(): RowSelection {
  return { anchor: null, selected: new Set<number>() }
}

/** Klik: zaznacza jeden wiersz, zastępując poprzednie zaznaczenie. */
export function selectRow(selection: RowSelection, seq: number): void {
  selection.selected.clear()
  selection.selected.add(seq)
  selection.anchor = seq
}

/** `Ctrl+klik`: dodaje wiersz do zaznaczenia albo go odejmuje. */
export function toggleRow(selection: RowSelection, seq: number): void {
  if (selection.selected.has(seq)) selection.selected.delete(seq)
  else selection.selected.add(seq)
  selection.anchor = seq
}

/**
 * `Shift+klik`: zakres od kotwicy do klikniętego wiersza, zastępuje zaznaczenie.
 * Kotwica **nie zmienia się**, więc kolejne `Shift+klik` rozciągają ten sam
 * zakres (jak w edytorach). Gdy kotwicy nie ma, zakresem jest sam kliknięty wiersz.
 */
export function selectRangeFromAnchor(selection: RowSelection, to: number, bounds: SeqBounds): void {
  selection.selected.clear()
  fillRange(selection, selection.anchor ?? to, to, bounds)
}

/** `Ctrl+Shift+klik`: zakres od kotwicy do klikniętego wiersza, dodany do zaznaczenia. */
export function addRangeFromAnchor(selection: RowSelection, to: number, bounds: SeqBounds): void {
  fillRange(selection, selection.anchor ?? to, to, bounds)
}

/** Dopisuje zakres `from`..`to` (w dowolnej kolejności) do zaznaczenia. */
function fillRange(
  selection: RowSelection,
  from: number,
  to: number,
  bounds: SeqBounds,
): void {
  // Kotwica może być już poza buforem (obcięta), a kliknięty wiersz zawsze jest
  // w buforze - przycinamy oba końce, żeby nie budować zakresu po liniach,
  // których już nie ma (i żeby pętla nie biegła przez miliony numerów).
  const clampedFrom = clampSeq(from, bounds)
  const clampedTo = clampSeq(to, bounds)
  const low = Math.min(clampedFrom, clampedTo)
  const high = Math.max(clampedFrom, clampedTo)
  for (let seq = low; seq <= high; seq += 1) {
    selection.selected.add(seq)
  }
}

/** `Ctrl+A`: cały bufor połączenia (numery linii, które w nim zostały). */
export function selectAll(selection: RowSelection, seqs: readonly number[]): void {
  selection.selected.clear()
  for (const seq of seqs) selection.selected.add(seq)
  selection.anchor = seqs.length > 0 ? seqs[0] : null
}

/** `Esc`: czyści zaznaczenie. */
export function clearSelection(selection: RowSelection): void {
  selection.selected.clear()
  selection.anchor = null
}

/** Liczba zaznaczonych wierszy. */
export function selectionSize(selection: RowSelection): number {
  return selection.selected.size
}

/** Czy nic nie jest zaznaczone. */
export function isEmpty(selection: RowSelection): boolean {
  return selection.selected.size === 0
}

/**
 * Tekst do schowka: zaznaczone linie w kolejności bufora, bez kodów `§` i ANSI,
 * rozdzielone znakiem nowej linii. Linie, które wypadły z bufora, po prostu nie
 * występują w `lines` - kopiujemy to, co zostało.
 */
export function selectionText(lines: readonly LogLine[], selection: RowSelection): string {
  const parts: string[] = []
  for (const line of lines) {
    if (selection.selected.has(line.seq)) parts.push(stripFormatting(line.raw))
  }
  return parts.join('\n')
}

/**
 * Usuwa z zaznaczenia numery linii, których nie ma już w buforze.
 *
 * Bufor obcina najstarsze linie, a zaznaczenie pamięta numery - bez tego kroku
 * po obcięciu **wszystkich** zaznaczonych linii zaznaczenie zostałoby niepuste
 * (nic nie widać, a „Kopiuj” nie ma czego skopiować).
 */
export function pruneSelection(selection: RowSelection, lines: readonly LogLine[]): void {
  const live = new Set<number>()
  for (const line of lines) live.add(line.seq)
  for (const seq of [...selection.selected]) {
    if (!live.has(seq)) selection.selected.delete(seq)
  }
  if (selection.anchor !== null && !live.has(selection.anchor)) selection.anchor = null
}

/** Cały bufor połączenia jako tekst do schowka („Kopiuj wszystko”). */
export function allLinesText(lines: readonly LogLine[]): string {
  return lines.map((line) => stripFormatting(line.raw)).join('\n')
}

/** Sprowadza numer linii do zakresu bufora. */
function clampSeq(seq: number, bounds: SeqBounds): number {
  if (bounds.max < bounds.min) return bounds.min
  return Math.min(bounds.max, Math.max(bounds.min, seq))
}

// Czyste operacje na liście presetów szybkich akcji.
//
// Trzymamy je poza komponentami i store'em, bo dotyczą danych, a nie UI:
// - `sanitizePresets` chroni widok przed ręcznie edytowanym `presets.json`
//   (wczytanie w Rustcie jest celowo łagodne - patrz `store.rs`),
// - `enabledPresets` wybiera to, co naprawdę widać na pasku,
// - `moveItem` i `rowIndexAt` obsługują kolejność z edytora (przeciąganie wiersza),
// - `newPresetId` nadaje nowym akcjom unikalne identyfikatory.

import { asPresetColor } from './palette'
import type { Preset } from '../api/contract'

let idSeq = 0

/**
 * Górny limit liczby akcji. Musi zgadzać się z `MAX_PRESETS` w
 * `src-tauri/src/preset.rs` - Rust odrzuca dłuższą listę przy zapisie, a edytor
 * blokuje dodanie wiersza już przy limicie (R-38), żeby błąd nie wyskakiwał
 * dopiero po kliknięciu „Zapisz”.
 */
export const MAX_PRESETS = 60

/**
 * Nowy identyfikator presetu. Licznik w module gwarantuje unikalność nawet
 * wtedy, gdy dwie akcje powstaną w tej samej milisekundzie.
 */
export function newPresetId(now: number = Date.now()): string {
  idSeq += 1
  return `preset-${now.toString(36)}-${idSeq}`
}

/** Sprawdza jeden wpis z pliku i sprowadza go do znanego kształtu. */
function sanitizePreset(raw: unknown): Preset | null {
  if (typeof raw !== 'object' || raw === null) return null
  const candidate = raw as Record<string, unknown>

  const id = typeof candidate.id === 'string' ? candidate.id.trim() : ''
  const text = typeof candidate.text === 'string' ? candidate.text.trim() : ''
  // Wpisu bez identyfikatora albo bez tekstu nie da się ani pokazać, ani zapisać.
  if (id === '' || text === '') return null

  const label = typeof candidate.label === 'string' ? candidate.label.trim() : ''
  return {
    id,
    // Pusta etykieta (ręczna edycja pliku) nie może dać niewidzialnego przycisku.
    label: label === '' ? text : label,
    text,
    color: asPresetColor(candidate.color),
    // Brak pola w pliku nie może gasić akcji - wyłączenie jest świadomą decyzją.
    enabled: candidate.enabled !== false,
  }
}

/** To, co trafia na pasek: akcje wyłączone zostają w edytorze i są ukryte. */
export function enabledPresets(presets: readonly Preset[]): Preset[] {
  return presets.filter((preset) => preset.enabled)
}

/** Sprowadza dane z pliku do listy, którą widok może bezpiecznie pokazać. */
export function sanitizePresets(raw: unknown): Preset[] {
  if (!Array.isArray(raw)) return []
  return raw.map(sanitizePreset).filter((preset): preset is Preset => preset !== null)
}

/**
 * Przenosi element z pozycji `from` na `to`, zwracając **nową** tablicę.
 * Pozycja spoza zakresu albo przenoszenie na samo siebie zwraca kopię bez zmian,
 * więc upuszczenie wiersza poza listę niczego nie psuje.
 */
export function moveItem<T>(items: readonly T[], from: number, to: number): T[] {
  const copy = [...items]
  if (from === to) return copy
  if (from < 0 || to < 0 || from >= copy.length || to >= copy.length) return copy

  const moved = copy[from]
  if (moved === undefined) return copy
  copy.splice(from, 1)
  copy.splice(to, 0, moved)
  return copy
}

/** Pionowy pas wiersza (z `getBoundingClientRect`, bez zależności od DOM). */
export interface RowBand {
  top: number
  bottom: number
}

/**
 * Wiersz pod kursorem w trakcie przeciągania.
 *
 * Bierzemy wiersz, którego pas zawiera `y`, a gdy kursor jest w przerwie między
 * wierszami albo poza listą - najbliższy pas. Dzięki temu upuszczenie nad samą
 * krawędzią działa, a przeciąganie nie „przeskakuje” o dwa wiersze.
 */
export function rowIndexAt(rows: readonly RowBand[], y: number): number | null {
  if (rows.length === 0) return null

  for (const [index, row] of rows.entries()) {
    if (y >= row.top && y <= row.bottom) return index
  }

  // Przy równej odległości wybieramy wiersz **niżej**: gdy kursor minie połowę
  // przerwy, przeciągany element idzie w dół, a nie zostaje na miejscu.
  let best = 0
  let bestDistance = Number.POSITIVE_INFINITY
  for (const [index, row] of rows.entries()) {
    const distance = y < row.top ? row.top - y : y - row.bottom
    if (distance <= bestDistance) {
      bestDistance = distance
      best = index
    }
  }
  return best
}

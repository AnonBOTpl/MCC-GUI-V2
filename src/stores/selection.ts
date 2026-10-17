// Store zaznaczenia wierszy logu (Paczka 3).
//
// Trzymamy je **osobno dla każdego połączenia** (klucz = identyfikator połączenia,
// tak samo jak statusy i logi), dzięki czemu przełączenie zakładki nie gubi
// zaznaczenia i nie przenosi go na inne konto.
//
// Cały stan to reaktywne obiekty (`reactive`) trzymane w mapie: zbiór `Set`
// w środku jest reaktywny, więc widok sam przerysowuje podświetlenie po zmianie
// zaznaczenia, bez ręcznego licznika wersji.

import { defineStore } from 'pinia'
import { reactive, ref } from 'vue'
import {
  addRangeFromAnchor as addRangeFromAnchorIn,
  clearSelection,
  createSelection,
  isEmpty as isEmptyIn,
  selectAll as selectAllIn,
  selectRangeFromAnchor as selectRangeFromAnchorIn,
  selectRow as selectRowIn,
  selectionSize,
  toggleRow as toggleRowIn,
  type RowSelection,
  type SeqBounds,
} from '../log/selection'

export const useSelectionStore = defineStore('selection', () => {
  const states = ref(new Map<string, RowSelection>())

  /** Zaznaczenie połączenia; tworzone przy pierwszym użyciu. */
  function forConnection(connectionId: string): RowSelection {
    let state = states.value.get(connectionId)
    if (!state) {
      state = reactive(createSelection())
      states.value.set(connectionId, state)
    }
    return state
  }

  /** Klik: jeden wiersz, zastępuje poprzednie zaznaczenie. */
  function selectRow(connectionId: string, seq: number): void {
    selectRowIn(forConnection(connectionId), seq)
  }

  /** `Ctrl+klik`: dodaje wiersz albo go odejmuje. */
  function toggleRow(connectionId: string, seq: number): void {
    toggleRowIn(forConnection(connectionId), seq)
  }

  /** `Shift+klik`: zakres od kotwicy do klikniętego wiersza. */
  function selectRangeFromAnchor(connectionId: string, to: number, bounds: SeqBounds): void {
    selectRangeFromAnchorIn(forConnection(connectionId), to, bounds)
  }

  /** `Ctrl+Shift+klik`: zakres od kotwicy dodany do zaznaczenia. */
  function addRangeFromAnchor(connectionId: string, to: number, bounds: SeqBounds): void {
    addRangeFromAnchorIn(forConnection(connectionId), to, bounds)
  }

  /** `Ctrl+A`: cały bufor połączenia. */
  function selectAll(connectionId: string, seqs: readonly number[]): void {
    selectAllIn(forConnection(connectionId), seqs)
  }

  /** `Esc` i zamknięcie menu: czyści zaznaczenie. */
  function clear(connectionId: string): void {
    clearSelection(forConnection(connectionId))
  }

  /**
   * Usuwa cały stan zaznaczenia połączenia - wołane, gdy zakładka znika (R-41).
   * `clear` zostawiłoby pusty wpis w mapie, a ten rośnie z każdym otwartym
   * i zamkniętym połączeniem.
   */
  function drop(connectionId: string): void {
    states.value.delete(connectionId)
  }

  /** Czy połączenie ma zapisany jakikolwiek stan zaznaczenia. */
  function has(connectionId: string): boolean {
    return states.value.has(connectionId)
  }

  function sizeFor(connectionId: string): number {
    return selectionSize(forConnection(connectionId))
  }

  function isEmpty(connectionId: string): boolean {
    return isEmptyIn(forConnection(connectionId))
  }

  return {
    forConnection,
    selectRow,
    toggleRow,
    selectRangeFromAnchor,
    addRangeFromAnchor,
    selectAll,
    clear,
    drop,
    has,
    sizeFor,
    isEmpty,
  }
})

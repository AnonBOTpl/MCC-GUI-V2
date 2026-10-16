// Store logów: per konto ograniczony bufor + partia aktualizacji.
//
// Dlaczego partia: przy ~200 liniach/s na konto każda linia osobno oznaczałaby
// setki przerysowań Vue na sekundę. Zamiast tego linie lądują w buforze od razu
// (dane są kompletne), a Vue dostaje jedną zmianę `revision` na klatkę
// (~60 razy/s). Dzięki temu UI pozostaje płynne.

import { defineStore } from 'pinia'
import { markRaw, ref, shallowRef } from 'vue'
import type { LogStream } from '../api/contract'
import { DEFAULT_LOG_LIMIT, LogBuffer, type LogLine } from '../log/buffer'
import { diagCounters } from '../log/diag'

/**
 * Ile najdłużej czekamy na klatkę, zanim odświeżymy widok mimo wszystko.
 * Gdy okno jest w tle, zminimalizowane albo zasłonięte, przeglądarka nie
 * produkuje klatek i `requestAnimationFrame` nie woła callbacku - bez tego
 * pilnowacza log zamarzałby na cały czas, gdy aplikacja siedzi w tle.
 */
const FLUSH_FALLBACK_MS = 200

/** Zbiorczy stan jednego konta w store logów. */
interface LogState {
  buffer: LogBuffer
  revision: number
  unread: number
}

export const useLogsStore = defineStore('logs', () => {
  // `shallowRef` + `markRaw`: mapa i bufory to duże, mutowalne obiekty - nie
  // chcemy, żeby Vue robił z nich reaktywne proxy (koszt przy tysiącach linii).
  // Zmiany sygnalizujemy licznikiem `revision`.
  const states = shallowRef(new Map<string, LogState>())
  const revision = shallowRef(0)

  /** Linie widoczne dla aktywnego konta; zmienia się raz na klatkę. */
  const activeRevision = ref(0)

  /** Czy odświeżenie jest już zaplanowane (jedno na klatkę, nie na linię). */
  let flushScheduled = false
  /** Uchwyt pilnowacza; `undefined`, gdy nic nie czeka na odświeżenie. */
  let fallbackTimer: ReturnType<typeof setTimeout> | undefined
  /** Uchwyt oczekującej klatki; `undefined`, gdy nic nie czeka na odświeżenie. */
  let frameHandle: number | undefined

  /** Bieżący limit linii na konto. Ustawiany z ustawień aplikacji przy starcie
   * i przy każdej ich zmianie (`setLimit`), a nie reaktywnie - bufory i tak
   * odświeżają widok licznikiem `revision`. */
  let currentLimit = DEFAULT_LOG_LIMIT

  function stateFor(accountId: string): LogState {
    let state = states.value.get(accountId)
    if (!state) {
      state = { buffer: markRaw(new LogBuffer(currentLimit)), revision: 0, unread: 0 }
      states.value.set(accountId, state)
    }
    return state
  }

  /**
   * Ustawia limit linii dla wszystkich kont i obcina istniejące bufory.
   * Dzięki temu ustawienie z modala działa od razu, bez restartu aplikacji.
   */
  function setLimit(limit: number): void {
    if (!Number.isFinite(limit) || limit <= 0) return
    currentLimit = limit
    for (const state of states.value.values()) {
      state.buffer.setLimit(limit)
    }
    scheduleFlush()
  }

  /**
   * Jedna aktualizacja na klatkę, niezależnie od liczby dopisanych linii.
   *
   * Odświeżamy na klatce animacji, ale **z pilnowaczem**: `requestAnimationFrame`
   * nie przychodzi, gdy okno nie produkuje klatek (aplikacja w tle, okno
   * zminimalizowane albo zasłonięte). Bez pilnowacza dane dochodziłyby do bufora,
   * a widok zostałby zamrożony do pierwszej klatki - czyli do momentu, w którym
   * użytkownik ruszy oknem. Wygrywa ten, kto zdąży pierwszy.
   */
  /**
   * Wykonuje zaplanowane odświeżenie i domyka **oba** oczekujące wywołania -
   * klatkę i pilnowacza. Bez oczekującego odświeżenia nic nie robi, żeby widok
   * nie przerysowywał się bez powodu.
   */
  function runFlush(kind: 'frame' | 'timer' | 'now'): void {
    if (!flushScheduled) return
    flushScheduled = false
    if (frameHandle !== undefined && typeof cancelAnimationFrame === 'function') {
      cancelAnimationFrame(frameHandle)
    }
    frameHandle = undefined
    if (fallbackTimer !== undefined) {
      clearTimeout(fallbackTimer)
      fallbackTimer = undefined
    }
    revision.value += 1
    activeRevision.value = revision.value
    // Diagnostyka: kto domknął odświeżenie (patrz `src/log/diag.ts`).
    if (kind === 'frame') diagCounters.flushFrame += 1
    else if (kind === 'timer') diagCounters.flushTimer += 1
    else diagCounters.flushNow += 1
  }

  function scheduleFlush(): void {
    if (flushScheduled) return
    flushScheduled = true
    if (typeof requestAnimationFrame === 'function') {
      frameHandle = requestAnimationFrame(() => {
        frameHandle = undefined
        runFlush('frame')
      })
    }
    if (typeof setTimeout === 'function') {
      fallbackTimer = setTimeout(() => {
        fallbackTimer = undefined
        runFlush('timer')
      }, FLUSH_FALLBACK_MS)
    } else if (typeof requestAnimationFrame !== 'function') {
      // Środowisko bez animacji i bez timerów - nie ma na co czekać.
      runFlush('timer')
    }
  }

  /**
   * Odświeża widok **natychmiast**, bez czekania na klatkę ani na pilnowacza.
   *
   * Wołane z widoku logu przy pierwszym zdarzeniu wejścia po powrocie do okna
   * (`src/log/wake.ts`). Powód: dla okna zminimalizowanego albo zasłoniętego
   * Chromium potrafi wstrzymać produkcję klatek i odłożyć zadania; dane są wtedy
   * już w buforze, ale widok stoi do pierwszej klatki - czyli do momentu, w którym
   * użytkownik sam ruszy oknem. Wtedy domykamy zaległość od razu.
   */
  function flushNow(): void {
    runFlush('now')
  }

  /** Dopisuje linie do bufora konta i zaplanuje odświeżenie widoku. */
  function append(
    accountId: string,
    lines: ReadonlyArray<{ stream: LogStream; raw: string; ts: number }>,
    activeAccountId: string | null,
  ): void {
    if (lines.length === 0) return
    const state = stateFor(accountId)
    state.buffer.pushMany(lines)
    // Diagnostyka: licznik bufora (patrz `src/log/diag.ts`).
    diagCounters.appended += lines.length
    state.revision += 1
    if (accountId !== activeAccountId) {
      state.unread += lines.length
    }
    scheduleFlush()
  }

  /**
   * Linie konta dla widoku - **świeża migawka**, nie tablica z bufora.
   *
   * Dlaczego kopia: bufor dopisuje linie do tej samej tablicy w miejscu, więc
   * `computed`, który zwracałby ją bez kopii, przy każdej dawce zwracałby tę samą
   * tożsamość. Vue porównuje wartości przez `Object.is` (`hasChanged`), a jego
   * wersyjne sprawdzanie zmian (`isDirty` + `refreshComputed` w `@vue/reactivity`)
   * uznawało wtedy, że nic się nie zmieniło - i **pomijało aktualizację widoku**
   * (`instance.job = effect.runIfDirty.bind(effect)` w `@vue/runtime-core`).
   * Objaw: przy zapełnionym buforze log zamarzał i odświeżał się dopiero przy
   * okazji innej zmiany (przewinięcia, pomiaru wysokości wierszy).
   */
  function linesFor(accountId: string): readonly LogLine[] {
    const buffer = states.value.get(accountId)?.buffer
    return buffer ? buffer.all().slice() : []
  }

  function sizeFor(accountId: string): number {
    return states.value.get(accountId)?.buffer.size ?? 0
  }

  function unreadFor(accountId: string): number {
    return states.value.get(accountId)?.unread ?? 0
  }

  function markRead(accountId: string): void {
    const state = states.value.get(accountId)
    if (state) state.unread = 0
  }

  /** Czyści logi konta (np. przy ponownym połączeniu). */
  function clear(accountId: string): void {
    const state = states.value.get(accountId)
    if (!state) return
    state.buffer.clear()
    state.unread = 0
    state.revision += 1
    scheduleFlush()
  }

  function remove(accountId: string): void {
    if (states.value.delete(accountId)) scheduleFlush()
  }

  return {
    revision,
    activeRevision,
    setLimit,
    flushNow,
    append,
    linesFor,
    sizeFor,
    unreadFor,
    markRead,
    clear,
    remove,
  }
})

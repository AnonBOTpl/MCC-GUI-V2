// Reporter diagnostyczny zamrożenia widoku logu - opcjonalny, wyłączony domyślnie.
//
// Po co: nie wiemy, która warstwa się zatrzymuje, gdy log przestaje się
// odświeżać do pierwszego ruchu myszą. Ten moduł zbiera liczby, które to
// rozstrzygają, i wysyła je do Rusta (`diag_report`), gdzie lądują w `diag.log`:
//
// - `frames` / `frameGapMaxMs` / `frameAgeMs` - czy przeglądarka produkuje klatki,
// - `intervalMs` - czy nasz timer raportujący w ogóle odpala (jeśli przerwa jest
//   wielokrotnie dłuższa niż 5 s, strona była wstrzymana w całości),
// - `counters` - czy linie doszły do store'u i czy odświeżenie widoku zostało
//   domknięte (i przez kogo: klatkę, pilnowacza czy zdarzenie wejścia),
// - `view` - stan widoku logu (rozmiar bufora, ile wierszy w DOM, przewijanie).
//
// Nic z tego nie zawiera treści logów ani poświadczeń. Włącza się ją świadomie,
// zmienną `MCC_GUI_DIAG=1` w środowisku aplikacji (patrz `src-tauri/src/diag.rs`).
// Bez niej moduł nie zakłada żadnej pętli ani nie woła Rusta poza jednym pytaniem
// `diag_enabled` na starcie.

import { backend } from '../api/backend'
import type { DiagReport } from '../api/contract'

/** Odstęp między raportami. Świadomie krótki: interesuje nas dokładność co do
 * sekund, a jeden raport na 5 s nie obciąża aplikacji. */
export const REPORT_INTERVAL_MS = 5000

/**
 * Liczniki zbierane w miejscach, które decydują o odświeżeniu widoku.
 * Trzymamy je w zwykłym obiekcie (nie w reaktywnym stanie), bo to dane dla
 * diagnostyki, a nie dla UI - i nie mogą wpływać na wydajność renderowania.
 */
export const diagCounters = {
  /** Zdarzenia `mcc://log` odebrane z Rusta. */
  received: 0,
  /** Linie dopisane do buforów kont. */
  appended: 0,
  /** Domknięcia odświeżenia przez klatkę animacji. */
  flushFrame: 0,
  /** Domknięcia odświeżenia przez pilnowacza (timer). */
  flushTimer: 0,
  /** Domknięcia odświeżenia na żądanie (`flushNow`, zdarzenie wejścia). */
  flushNow: 0,
  /** Zdarzenia wejścia, które wołały `flushNow`. */
  wake: 0,
  /** Przerysowania widoku logu (`onUpdated`). */
  renders: 0,
}

/** Stan widoku logu w chwili raportu (mierzy go `LogView`). */
export interface ViewProbe {
  bufferSize: number
  rowsInDom: number
  scrollHeight: number
  scrollTop: number
  clientHeight: number
  follow: boolean
}

/** Widok logu zgłasza się tutaj, żeby raport zawierał jego stan. */
let viewProbe: (() => ViewProbe) | null = null

/** Podłącza (albo odpina, gdy `null`) źródło stanu widoku. */
export function setViewProbe(probe: (() => ViewProbe) | null): void {
  viewProbe = probe
}

let active = false

/** Czy diagnostyka jest włączona (odpowiedź Rusta, pytamy raz na start). */
export function isDiagActive(): boolean {
  return active
}

/**
 * Pyta Rusta, czy diagnostyka jest włączona, i jeśli tak - startuje pętlę
 * raportującą. Bez diagnostyki nie robi nic poza tym jednym pytaniem.
 */
export async function startDiagnostics(): Promise<void> {
  try {
    active = await backend.diagEnabled()
  } catch {
    // Brak odpowiedzi Rusta to nie powód, żeby cokolwiek pokazywać w UI.
    active = false
  }
  if (active) startLoop()
}

/**
 * Pętla raportująca: jedna klatka animacji na raz (mierzy odstępy między
 * klatkami) plus timer co {@link REPORT_INTERVAL_MS}.
 */
function startLoop(): void {
  const hasFrames = typeof requestAnimationFrame === 'function'
  let lastFrameAt = performance.now()
  let previousFrameAt = lastFrameAt
  let frames = hasFrames ? 0 : -1
  let gapMax = 0

  if (hasFrames) {
    const onFrame = (time: number): void => {
      frames += 1
      const gap = time - previousFrameAt
      if (gap > gapMax) gapMax = gap
      previousFrameAt = time
      lastFrameAt = time
      requestAnimationFrame(onFrame)
    }
    requestAnimationFrame(onFrame)
  }

  let previousReportAt = performance.now()

  window.setInterval(() => {
    const now = performance.now()
    const report: DiagReport = {
      intervalMs: Math.round(now - previousReportAt),
      frames,
      frameGapMaxMs: hasFrames ? Math.round(gapMax) : -1,
      frameAgeMs: hasFrames ? Math.round(now - lastFrameAt) : -1,
      visibility: document.visibilityState,
      focused: document.hasFocus(),
      counters: { ...diagCounters },
      view: viewProbe ? viewProbe() : null,
    }
    previousReportAt = now
    if (hasFrames) {
      frames = 0
      gapMax = 0
    }

    void backend.diagReport(report).catch(() => {
      // Diagnostyka nie może psuć aplikacji: brak zapisu ignorujemy.
    })
  }, REPORT_INTERVAL_MS)
}

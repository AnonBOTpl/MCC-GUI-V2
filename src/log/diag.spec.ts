import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import type { DiagReport } from '../api/contract'
import { diagCounters, REPORT_INTERVAL_MS, setViewProbe, startDiagnostics } from './diag'

// Reporter diagnostyczny ma sens tylko wtedy, gdy (a) przy wyłączonej diagnostyce
// nie robi nic i nie wysyła nic do Rusta, (b) przy włączonej wysyła raporty z
// licznikami i stanem widoku. Te dwie rzeczy sprawdzamy tutaj - reszta to liczby,
// które interpretujemy dopiero z pliku `diag.log`.

const diagEnabled = vi.fn<() => Promise<boolean>>()
const diagReport = vi.fn<(report: DiagReport) => Promise<void>>()

vi.mock('../api/backend', () => ({
  backend: {
    diagEnabled: () => diagEnabled(),
    diagReport: (report: DiagReport) => diagReport(report),
  },
}))

describe('reporter diagnostyczny', () => {
  beforeEach(() => {
    vi.useFakeTimers()
    vi.stubGlobal('requestAnimationFrame', () => 1)
    diagEnabled.mockReset()
    diagReport.mockReset()
    diagReport.mockResolvedValue(undefined)
    // Liczniki są modułowe - zerujemy je, żeby testy się nie sumowały.
    for (const key of Object.keys(diagCounters) as Array<keyof typeof diagCounters>) {
      diagCounters[key] = 0
    }
    setViewProbe(null)
  })

  afterEach(() => {
    vi.useRealTimers()
    vi.unstubAllGlobals()
  })

  it('nie wysyła nic, gdy diagnostyka jest wyłączona', async () => {
    diagEnabled.mockResolvedValue(false)

    await startDiagnostics()
    vi.advanceTimersByTime(REPORT_INTERVAL_MS * 3)

    expect(diagReport).not.toHaveBeenCalled()
  })

  it('wysyła raport z licznikami i stanem widoku, gdy diagnostyka jest włączona', async () => {
    diagEnabled.mockResolvedValue(true)
    setViewProbe(() => ({
      bufferSize: 42,
      rowsInDom: 7,
      scrollHeight: 900,
      scrollTop: 880,
      clientHeight: 200,
      follow: true,
    }))
    diagCounters.received = 3
    diagCounters.flushTimer = 2

    await startDiagnostics()
    vi.advanceTimersByTime(REPORT_INTERVAL_MS)

    expect(diagReport).toHaveBeenCalledTimes(1)
    const report = diagReport.mock.calls[0][0]
    expect(report.counters.received).toBe(3)
    expect(report.counters.flushTimer).toBe(2)
    expect(report.view).toEqual({
      bufferSize: 42,
      rowsInDom: 7,
      scrollHeight: 900,
      scrollTop: 880,
      clientHeight: 200,
      follow: true,
    })
    expect(report.visibility).toBe(document.visibilityState)
    expect(report.intervalMs).toBeGreaterThanOrEqual(0)
  })

  it('raportuje brak stanu widoku, gdy żaden widok nie jest zamontowany', async () => {
    diagEnabled.mockResolvedValue(true)

    await startDiagnostics()
    vi.advanceTimersByTime(REPORT_INTERVAL_MS)

    expect(diagReport.mock.calls[0][0].view).toBeNull()
  })

  it('nigdy nie odrzuca, gdy zapis raportu zawiedzie', async () => {
    diagEnabled.mockResolvedValue(true)
    diagReport.mockRejectedValue(new Error('brak zapisu'))

    await startDiagnostics()

    expect(() => vi.advanceTimersByTime(REPORT_INTERVAL_MS)).not.toThrow()
  })
})

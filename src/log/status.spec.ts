import { describe, expect, it } from 'vitest'
import type { AccountStatus } from '../api/contract'
import { isConnectionLive, statusDotClass, statusKey } from './status'

const ALL: AccountStatus[] = [
  'stopped',
  'starting',
  'awaiting_login',
  'running',
  'stopping',
  'crashed',
]

describe('statusy połączenia', () => {
  it('każdy status ma klucz tłumaczenia i kolor kropki', () => {
    for (const status of ALL) {
      expect(statusKey(status)).toMatch(/^status\./)
      expect(statusDotClass(status)).toMatch(/^bg-/)
    }
  })

  it('żywe połączenie zatrzymujemy, a martwe uruchamiamy', () => {
    // Zielona strzałka (start) tylko dla stanów, w których nic nie działa.
    expect(isConnectionLive('stopped')).toBe(false)
    expect(isConnectionLive('crashed')).toBe(false)

    // Czerwony kwadrat (stop) także wtedy, gdy proces jest dopiero w drodze.
    expect(isConnectionLive('starting')).toBe(true)
    expect(isConnectionLive('awaiting_login')).toBe(true)
    expect(isConnectionLive('running')).toBe(true)
    expect(isConnectionLive('stopping')).toBe(true)
  })
})

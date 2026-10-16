// Mapowanie statusu konta na klucz i18n oraz kolor kropki statusu.
//
// Statusy z Rusta są `snake_case` (kontrakt), a klucze i18n camelCase,
// więc tłumaczymy je w jednym miejscu, żeby nie rozjechały się w komponentach.

import type { AccountStatus } from '../api/contract'

const STATUS_KEYS: Record<AccountStatus, string> = {
  stopped: 'status.stopped',
  starting: 'status.starting',
  awaiting_login: 'status.awaitingLogin',
  running: 'status.running',
  stopping: 'status.stopping',
  crashed: 'status.crashed',
}

/** Klasy Tailwind dla kropki statusu (zielona = działa, czerwona = awaria itd.). */
const STATUS_DOT_CLASSES: Record<AccountStatus, string> = {
  stopped: 'bg-slate-500',
  starting: 'bg-amber-400',
  awaiting_login: 'bg-sky-400',
  running: 'bg-emerald-400',
  stopping: 'bg-amber-400',
  crashed: 'bg-red-500',
}

export function statusKey(status: AccountStatus): string {
  return STATUS_KEYS[status]
}

export function statusDotClass(status: AccountStatus): string {
  return STATUS_DOT_CLASSES[status]
}

/**
 * Czy połączenie ma żywy proces (albo jest w drodze do takiego stanu).
 *
 * Decyduje o przycisku przy serwerze w panelu: żywe połączenie zatrzymujemy
 * (czerwony kwadrat), a zatrzymane uruchamiamy (zielona strzałka).
 */
export function isConnectionLive(status: AccountStatus): boolean {
  return (
    status === 'starting' ||
    status === 'awaiting_login' ||
    status === 'running' ||
    status === 'stopping'
  )
}

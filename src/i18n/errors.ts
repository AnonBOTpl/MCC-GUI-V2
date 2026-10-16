// Tłumaczenie stabilnych kodów błędów z Rusta.
//
// Rust nigdy nie wysyła tekstu dla człowieka - tylko kod
// i parametry. Tutaj zamieniamy je na klucz i18n, a nieznany kod sprowadzamy do
// komunikatu ogólnego, żeby UI nie pokazało surowego kodu.

import { i18n } from './index'
import type { ErrorPayload } from '../api/contract'

/** Kody błędów, dla których mamy osobne tłumaczenie. */
const KNOWN_ERROR_CODES = new Set([
  'mcc.executableNotSet',
  'mcc.pathEmpty',
  'mcc.pathNotFound',
  'mcc.pathNotFile',
  'mcc.pathNotExe',
  'settings.languageInvalid',
  'settings.delayOutOfRange',
  'settings.bufferOutOfRange',
  'settings.retentionOutOfRange',
  'store.writeFailed',
  'dialog.failed',
  'server.invalid',
  'server.invalidVersion',
  'connection.notFound',
  'connection.notRunning',
  'connection.alreadyRunning',
  'account.alreadyExists',
  'account.invalidId',
  'account.duplicateId',
  'account.loginRequired',
  'account.tooManyServers',
  'account.notFound',
  'account.dirCreateFailed',
  'account.settingsWriteFailed',
  'process.spawnFailed',
  'process.jobCreateFailed',
  'process.jobAssignFailed',
  'process.handleUnavailable',
  'process.stdinClosed',
  'process.writeFailed',
  'process.exitedUnexpectedly',
  'input.empty',
  'input.tooLong',
  'input.controlChars',
  'presets.tooMany',
  'presets.invalidId',
  'presets.duplicateId',
  'presets.labelRequired',
  'presets.labelTooLong',
  'presets.labelControlChars',
  'presets.textRequired',
  'presets.invalidColor',
  'presets.readFailed',
  'url.notAllowed',
  'url.openFailed',
  'clipboard.writeFailed',
  'clipboard.tooLong',
  'chatlog.dirEmpty',
  'chatlog.dirNotAbsolute',
  'chatlog.dirNotWritable',
  'chatlog.openFailed',
  'chatlog.writeFailed',
  'app.unexpectedError',
  'app.closeFailed',
])

/**
 * Zamienia kod i parametry błędu na gotowy tekst w bieżącym języku.
 * `overrides` pozwala podmienić parametry, np. `accountId` na etykietę konta.
 */
export function translateError(
  payload: ErrorPayload,
  overrides: Record<string, unknown> = {},
): string {
  const code = KNOWN_ERROR_CODES.has(payload.code) ? payload.code : 'app.unexpectedError'
  return i18n.global.t(`errors.${code}`, { ...payload.params, ...overrides })
}
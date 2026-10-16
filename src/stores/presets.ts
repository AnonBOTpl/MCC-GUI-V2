// Store presetów szybkich akcji (Etap 6).
//
// Presety trzyma `presets.json` w katalogu danych aplikacji, a zapis i walidacja
// należą do Rusta (`preset::validate_presets`). Frontend:
// - wczytuje listę raz na starcie (`load`) i czyści ją `sanitizePresets`, żeby
//   ręcznie edytowany plik nie wywalił widoku,
// - zapisuje całą listę (`save`), a stan podstawia wersją znormalizowaną z Rusta,
// - umie przeładować plik z dysku (`reload`, przycisk `Reload`),
// - umie **uruchomić** preset (`run`).
//
// Wysyłka idzie przez store kont (`accounts.send`) - dokładnie tą samą drogą co
// wiadomość z pola wpisywania. Zasięg bierze się więc z checkboxa „Wyślij do
// wszystkich” przy polu, a nie z presetu: akcja ma robić to samo, co wpisana
// ręcznie komenda. Gdy nie było do kogo wysłać, mówimy o tym przy pasku, zamiast
// udawać, że akcja zadziałała.
//
// Akcje wyłączone w ogóle nie trafiają na pasek (`enabledPresets`), więc `run`
// sprawdza `enabled` jeszcze raz jako zabezpieczenie przed wywołaniem z kodu.

import { defineStore } from 'pinia'
import { ref } from 'vue'
import { backend } from '../api/backend'
import { sanitizePresets } from '../presets/model'
import type { ErrorPayload, Preset } from '../api/contract'
import { useAccountsStore } from './accounts'

/** Komunikat przy pasku akcji (po `Reload` albo gdy nie było gdzie wysłać). */
export type PresetNotice =
  | { kind: 'reloaded'; count: number }
  | { kind: 'noneRunning' }
  | { kind: 'noActive' }

/** Ile trzymamy komunikat przy pasku, zanim zniknie. */
const NOTICE_TIMEOUT_MS = 3000

export const usePresetsStore = defineStore('presets', () => {
  /** Lista presetów w kolejności z paska. */
  const presets = ref<Preset[]>([])
  /** Czy wczytaliśmy plik (potrzebne, żeby nie pokazywać „brak akcji” przed czasem). */
  const loaded = ref(false)
  const notice = ref<PresetNotice | null>(null)

  let noticeTimer: ReturnType<typeof setTimeout> | null = null

  /** Pokazuje krótki komunikat przy pasku i zleca jego schowanie. */
  function showNotice(next: PresetNotice): void {
    notice.value = next
    if (noticeTimer !== null) clearTimeout(noticeTimer)
    noticeTimer = setTimeout(() => {
      notice.value = null
      noticeTimer = null
    }, NOTICE_TIMEOUT_MS)
  }

  /** Błędy pokazujemy wspólnym mechanizmem toastów ze store kont. */
  function pushError(payload: ErrorPayload): void {
    useAccountsStore().pushError(payload)
  }

  /** Wczytuje `presets.json`. Wywoływane raz przy starcie aplikacji. */
  async function load(): Promise<void> {
    try {
      presets.value = sanitizePresets(await backend.listPresets())
    } catch (error) {
      pushError(error as ErrorPayload)
    } finally {
      loaded.value = true
    }
  }

  /**
   * Zapisuje całą listę. Rust waliduje wpisy i zwraca wersję znormalizowaną,
   * więc edytor zamykamy dopiero po jej przyjęciu (`true`).
   */
  async function save(list: Preset[]): Promise<boolean> {
    try {
      presets.value = await backend.savePresets(list)
      return true
    } catch (error) {
      pushError(error as ErrorPayload)
      return false
    }
  }

  /**
   * Wczytuje plik ponownie (przycisk `Reload`). Ścieżka jest ścisła: zepsuty
   * plik zgłasza błąd, ale nie psuje tego, co już mamy na pasku.
   */
  async function reload(): Promise<boolean> {
    try {
      const fromDisk = sanitizePresets(await backend.reloadPresets())
      presets.value = fromDisk
      showNotice({ kind: 'reloaded', count: fromDisk.length })
      return true
    } catch (error) {
      pushError(error as ErrorPayload)
      return false
    }
  }

  /**
   * Uruchamia preset. Zasięg jest taki sam jak przy wpisie z klawiatury:
   * aktywna zakładka albo wszystkie działające, gdy zaznaczony jest checkbox
   * „Wyślij do wszystkich”.
   */
  async function run(preset: Preset): Promise<void> {
    if (!preset.enabled) return

    const accounts = useAccountsStore()
    const toAll = accounts.sendToAll
    const delivered = await accounts.send(preset.text)
    if (delivered > 0) return

    showNotice(toAll ? { kind: 'noneRunning' } : { kind: 'noActive' })
  }

  return {
    presets,
    loaded,
    notice,
    load,
    save,
    reload,
    run,
    showNotice,
  }
})

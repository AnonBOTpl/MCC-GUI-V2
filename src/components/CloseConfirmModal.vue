<script setup lang="ts">
// Pytanie o zamknięcie aplikacji, gdy działają połączenia (Paczka 2.2).
//
// Okno jest już **wstrzymane** przez Rusta (`prevent_close`), więc ten modal sam
// nic nie zamyka: „Zamknij i rozłącz” woła komendę `close_app` (Rust najpierw
// łagodnie zatrzymuje połączenia, potem zamyka okno), a „Zostaw uruchomione”
// - tak samo jak Esc i klik poza modalem - nie robi nic i okno zostaje otwarte.

import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useSettingsStore } from '../stores/settings'
import BaseModal from './BaseModal.vue'

const props = defineProps<{
  open: boolean
  /** Ile połączeń zostałoby rozłączonych - liczba z Rusta, nie z UI. */
  active: number
}>()
const emit = defineEmits<{ (event: 'close'): void }>()

const { t } = useI18n()
const settings = useSettingsStore()

const dontAsk = ref(false)
/** Blokuje podwójne kliknięcie „Zamknij i rozłącz” w trakcie zamykania. */
const closing = ref(false)

/** Każde otwarcie zaczyna od czystego stanu. */
watch(
  () => props.open,
  (open) => {
    if (open) {
      dontAsk.value = false
      closing.value = false
    }
  },
)

/**
 * Potwierdzenie zamknięcia.
 *
 * „Nie pytaj więcej” zapisujemy **przed** zamknięciem, żeby wybór przetrwał
 * restart aplikacji. Gdy zapis się nie uda, i tak zamykamy - błąd ląduje
 * w toaście, a przy następnym uruchomieniu pytanie po prostu wróci.
 */
async function confirm(): Promise<void> {
  if (closing.value) return
  closing.value = true

  if (dontAsk.value) await settings.save({ confirmClose: false })

  const closed = await settings.closeApp()
  // Nie udało się zamknąć (np. okna już nie ma) - zostawiamy modal otwarty,
  // żeby użytkownik mógł spróbować ponownie albo rozłączyć połączenia sam.
  if (!closed) closing.value = false
}
</script>

<template>
  <BaseModal
    :open="open"
    :label="t('closeConfirm.title')"
    :close-label="t('closeConfirm.cancel')"
    :body-class="'gap-4 py-4'"
    :dismissible="!closing"
    @close="emit('close')"
  >
    <p class="text-xs text-slate-300">
      {{ t('closeConfirm.message', { count: active }) }}
    </p>

    <label class="flex items-center gap-2 text-xs text-slate-300">
      <input
        v-model="dontAsk"
        type="checkbox"
        class="h-3.5 w-3.5 shrink-0 accent-blue-600"
      >
      {{ t('closeConfirm.dontAsk') }}
    </label>

    <template #footer>
      <button
        type="button"
        class="ml-auto h-8 rounded border border-slate-700 px-3 text-xs text-slate-200 transition-colors hover:bg-slate-800"
        @click="emit('close')"
      >
        {{ t('closeConfirm.cancel') }}
      </button>
      <button
        type="button"
        class="h-8 rounded bg-blue-600 px-4 text-xs font-medium text-white transition-colors hover:bg-blue-700 active:bg-blue-800 disabled:cursor-not-allowed disabled:opacity-60"
        :disabled="closing"
        @click="confirm"
      >
        {{ t('closeConfirm.confirm') }}
      </button>
    </template>
  </BaseModal>
</template>

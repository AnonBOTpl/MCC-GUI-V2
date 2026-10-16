<script setup lang="ts">
// Pytanie o zamknięcie aplikacji, gdy działają połączenia (Paczka 2.2).
//
// Okno jest już **wstrzymane** przez Rusta (`prevent_close`), więc ten modal sam
// nic nie zamyka: „Zamknij i rozłącz” woła komendę `close_app` (Rust najpierw
// łagodnie zatrzymuje połączenia, potem zamyka okno), a „Zostaw uruchomione”
// - tak samo jak Esc i klik poza modalem - nie robi nic i okno zostaje otwarte.

import { onUnmounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { X } from '@lucide/vue'
import { useSettingsStore } from '../stores/settings'

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

/** Esc działa jak „Zostaw uruchomione”. */
function onKeydown(event: KeyboardEvent): void {
  if (event.key === 'Escape' && props.open && !closing.value) emit('close')
}
window.addEventListener('keydown', onKeydown)
onUnmounted(() => window.removeEventListener('keydown', onKeydown))

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
  <Transition name="modal">
    <div
      v-if="open"
      class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4"
      @click.self="emit('close')"
    >
      <div class="flex w-full max-w-md flex-col rounded-lg border border-slate-700 bg-slate-900 shadow-2xl">
        <header class="flex shrink-0 items-center justify-between border-b border-slate-800 px-4 py-3">
          <h2 class="text-sm font-semibold text-slate-100">
            {{ t('closeConfirm.title') }}
          </h2>
          <button
            type="button"
            class="flex h-7 w-7 items-center justify-center rounded text-slate-400 transition-colors hover:bg-slate-800 hover:text-slate-100"
            :title="t('closeConfirm.cancel')"
            @click="emit('close')"
          >
            <X :size="16" />
          </button>
        </header>

        <div class="flex flex-col gap-4 px-4 py-4">
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
        </div>

        <footer class="flex shrink-0 items-center justify-end gap-2 border-t border-slate-800 px-4 py-3">
          <button
            type="button"
            class="h-8 rounded border border-slate-700 px-3 text-xs text-slate-200 transition-colors hover:bg-slate-800"
            @click="emit('close')"
          >
            {{ t('closeConfirm.cancel') }}
          </button>
          <button
            type="button"
            class="h-8 rounded bg-blue-600 px-4 text-xs font-medium text-white transition-colors hover:bg-blue-500 disabled:cursor-not-allowed disabled:opacity-60"
            :disabled="closing"
            @click="confirm"
          >
            {{ t('closeConfirm.confirm') }}
          </button>
        </footer>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.modal-enter-active,
.modal-leave-active {
  transition: opacity 0.15s ease;
}
.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}
</style>

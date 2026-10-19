<script setup lang="ts">
// Proste okno potwierdzenia akcji (R-19, R-44).
//
// Cienka warstwa na `BaseModal`: nagłówek, jedno zdanie treści i dwa przyciski -
// akcja bezpieczna (zamyka okno) oraz akcja, którą użytkownik potwierdza.
// Treść i etykiety przekazuje rodzic, więc teksty idą przez i18n w miejscu,
// gdzie znana jest liczba i nazwa (konto, serwer).

import { useI18n } from 'vue-i18n'
import BaseModal from './BaseModal.vue'

const props = defineProps<{
  open: boolean
  title: string
  message: string
  confirmLabel: string
  /** Etykieta przycisku bezpiecznego i krzyżyka; domyślnie „Anuluj”. */
  cancelLabel?: string
  /** Akcja nieodwracalna (np. usunięcie konta) - przycisk w kolorze ostrzeżenia. */
  danger?: boolean
}>()
const emit = defineEmits<{ (event: 'confirm'): void; (event: 'close'): void }>()

const { t } = useI18n()

function cancelText(): string {
  return props.cancelLabel ?? t('confirm.cancel')
}
</script>

<template>
  <BaseModal
    :open="open"
    :label="title"
    :close-label="cancelText()"
    :body-class="'gap-4 py-4'"
    @close="emit('close')"
  >
    <p class="text-xs leading-5 text-slate-300">
      {{ message }}
    </p>

    <template #footer>
      <button
        type="button"
        class="ml-auto h-8 rounded border border-slate-700 px-3 text-xs text-slate-200 transition-colors hover:bg-slate-800"
        @click="emit('close')"
      >
        {{ cancelText() }}
      </button>
      <button
        type="button"
        class="h-8 rounded px-4 text-xs font-medium text-white transition-colors"
        :class="danger ? 'bg-red-600 hover:bg-red-700 active:bg-red-800' : 'bg-blue-600 hover:bg-blue-700 active:bg-blue-800'"
        @click="emit('confirm')"
      >
        {{ confirmLabel }}
      </button>
    </template>
  </BaseModal>
</template>

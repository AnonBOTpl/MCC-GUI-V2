<script setup lang="ts">
import { onUnmounted, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { TriangleAlert, X } from '@lucide/vue'
import { translateError } from '../i18n/errors'
import { useAccountsStore } from '../stores/accounts'

const { t } = useI18n()
const accounts = useAccountsStore()

/** Automatyczne ukrywanie komunikatów po 8 sekundach. */
const HIDE_AFTER_MS = 8000
const timers = new Map<number, ReturnType<typeof setTimeout>>()

watch(
  () => accounts.errors.map((error) => error.id),
  (ids) => {
    for (const error of accounts.errors) {
      if (timers.has(error.id)) continue
      timers.set(
        error.id,
        setTimeout(() => {
          timers.delete(error.id)
          accounts.dismissError(error.id)
        }, HIDE_AFTER_MS),
      )
    }
    for (const [id, timer] of timers) {
      if (!ids.includes(id)) {
        clearTimeout(timer)
        timers.delete(id)
      }
    }
  },
  { immediate: true },
)

onUnmounted(() => {
  for (const timer of timers.values()) clearTimeout(timer)
  timers.clear()
})

/** Podmienia `accountId` na etykietę konta, gdy konto jeszcze istnieje. */
function messageFor(error: (typeof accounts.errors)[number]): string {
  const account = accounts.accounts.find((item) => item.id === error.params.accountId)
  return translateError(error, account ? { accountId: account.label } : {})
}
</script>

<template>
  <!-- `z-[60]` > `z-50` modali: błąd zapisu w oknie dialogowym musi być widać,
       bo to jedyna informacja, dlaczego okno zostało otwarte (R-20). -->
  <div class="pointer-events-none fixed bottom-4 left-4 z-[60] flex w-96 flex-col gap-2">
    <TransitionGroup name="toast">
      <div
        v-for="error in accounts.errors"
        :key="error.id"
        class="pointer-events-auto flex items-start gap-2 rounded border border-red-800 bg-red-950/95 px-3 py-2 shadow-lg"
        role="alert"
      >
        <TriangleAlert
          :size="16"
          class="mt-0.5 shrink-0 text-red-400"
        />
        <div class="min-w-0 flex-1">
          <p class="text-xs font-semibold text-red-200">
            {{ t('errors.title') }}
          </p>
          <p class="text-xs text-red-100">
            {{ messageFor(error) }}
          </p>
        </div>
        <button
          type="button"
          class="flex h-6 w-6 shrink-0 items-center justify-center rounded text-red-300 transition-colors hover:bg-red-900 hover:text-white"
          :title="t('errors.dismiss')"
          @click="accounts.dismissError(error.id)"
        >
          <X :size="14" />
        </button>
      </div>
    </TransitionGroup>
  </div>
</template>

<style scoped>
.toast-enter-active,
.toast-leave-active {
  transition:
    opacity 0.15s ease,
    transform 0.15s ease;
}
.toast-enter-from,
.toast-leave-to {
  opacity: 0;
  transform: translateY(6px);
}
</style>

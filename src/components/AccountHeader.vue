<script setup lang="ts">
// Nagłówek aktywnego **połączenia**: serwer, konto i przyciski sterujące.
// Dla stanów `stopped`/`crashed` pokazujemy „Uruchom", w pozostałych
// „Rozłącz" (aktywne tylko dla działającego procesu) i „Połącz ponownie".

import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { Play, PlugZap, Unplug } from '@lucide/vue'
import type { Connection } from '../api/contract'
import { statusDotClass, statusKey } from '../log/status'
import { useAccountsStore } from '../stores/accounts'

const props = defineProps<{ connection: Connection; accountLabel: string; follow: boolean }>()
const emit = defineEmits<{ (event: 'update:follow', value: boolean): void }>()

const { t } = useI18n()
const accounts = useAccountsStore()

const status = computed(() => accounts.statusOf(props.connection.id))
/** Zatrzymane i po awarii uruchamiamy przyciskiem `Start`. */
const canStart = computed(() => status.value === 'stopped' || status.value === 'crashed')
/** Rozłączyć można tylko proces, który faktycznie działa. */
const canDisconnect = computed(
  () => status.value === 'running' || status.value === 'awaiting_login',
)
</script>

<template>
  <header class="flex shrink-0 items-center gap-3 border-b border-slate-800 bg-slate-900/40 px-3 py-2">
    <span
      class="h-2.5 w-2.5 rounded-full"
      :class="statusDotClass(status)"
    />
    <span class="text-sm font-semibold text-slate-100">{{ props.connection.server }}</span>
    <span class="text-xs text-slate-400">{{ t(statusKey(status)) }}</span>
    <span class="text-xs text-slate-500">{{ props.accountLabel }}</span>

    <div class="ml-auto flex items-center gap-2">
      <label class="flex cursor-pointer items-center gap-1.5 text-xs text-slate-300">
        <input
          type="checkbox"
          class="h-3.5 w-3.5 accent-blue-500"
          :checked="props.follow"
          @change="emit('update:follow', ($event.target as HTMLInputElement).checked)"
        >
        {{ t('account.autoScroll') }}
      </label>

      <button
        v-if="canStart"
        type="button"
        class="flex h-7 items-center gap-1.5 rounded bg-blue-600 px-2 text-xs font-medium text-white transition-colors hover:bg-blue-500"
        @click="accounts.startConnection(props.connection.id)"
      >
        <Play :size="13" />
        {{ t('account.start') }}
      </button>

      <template v-else>
        <button
          type="button"
          class="flex h-7 items-center gap-1.5 rounded border border-slate-700 px-2 text-xs text-slate-200 transition-colors hover:bg-slate-800 disabled:cursor-not-allowed disabled:opacity-40"
          :disabled="!canDisconnect"
          @click="accounts.disconnectConnection(props.connection.id)"
        >
          <Unplug :size="13" />
          {{ t('account.disconnect') }}
        </button>

        <button
          type="button"
          class="flex h-7 items-center gap-1.5 rounded border border-slate-700 px-2 text-xs text-slate-200 transition-colors hover:bg-slate-800"
          @click="accounts.restartConnection(props.connection.id)"
        >
          <PlugZap :size="13" />
          {{ t('account.reconnect') }}
        </button>
      </template>
    </div>
  </header>
</template>

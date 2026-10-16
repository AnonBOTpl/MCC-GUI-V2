<script setup lang="ts">
// Zakładki: jedna na **połączenie** (konto + serwer), bo to jedno połączenie
// odpowiada jednemu procesowi MCC.

import { useI18n } from 'vue-i18n'
import { X } from '@lucide/vue'
import { statusDotClass, statusKey } from '../log/status'
import { useAccountsStore } from '../stores/accounts'
import { useLogsStore } from '../stores/logs'

const { t } = useI18n()
const accounts = useAccountsStore()
const logs = useLogsStore()

function unreadFor(id: string): number {
  // `logs.activeRevision` w zależności wymusza przeliczenie po partii linii.
  void logs.activeRevision
  return logs.unreadFor(id)
}

function accountLabel(accountId: string): string {
  return accounts.accounts.find((account) => account.id === accountId)?.label ?? accountId
}
</script>

<template>
  <div
    v-if="accounts.connections.length > 0"
    class="flex shrink-0 items-stretch overflow-x-auto border-b border-slate-800 bg-slate-950"
    role="tablist"
  >
    <!-- Zakładka i przycisk zamknięcia to osobne elementy: zagnieżdżony
         `<button>` w `<button>` jest niepoprawnym HTML-em i psuje dostępność. -->
    <div
      v-for="conn in accounts.connections"
      :key="conn.id"
      class="flex items-stretch border-b-2 transition-colors"
      :class="
        conn.id === accounts.activeConnectionId
          ? 'border-blue-500 bg-slate-900'
          : 'border-transparent bg-slate-950 hover:bg-slate-900/60'
      "
    >
      <button
        type="button"
        role="tab"
        :aria-selected="conn.id === accounts.activeConnectionId"
        class="flex min-w-44 max-w-60 items-center gap-2 py-2 pl-3 pr-1 text-left"
        :title="`${accountLabel(conn.accountId)} — ${conn.server}`"
        @click="accounts.setActiveConnection(conn.id)"
      >
        <span
          class="h-2 w-2 shrink-0 rounded-full"
          :class="statusDotClass(accounts.statusOf(conn.id))"
          :title="t(statusKey(accounts.statusOf(conn.id)))"
        />
        <span class="min-w-0 flex-1">
          <span class="block truncate text-xs font-medium text-slate-200">{{ conn.server }}</span>
          <span class="block truncate text-[11px] text-slate-400">
            {{ accountLabel(conn.accountId) }} · {{ t(statusKey(accounts.statusOf(conn.id))) }}
          </span>
        </span>
        <span
          v-if="unreadFor(conn.id) > 0"
          class="shrink-0 rounded-full bg-blue-600 px-1.5 py-0.5 text-[10px] font-semibold text-white"
          :title="t('tabs.unread', unreadFor(conn.id))"
        >
          {{ unreadFor(conn.id) }}
        </span>
      </button>

      <button
        type="button"
        class="my-1 mr-1 flex h-6 w-6 shrink-0 items-center justify-center self-center rounded text-slate-400 transition-colors hover:bg-slate-700 hover:text-white"
        :title="t('tabs.close')"
        @click.stop="accounts.closeConnection(conn.id)"
      >
        <X :size="12" />
      </button>
    </div>
  </div>
</template>

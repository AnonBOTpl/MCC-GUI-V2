<script setup lang="ts">
// Panel kont i serwerów (model „konto + lista serwerów").
//
// Konto to tożsamość logowania; pod nim widać jego zapisane serwery. Klik w
// serwer otwiera zakładkę połączenia i startuje proces. Nowy serwer dodaje się
// inline, z podpowiedziami z historii adresów.
//
// Przy każdym serwerze można też ustawić **wymuszoną wersję Minecrafta**: gdy
// serwer nie odpowiada na ping, MCC pyta w konsoli „Server version:", a wersja
// z tego pola sprawia, że MCC nie pyta wcale.

import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { Play, Plus, Square, Tag, Trash2, X } from '@lucide/vue'
import { connectionId, type ServerEntry } from '../api/contract'
import { isConnectionLive, statusDotClass, statusKey } from '../log/status'
import { useAccountsStore } from '../stores/accounts'
import { useSettingsStore } from '../stores/settings'
import { isSingleClick } from '../ui/click'

const emit = defineEmits<{ (event: 'addAccount'): void }>()

const { t } = useI18n()
const accounts = useAccountsStore()
const settings = useSettingsStore()

/** Konto, dla którego pokazujemy pole dodawania serwera. */
const addingFor = ref<string | null>(null)
const draftServer = ref('')
const draftVersion = ref('')
const busy = ref(false)

function toggleAdd(accountId: string): void {
  addingFor.value = addingFor.value === accountId ? null : accountId
  draftServer.value = ''
  draftVersion.value = ''
}

/** Otwiera formularz wypełniony istniejącym serwerem, żeby zmienić wersję. */
function editVersion(accountId: string, entry: ServerEntry): void {
  addingFor.value = accountId
  draftServer.value = entry.address
  draftVersion.value = entry.version ?? ''
}

// Przycisk przy serwerze zmienia się po pierwszym kliknięciu (Start → Rozłącz),
// więc drugi klik dwukliku musi zostać zignorowany - inaczej użytkownik, który
// chciał tylko uruchomić połączenie, od razu je zamyka.
function startFromSidebar(event: MouseEvent, accountId: string, server: string): void {
  if (!isSingleClick(event)) return
  void accounts.openConnection(accountId, server)
}

function stopFromSidebar(event: MouseEvent, id: string): void {
  if (!isSingleClick(event)) return
  void accounts.disconnectConnection(id)
}

/** Dodaje lub aktualizuje serwer; `connect` decyduje, czy startujemy proces. */
async function submit(accountId: string, connect: boolean): Promise<void> {
  const server = draftServer.value.trim()
  if (server === '' || busy.value) return
  busy.value = true
  const added = await accounts.addServer(accountId, server, draftVersion.value, connect)
  busy.value = false
  if (added) {
    // Adres trafia do historii podpowiedzi (jak w polu serwera w starym GUI).
    await settings.rememberServer(server)
    draftServer.value = ''
    draftVersion.value = ''
    addingFor.value = null
  }
}
</script>

<template>
  <aside class="flex w-72 shrink-0 flex-col overflow-y-auto border-r border-slate-800 bg-slate-950/70">
    <!-- Dodawanie konta jest w nagłówku panelu, tam gdzie konto potem widać
         (modal otwiera `App.vue`). Licznik pokazuje aktywne połączenia. -->
    <div class="flex items-center justify-between gap-2 px-3 pt-3 pb-2">
      <span class="flex items-baseline gap-2">
        <span class="text-[11px] uppercase tracking-wide text-slate-500">
          {{ t('sidebar.accounts') }}
        </span>
        <span class="text-[11px] text-slate-600">
          {{ t('sidebar.activeAccounts', accounts.activeCount) }}
        </span>
      </span>
      <button
        type="button"
        class="flex h-6 shrink-0 items-center gap-1 rounded bg-blue-600 px-2 text-[11px] font-medium text-white transition-colors hover:bg-blue-500"
        @click="emit('addAccount')"
      >
        <Plus :size="11" />
        {{ t('sidebar.addAccount') }}
      </button>
    </div>

    <div
      v-for="account in accounts.accounts"
      :key="account.id"
      class="mx-1.5 mb-1.5 rounded-lg px-1.5 py-1"
    >
      <!-- Nagłówek konta -->
      <div class="flex items-start gap-2 px-1 py-0.5">
        <span
          class="mt-1.5 h-2 w-2 shrink-0 rounded-full"
          :class="statusDotClass(accounts.accountStatus(account.id))"
          :title="t(statusKey(accounts.accountStatus(account.id)))"
        />
        <div class="min-w-0 flex-1">
          <p class="truncate text-xs font-semibold text-slate-100">
            {{ account.label }}
          </p>
          <p class="truncate text-[11px] text-slate-500">
            {{ account.type === 'microsoft' ? t('accountType.microsoft') : t('accountType.offline') }}
            · {{ account.login }}
          </p>
        </div>
        <button
          type="button"
          class="mt-0.5 flex h-5 w-5 shrink-0 items-center justify-center rounded text-slate-500 transition-colors hover:bg-slate-800 hover:text-red-400"
          :title="t('sidebar.removeAccount')"
          @click="accounts.removeAccount(account.id)"
        >
          <Trash2 :size="12" />
        </button>
      </div>

      <!-- Serwery konta -->
      <div
        v-for="entry in account.servers"
        :key="entry.address"
        class="group flex items-center gap-2 rounded px-1.5 py-1 pl-5 transition-colors"
        :class="
          accounts.activeConnection?.id === connectionId(account.id, entry.address)
            ? 'bg-blue-600/20'
            : 'hover:bg-slate-800/60'
        "
      >
        <span
          class="h-2 w-2 shrink-0 rounded-full"
          :class="statusDotClass(accounts.statusOf(connectionId(account.id, entry.address)))"
        />
        <button
          type="button"
          class="min-w-0 flex-1 truncate text-left text-xs text-slate-300 transition-colors hover:text-slate-100"
          :title="t('sidebar.editServer')"
          @click="editVersion(account.id, entry)"
        >
          {{ entry.address }}
        </button>

        <button
          type="button"
          class="flex h-5 shrink-0 items-center gap-1 rounded px-1 text-[10px] transition-colors hover:bg-slate-800"
          :class="entry.version ? 'text-blue-300' : 'text-slate-600'"
          :title="t('sidebar.setVersion')"
          @click="editVersion(account.id, entry)"
        >
          <Tag :size="10" />
          {{ entry.version ?? t('sidebar.versionAuto') }}
        </button>

        <!-- Zielona strzałka = start; po połączeniu ten sam przycisk zmienia się
             w czerwony kwadrat, który zatrzymuje proces (zakładka zostaje). -->
        <button
          v-if="!isConnectionLive(accounts.statusOf(connectionId(account.id, entry.address)))"
          type="button"
          class="flex h-6 w-6 shrink-0 items-center justify-center rounded text-green-400 transition-colors hover:bg-green-600/20 hover:text-green-300"
          :title="t('account.start')"
          @click="startFromSidebar($event, account.id, entry.address)"
        >
          <Play :size="13" />
        </button>

        <button
          v-else
          type="button"
          class="flex h-6 w-6 shrink-0 items-center justify-center rounded text-red-400 transition-colors hover:bg-red-600/20 hover:text-red-300 disabled:cursor-not-allowed disabled:opacity-50"
          :title="t('account.disconnect')"
          :disabled="accounts.statusOf(connectionId(account.id, entry.address)) === 'stopping'"
          @click="stopFromSidebar($event, connectionId(account.id, entry.address))"
        >
          <Square
            :size="11"
            fill="currentColor"
          />
        </button>

        <button
          type="button"
          class="flex h-5 w-5 shrink-0 items-center justify-center rounded text-slate-600 opacity-0 transition-colors group-hover:opacity-100 hover:bg-slate-800 hover:text-red-400"
          :title="t('sidebar.removeServer')"
          @click="accounts.removeServer(account.id, entry.address)"
        >
          <X :size="11" />
        </button>
      </div>

      <!-- Dodawanie / edycja serwera -->
      <div
        v-if="addingFor !== account.id"
        class="pl-5"
      >
        <button
          type="button"
          class="flex items-center gap-1 rounded px-1.5 py-1 pl-5 text-[11px] text-slate-500 transition-colors hover:text-blue-400"
          @click="toggleAdd(account.id)"
        >
          <Plus :size="11" />
          {{ t('sidebar.addServer') }}
        </button>
      </div>

      <div
        v-else
        class="flex flex-col gap-1 px-1.5 pt-1 pl-5"
      >
        <div class="flex items-center justify-between px-0.5">
          <span class="text-[10px] uppercase tracking-wide text-slate-500">
            {{ t('sidebar.addServer') }}
          </span>
          <button
            type="button"
            class="flex h-5 w-5 items-center justify-center rounded text-slate-500 transition-colors hover:bg-slate-800 hover:text-slate-200"
            :title="t('sidebar.cancel')"
            @click="toggleAdd(account.id)"
          >
            <X :size="11" />
          </button>
        </div>

        <input
          v-model="draftServer"
          type="text"
          class="h-7 w-full rounded border border-slate-700 bg-slate-900 px-2 text-xs text-slate-100 outline-none placeholder:text-slate-500 focus:border-blue-500"
          :placeholder="t('sidebar.serverPlaceholder')"
          autocomplete="off"
          spellcheck="false"
          maxlength="120"
          list="server-history"
          @keydown.enter.prevent="submit(account.id, true)"
          @keydown.esc="toggleAdd(account.id)"
        >
        <input
          v-model="draftVersion"
          type="text"
          class="h-7 w-full rounded border border-slate-700 bg-slate-900 px-2 font-mono text-xs text-slate-100 outline-none placeholder:text-slate-500 focus:border-blue-500"
          :placeholder="`${t('sidebar.version')}: ${t('sidebar.versionAuto')}`"
          autocomplete="off"
          spellcheck="false"
          maxlength="32"
          @keydown.enter.prevent="submit(account.id, true)"
          @keydown.esc="toggleAdd(account.id)"
        >
        <div class="flex gap-1">
          <button
            type="button"
            class="h-7 flex-1 rounded bg-blue-600 px-2 text-[11px] font-medium text-white transition-colors hover:bg-blue-500 disabled:opacity-50"
            :disabled="draftServer.trim() === '' || busy"
            @click="submit(account.id, true)"
          >
            {{ t('sidebar.connectNow') }}
          </button>
          <button
            type="button"
            class="h-7 rounded border border-slate-700 px-2 text-[11px] text-slate-200 transition-colors hover:bg-slate-800 disabled:opacity-50"
            :disabled="draftServer.trim() === '' || busy"
            @click="submit(account.id, false)"
          >
            {{ t('sidebar.saveOnly') }}
          </button>
        </div>
        <p class="text-[10px] leading-snug text-slate-500">
          {{ t('sidebar.versionHint') }}
        </p>
      </div>
    </div>

    <p
      v-if="accounts.accounts.length === 0"
      class="px-3 py-2 text-xs text-slate-500"
    >
      {{ t('sidebar.empty') }}
    </p>

    <!-- Podpowiedzi adresów z historii (Etap 4). Jedna lista na całą aplikację. -->
    <datalist id="server-history">
      <option
        v-for="entry in settings.appSettings.serverHistory"
        :key="entry"
        :value="entry"
      />
    </datalist>
  </aside>
</template>

<script setup lang="ts">
// Zakładki: jedna na **połączenie** (konto + serwer), bo to jedno połączenie
// odpowiada jednemu procesowi MCC.

import { computed, nextTick, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { X } from '@lucide/vue'
import { isConnectionLive, statusDotClass, statusKey } from '../log/status'
import { useAccountsStore } from '../stores/accounts'
import { useLogsStore } from '../stores/logs'
import ConfirmModal from './ConfirmModal.vue'

const { t } = useI18n()
const accounts = useAccountsStore()
const logs = useLogsStore()

/** Pasek zakładek - przewijany w poziomie przy przepełnieniu. */
const tabsScroller = ref<HTMLElement | null>(null)

/** Elementy zakładek po identyfikatorze połączenia (do dosuwania do widoku). */
const tabElements = new Map<string, HTMLElement>()

/**
 * Zapamiętuje element zakładki. Vue woła tę funkcję z `null` przy odmontowaniu,
 * więc mapa sama się oczyszcza po zamknięciu zakładki.
 */
function setTabElement(id: string, element: unknown): void {
  if (element instanceof HTMLElement) tabElements.set(id, element)
  else tabElements.delete(id)
}

/**
 * Dosuwa aktywną zakładkę do widoku - bez tego przy kilkudziesięciu zakładkach
 * aktywna potrafi zostać poza ekranem, a użytkownik nie widzi, gdzie jest.
 *
 * Liczymy `scrollLeft` sami, zamiast `scrollIntoView`, żeby nie ruszać
 * ewentualnego przewijania przodków i żeby zachowanie było przewidywalne.
 */
async function scrollActiveIntoView(): Promise<void> {
  await nextTick()
  const container = tabsScroller.value
  const id = accounts.activeConnectionId
  if (!container || !id) return

  const element = tabElements.get(id)
  if (!element) return

  const left = element.offsetLeft
  const right = left + element.offsetWidth
  if (left < container.scrollLeft) {
    container.scrollLeft = left
  } else if (right > container.scrollLeft + container.clientWidth) {
    container.scrollLeft = right - container.clientWidth
  }
}

onMounted(() => {
  void scrollActiveIntoView()
})

// Zakładki zmieniają się przy przełączaniu, dodawaniu i po przywróceniu sesji.
watch(
  () => [accounts.activeConnectionId, accounts.connections.length],
  () => {
    void scrollActiveIntoView()
  },
)

/**
 * Kółko myszy nad paskiem zakładek przewija go poziomo. Gdy nie ma czego
 * przewijać, nie ruszamy zdarzenia - niech działa domyślne zachowanie.
 */
function onWheel(event: WheelEvent): void {
  const container = tabsScroller.value
  if (!container) return

  const maxScroll = container.scrollWidth - container.clientWidth
  if (maxScroll <= 0) return

  // Pionowe kółko przesuwamy w poziomie; oś pozioma (touchpad) działa jak dotąd.
  const delta = Math.abs(event.deltaY) > Math.abs(event.deltaX) ? event.deltaY : event.deltaX
  container.scrollLeft = Math.max(0, Math.min(maxScroll, container.scrollLeft + delta))
  event.preventDefault()
}

function unreadFor(id: string): number {
  // `logs.activeRevision` w zależności wymusza przeliczenie po partii linii.
  void logs.activeRevision
  return logs.unreadFor(id)
}

function accountLabel(accountId: string): string {
  return accounts.accounts.find((account) => account.id === accountId)?.label ?? accountId
}

/**
 * Zakładka czekająca na potwierdzenie zamknięcia (R-44).
 *
 * Zamknięcie zakładki zatrzymuje proces, więc pytamy tylko wtedy, gdy proces
 * jeszcze żyje (`starting`, `awaiting_login`, `running`, `stopping`).
 * Zatrzymana i po awarii zamyka się od razu - nie ma czego tracić.
 */
const pendingClose = ref<string | null>(null)
const pendingConnection = computed(
  () => accounts.connections.find((conn) => conn.id === pendingClose.value) ?? null,
)

function requestClose(id: string): void {
  if (isConnectionLive(accounts.statusOf(id))) pendingClose.value = id
  else void accounts.closeConnection(id)
}

async function confirmClose(): Promise<void> {
  const id = pendingClose.value
  pendingClose.value = null
  if (id) await accounts.closeConnection(id)
}

/** Identyfikator karty dla ARIA; panel w `App.vue` używa `panel-<id>` (R-27). */
function tabId(id: string): string {
  return `tab-${id}`
}
</script>

<template>
  <div
    v-if="accounts.connections.length > 0"
    ref="tabsScroller"
    class="relative flex shrink-0 items-stretch overflow-x-auto border-b border-slate-800 bg-slate-950"
    role="tablist"
    aria-orientation="horizontal"
    :aria-label="t('tabs.label')"
    @wheel="onWheel"
  >
    <!-- Zakładka i przycisk zamknięcia to osobne elementy: zagnieżdżony
         `<button>` w `<button>` jest niepoprawnym HTML-em i psuje dostępność. -->
    <div
      v-for="conn in accounts.connections"
      :key="conn.id"
      :ref="(el) => setTabElement(conn.id, el)"
      role="presentation"
      class="flex shrink-0 items-stretch border-b-2 transition-colors"
      :class="
        conn.id === accounts.activeConnectionId
          ? 'border-blue-500 bg-slate-900'
          : 'border-transparent bg-slate-950 hover:bg-slate-900/60'
      "
    >
      <button
        :id="tabId(conn.id)"
        type="button"
        role="tab"
        :aria-controls="`panel-${conn.id}`"
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
        :aria-label="t('tabs.close')"
        @click.stop="requestClose(conn.id)"
      >
        <X :size="12" />
      </button>
    </div>
  </div>

  <!-- Potwierdzenie zamknięcia działającej zakładki (R-44). Poza `role="tablist"`,
       bo dzieci listy zakładek mogą być tylko zakładkami. -->
  <ConfirmModal
    v-if="accounts.connections.length > 0"
    :open="pendingConnection !== null"
    :title="t('confirm.closeTabTitle')"
    :message="t('confirm.closeTabMessage', { server: pendingConnection?.server ?? '' })"
    :confirm-label="t('confirm.closeTabAction')"
    @close="pendingClose = null"
    @confirm="confirmClose"
  />
</template>

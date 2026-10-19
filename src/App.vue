<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import TitleBar from './components/TitleBar.vue'
import SettingsModal from './components/SettingsModal.vue'
import AddAccountModal from './components/AddAccountModal.vue'
import AccountSidebar from './components/AccountSidebar.vue'
import AccountTabs from './components/AccountTabs.vue'
import AccountHeader from './components/AccountHeader.vue'
import LoginBanner from './components/LoginBanner.vue'
import LogView from './components/LogView.vue'
import QuickActions from './components/QuickActions.vue'
import InputBar from './components/InputBar.vue'
import ErrorToasts from './components/ErrorToasts.vue'
import CloseConfirmModal from './components/CloseConfirmModal.vue'
import { events } from './api/backend'
import { useSettingsStore } from './stores/settings'
import { useAccountsStore } from './stores/accounts'
import { usePresetsStore } from './stores/presets'

const { t } = useI18n()
const settings = useSettingsStore()
const accounts = useAccountsStore()
const presets = usePresetsStore()

// Zdarzenia z Rusta podłączamy raz na cały czas życia aplikacji.
const unsubscribe = accounts.subscribe()

/**
 * Pytanie o zamknięcie okna. Rust wstrzymuje zamknięcie tylko wtedy, gdy
 * naprawdę działają połączenia, więc modal pokazujemy dokładnie na to zdarzenie.
 */
const closePrompt = ref(false)
const closePromptActive = ref(0)
const unlistenClose: Array<() => void> = []
void events
  .onCloseRequested((payload) => {
    closePromptActive.value = payload.active
    closePrompt.value = true
  })
  .then((unlisten) => unlistenClose.push(unlisten))

onUnmounted(() => {
  unsubscribe()
  for (const unlisten of unlistenClose) unlisten()
})

/**
 * Autoprzewijanie logu **per zakładka**: stan trzyma store kont (klucz =
 * identyfikator połączenia), więc pauza w jednej zakładce nie wyłącza śledzenia
 * w drugiej. Gdy nie ma otwartego połączenia, wartość nie ma znaczenia.
 */
const follow = computed<boolean>({
  get: () => {
    const id = accounts.activeConnectionId
    return id ? accounts.isFollowing(id) : true
  },
  set: (value: boolean) => {
    const id = accounts.activeConnectionId
    if (id) accounts.setFollowing(id, value)
  },
})

/** Modal ustawień; otwiera się też sam przy pierwszym uruchomieniu (brak MCC). */
const settingsOpen = ref(false)

/** Modal dodawania konta - otwiera go przycisk w nagłówku panelu kont. */
const addAccountOpen = ref(false)

/** Etykieta konta aktywnego połączenia (w nagłówku zakładki). */
function activeAccountLabel(): string {
  return accounts.activeAccount?.label ?? ''
}

/**
 * Pusty stan ma dwa warianty (R-29): bez kont podpowiada dodanie konta, z kontami
 * - otwarcie zakładki zieloną strzałką. Wcześniej oba pokazywały ten sam tekst.
 */
const hasAccounts = computed(() => accounts.accounts.length > 0)

onMounted(async () => {
  await settings.load()
  // Konta wracają do panelu po lewej **zawsze**; przełącznik dotyczy tylko zakładek.
  await accounts.restoreAccounts()
  // Okna połączeń z poprzedniej sesji wracają jako zatrzymane (bez procesów).
  await accounts.restoreOpenTabs(settings.appSettings.restoreTabs)
  // Szybkie akcje wczytujemy raz na całe życie aplikacji (store, nie komponent
  // paska, żeby przełączanie zakładek nie czytało pliku od nowa).
  await presets.load()
  if (!settings.appSettings.mccPath) settingsOpen.value = true
})
</script>

<template>
  <div class="flex h-full flex-col">
    <TitleBar @open-settings="settingsOpen = true" />

    <div class="flex min-h-0 flex-1">
      <AccountSidebar @add-account="addAccountOpen = true" />

      <div class="flex min-w-0 flex-1 flex-col">
        <AccountTabs />

        <!-- `tabpanel` wskazuje na aktywną zakładkę (R-27): czytnik ekranu wie,
             że ta treść należy do wybranej karty. -->
        <div
          v-if="accounts.activeConnection"
          :id="`panel-${accounts.activeConnection.id}`"
          role="tabpanel"
          :aria-labelledby="`tab-${accounts.activeConnection.id}`"
          class="flex min-h-0 flex-1 flex-col"
        >
          <AccountHeader
            v-model:follow="follow"
            :connection="accounts.activeConnection"
            :account-label="activeAccountLabel()"
          />
          <LoginBanner :connection-id="accounts.activeConnection.id" />
          <LogView
            v-model:follow="follow"
            :account-id="accounts.activeConnection.id"
          />
          <QuickActions />
          <InputBar />
        </div>

        <main
          v-else
          class="flex flex-1 items-center justify-center"
        >
          <div class="text-center">
            <h1 class="text-lg font-semibold text-slate-200">
              {{ hasAccounts ? t('empty.chooseTitle') : t('empty.title') }}
            </h1>
            <p class="mt-1 text-sm text-slate-400">
              {{ hasAccounts ? t('empty.chooseHint') : t('empty.hint') }}
            </p>
          </div>
        </main>
      </div>
    </div>

    <ErrorToasts />
    <SettingsModal
      :open="settingsOpen"
      @close="settingsOpen = false"
    />
    <AddAccountModal
      :open="addAccountOpen"
      @close="addAccountOpen = false"
    />
    <CloseConfirmModal
      :open="closePrompt"
      :active="closePromptActive"
      @close="closePrompt = false"
    />
  </div>
</template>

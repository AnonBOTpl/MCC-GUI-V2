<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { Languages } from '@lucide/vue'
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
import { useSettingsStore } from './stores/settings'
import { useAccountsStore } from './stores/accounts'
import { usePresetsStore } from './stores/presets'
import { SUPPORTED_LOCALES, type Locale } from './i18n'

const { t } = useI18n()
const settings = useSettingsStore()
const accounts = useAccountsStore()
const presets = usePresetsStore()

// Zdarzenia z Rusta podłączamy raz na cały czas życia aplikacji.
const unsubscribe = accounts.subscribe()
onUnmounted(unsubscribe)

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

    <div class="flex shrink-0 items-center justify-end gap-2 border-b border-slate-800 bg-slate-900/60 px-3 py-1.5">
      <Languages
        :size="14"
        class="text-slate-400"
      />
      <span class="text-xs text-slate-400">{{ t('language.label') }}</span>
      <div class="flex overflow-hidden rounded border border-slate-700">
        <button
          v-for="loc in SUPPORTED_LOCALES"
          :key="loc"
          type="button"
          class="px-2.5 py-1 text-xs transition-colors"
          :class="
            settings.locale === loc
              ? 'bg-blue-600 text-white'
              : 'bg-slate-900 text-slate-300 hover:bg-slate-800'
          "
          @click="settings.setLocale(loc as Locale)"
        >
          {{ loc.toUpperCase() }}
        </button>
      </div>
    </div>

    <div class="flex min-h-0 flex-1">
      <AccountSidebar @add-account="addAccountOpen = true" />

      <div class="flex min-w-0 flex-1 flex-col">
        <AccountTabs />

        <template v-if="accounts.activeConnection">
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
        </template>

        <main
          v-else
          class="flex flex-1 items-center justify-center"
        >
          <div class="text-center">
            <h1 class="text-lg font-semibold text-slate-200">
              {{ t('empty.title') }}
            </h1>
            <p class="mt-1 text-sm text-slate-400">
              {{ t('empty.hint') }}
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
  </div>
</template>

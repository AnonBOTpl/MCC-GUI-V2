<script setup lang="ts">
// Banner logowania Microsoft (Etap 3).
//
// Pokazuje kod device code i adres do otwarcia. Przycisk „Otwórz" jest akcją
// użytkownika - aplikacja nigdy nie otwiera przeglądarki sama.
// Adres jest walidowany ponownie w Rustcie (tylko `https://`).

import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { Check, Copy, ExternalLink, Info, KeyRound } from '@lucide/vue'
import { useAccountsStore } from '../stores/accounts'

// `connectionId` to identyfikator połączenia (`konto::serwer`) - prompt logowania
// dotyczy procesu, więc jest kluczowany połączeniem.
const props = defineProps<{ connectionId: string }>()

const { t } = useI18n()
const accounts = useAccountsStore()

const prompt = computed(() => accounts.loginPromptFor(props.connectionId))
const copied = ref(false)

async function copyCode(): Promise<void> {
  const current = prompt.value
  if (!current) return
  const ok = await accounts.copyLoginCode(current.code)
  if (!ok) return
  copied.value = true
  setTimeout(() => {
    copied.value = false
  }, 2000)
}

async function openLink(): Promise<void> {
  const current = prompt.value
  if (!current) return
  await accounts.openLoginUrl(current.url)
}
</script>

<template>
  <Transition name="banner">
    <section
      v-if="prompt"
      class="flex shrink-0 flex-wrap items-center gap-3 border-b border-sky-900 bg-sky-950/60 px-3 py-2"
    >
      <KeyRound
        :size="18"
        class="shrink-0 text-sky-300"
      />

      <div class="min-w-0">
        <p class="flex items-center gap-1.5 text-xs font-semibold text-sky-100">
          {{ t('loginBanner.title') }}
          <!-- Podpowiedź o wyborze konta; metoda device code nie przenosi loginu,
               więc o koncie decyduje sesja przeglądarki użytkownika. -->
          <span class="group relative inline-flex">
            <button
              type="button"
              class="flex h-6 w-6 items-center justify-center rounded-full text-sky-300 transition-colors hover:bg-sky-900 hover:text-sky-100"
              :aria-label="t('loginBanner.helpLabel')"
            >
              <Info :size="12" />
            </button>
            <span
              role="tooltip"
              class="pointer-events-none absolute top-full left-0 z-20 mt-1.5 hidden w-72 rounded border border-slate-700 bg-slate-900 px-2.5 py-2 text-left text-[11px] leading-snug font-normal text-slate-200 shadow-lg group-hover:block group-focus-within:block"
            >
              {{ t('loginBanner.help') }}
            </span>
          </span>
        </p>
        <p class="text-xs text-sky-200/80">
          {{ t('loginBanner.hint') }}
        </p>
      </div>

      <code
        class="select-text rounded border border-sky-800 bg-slate-950 px-3 py-1 font-mono text-sm font-semibold tracking-widest text-sky-100"
      >{{ prompt.code }}</code>

      <button
        type="button"
        class="flex h-7 shrink-0 items-center gap-1.5 rounded border border-sky-700 px-2 text-xs text-sky-100 transition-colors hover:bg-sky-900"
        @click="copyCode"
      >
        <Check
          v-if="copied"
          :size="13"
        />
        <Copy
          v-else
          :size="13"
        />
        {{ copied ? t('loginBanner.copied') : t('loginBanner.copy') }}
      </button>

      <span class="min-w-0 flex-1 truncate text-xs text-slate-400">{{ prompt.url }}</span>

      <button
        type="button"
        class="flex h-7 shrink-0 items-center gap-1.5 rounded bg-sky-700 px-2.5 text-xs font-medium text-white transition-colors hover:bg-sky-800"
        @click="openLink"
      >
        <ExternalLink :size="13" />
        {{ t('loginBanner.open') }}
      </button>
    </section>
  </Transition>
</template>

<style scoped>
.banner-enter-active,
.banner-leave-active {
  transition:
    opacity 0.15s ease,
    transform 0.15s ease;
}
.banner-enter-from,
.banner-leave-to {
  opacity: 0;
  transform: translateY(-4px);
}
</style>

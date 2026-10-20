<script setup lang="ts">
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { Send } from '@lucide/vue'
import { useAccountsStore } from '../stores/accounts'
import { useTransientNotice } from '../ui/notice'

const { t } = useI18n()
const accounts = useAccountsStore()

const text = ref('')
/**
 * Trwa wysyłka poprzedniego wpisu. Bez tej flagi drugi `submit` (Enter albo
 * dwuklik) w trakcie oczekiwania na IPC wysyłał ten sam tekst jeszcze raz,
 * bo pole czyścimy dopiero po odpowiedzi Rusta (S-01).
 */
const sending = ref(false)

/** Dlaczego wpis nie wyszedł: nie ma do kogo wysłać. */
type InputNotice = 'noneRunning' | 'noActive'

const NOTICE_KEYS: Record<InputNotice, string> = {
  noneRunning: 'input.noticeNoneRunning',
  noActive: 'input.noticeNoActive',
}

const { notice, show, clear } = useTransientNotice<InputNotice>()
const noticeText = computed(() => (notice.value ? t(NOTICE_KEYS[notice.value]) : ''))

/**
 * Wysyła wpis i czyści pole (wpis jest jednorazowy, jak w konsoli).
 *
 * Pole czyścimy **tylko wtedy, gdy wpis naprawdę wyszedł** (R-24): przy braku
 * odbiorców tekst zostaje w polu, a nad nim pojawia się krótkie wyjaśnienie.
 * Nie nadpisujemy też tego, co użytkownik zdążył wpisać podczas wysyłki.
 */
async function submit(): Promise<void> {
  if (sending.value) return
  const value = text.value.trim()
  if (value === '') return
  const toAll = accounts.sendToAll
  sending.value = true
  try {
    const sent = await accounts.send(value)

    if (sent === 0) {
      show(toAll ? 'noneRunning' : 'noActive')
      return
    }

    clear()
    if (text.value.trim() === value) text.value = ''
  } finally {
    sending.value = false
  }
}
</script>

<template>
  <div class="shrink-0 border-t border-slate-800 bg-slate-900/60">
    <!-- Komunikat nie zmienia szerokości pola ani położenia przycisków. -->
    <p
      v-if="noticeText"
      class="px-3 pt-1.5 text-[11px] text-amber-300"
      role="status"
    >
      {{ noticeText }}
    </p>

    <form
      class="flex items-center gap-2 px-3 py-2"
      @submit.prevent="submit"
    >
      <input
        v-model="text"
        type="text"
        class="h-8 flex-1 rounded border border-slate-700 bg-slate-900 px-3 font-mono text-xs text-slate-100 outline-none placeholder:text-slate-500 focus:border-blue-500"
        :placeholder="t('input.placeholder')"
        autocomplete="off"
        spellcheck="false"
        maxlength="512"
      >

      <label class="flex shrink-0 cursor-pointer items-center gap-1.5 text-xs text-slate-300">
        <input
          v-model="accounts.sendToAll"
          type="checkbox"
          class="h-3.5 w-3.5 accent-blue-500"
        >
        {{ t('input.sendToAll') }}
      </label>

      <button
        type="submit"
        class="flex h-8 shrink-0 items-center gap-1.5 rounded bg-blue-600 px-3 text-xs font-medium text-white transition-colors hover:bg-blue-700 active:bg-blue-800 disabled:cursor-not-allowed disabled:bg-slate-700 disabled:text-slate-400"
        :disabled="text.trim() === '' || sending"
      >
        <Send :size="14" />
        {{ t('input.send') }}
      </button>
    </form>
  </div>
</template>

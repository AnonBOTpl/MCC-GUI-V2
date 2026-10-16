<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { Send } from '@lucide/vue'
import { useAccountsStore } from '../stores/accounts'

const { t } = useI18n()
const accounts = useAccountsStore()

const text = ref('')

/** Wysyła wpis i czyści pole (wpis jest jednorazowy, jak w konsoli). */
async function submit(): Promise<void> {
  const value = text.value.trim()
  if (value === '') return
  text.value = ''
  await accounts.send(value)
}
</script>

<template>
  <form
    class="flex shrink-0 items-center gap-2 border-t border-slate-800 bg-slate-900/60 px-3 py-2"
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
      class="flex h-8 shrink-0 items-center gap-1.5 rounded bg-blue-600 px-3 text-xs font-medium text-white transition-colors hover:bg-blue-500 disabled:cursor-not-allowed disabled:bg-slate-700 disabled:text-slate-400"
      :disabled="text.trim() === ''"
    >
      <Send :size="14" />
      {{ t('input.send') }}
    </button>
  </form>
</template>

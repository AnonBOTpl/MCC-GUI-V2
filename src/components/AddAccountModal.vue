<script setup lang="ts">
// Dodawanie konta w modalu (wariant 1 z `docs/mockups/add-account-designs.html`).
//
// Po co modal, a nie pasek na górze: konto dodaje się raz, a pasek zajmował stały
// wiersz nad logiem i stał daleko od panelu kont, w którym konto się potem
// pokazuje. Tutaj akcja jest tam, gdzie skutek, a jest gdzie wytłumaczyć różnicę
// między kontem Microsoft i offline.
//
// Zmiany edytujemy od razu w kopii roboczej; „Anuluj", klik poza modalem i `Esc`
// nic nie zapisują. Konto powstaje dopiero po „Dodaj konto" i **nie uruchamia
// żadnego połączenia** - serwer wybiera się osobno w panelu.

import { computed, nextTick, onUnmounted, reactive, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { KeyRound, X } from '@lucide/vue'
import type { AccountType } from '../api/contract'
import { useAccountsStore } from '../stores/accounts'

const props = defineProps<{ open: boolean }>()
const emit = defineEmits<{ (event: 'close'): void }>()

const { t } = useI18n()
const accounts = useAccountsStore()

/** Karty wyboru typu konta: tytuł i jedno zdanie wyjaśnienia. */
const TYPE_OPTIONS: ReadonlyArray<{
  value: AccountType
  titleKey: string
  hintKey: string
}> = [
  { value: 'microsoft', titleKey: 'accountType.microsoft', hintKey: 'addAccount.typeMicrosoftHint' },
  { value: 'offline', titleKey: 'accountType.offline', hintKey: 'addAccount.typeOfflineHint' },
]

const form = reactive({
  type: 'microsoft' as AccountType,
  login: '',
  label: '',
})

const submitting = ref(false)
const loginInput = ref<HTMLInputElement | null>(null)

/** Podpowiedź loginu zależy od typu: e-mail dla Microsoft, ksywka dla offline. */
const loginPlaceholder = computed(() =>
  form.type === 'microsoft'
    ? t('addAccount.loginPlaceholder')
    : t('addAccount.loginPlaceholderOffline'),
)

const canSubmit = computed(() => form.login.trim() !== '' && !submitting.value)

/** Kolejne identyfikatory kont; trwałość kont pilnuje store. */
let counter = 0
function nextId(): string {
  counter += 1
  return `acc-${Date.now().toString(36)}-${counter}`
}

/** Każde otwarcie zaczyna od pustego formularza i ustawia fokus na loginie. */
watch(
  () => props.open,
  (open) => {
    if (!open) return
    form.type = 'microsoft'
    form.login = ''
    form.label = ''
    submitting.value = false
    void nextTick(() => loginInput.value?.focus())
  },
)

/** Zamyka modal z klawiatury (Escape), nic nie zapisując. */
function onKeydown(event: KeyboardEvent): void {
  if (event.key === 'Escape' && props.open) emit('close')
}
window.addEventListener('keydown', onKeydown)
onUnmounted(() => window.removeEventListener('keydown', onKeydown))

async function submit(): Promise<void> {
  if (!canSubmit.value) return
  submitting.value = true
  const login = form.login.trim()
  const ok = await accounts.addAccount({
    id: nextId(),
    // Pusta nazwa to login - tę samą regułę egzekwuje Rust przy zapisie.
    label: form.label.trim() || login,
    type: form.type,
    login,
  })
  submitting.value = false
  // Przy błędzie modal zostaje otwarty: toast z opisem pokazuje store kont.
  if (ok) emit('close')
}
</script>

<template>
  <Transition name="modal">
    <div
      v-if="open"
      class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4"
      @click.self="emit('close')"
    >
      <form
        class="flex w-full max-w-md flex-col rounded-lg border border-slate-700 bg-slate-900 shadow-2xl"
        @submit.prevent="submit"
      >
        <header class="flex shrink-0 items-center justify-between border-b border-slate-800 px-4 py-3">
          <h2 class="text-sm font-semibold text-slate-100">
            {{ t('addAccount.title') }}
          </h2>
          <button
            type="button"
            class="flex h-7 w-7 items-center justify-center rounded text-slate-400 transition-colors hover:bg-slate-800 hover:text-slate-100"
            :title="t('addAccount.cancel')"
            @click="emit('close')"
          >
            <X :size="16" />
          </button>
        </header>

        <div class="flex flex-col gap-4 px-4 py-4">
          <!-- Typ konta: karty zamiast listy rozwijanej, żeby dało się wytłumaczyć różnicę. -->
          <div
            role="radiogroup"
            :aria-label="t('addAccount.title')"
            class="grid grid-cols-2 gap-2"
          >
            <button
              v-for="option in TYPE_OPTIONS"
              :key="option.value"
              type="button"
              role="radio"
              :aria-checked="form.type === option.value"
              class="flex flex-col gap-1 rounded-md border bg-slate-950 p-2 text-left transition-colors"
              :class="
                form.type === option.value
                  ? 'border-blue-500 ring-1 ring-blue-500'
                  : 'border-slate-700 hover:border-slate-600'
              "
              @click="form.type = option.value"
            >
              <span class="text-xs font-semibold text-slate-100">{{ t(option.titleKey) }}</span>
              <span class="text-[11px] leading-snug text-slate-500">{{ t(option.hintKey) }}</span>
            </button>
          </div>

          <label class="flex flex-col gap-1">
            <span class="text-[11px] uppercase tracking-wide text-slate-400">
              {{ t('addAccount.login') }}
            </span>
            <input
              ref="loginInput"
              v-model="form.login"
              type="text"
              class="h-8 rounded border border-slate-700 bg-slate-950 px-2 text-xs text-slate-100 outline-none placeholder:text-slate-500 focus:border-blue-500"
              :placeholder="loginPlaceholder"
              autocomplete="off"
              spellcheck="false"
              maxlength="120"
            >
          </label>

          <label class="flex flex-col gap-1">
            <span class="flex items-baseline gap-2 text-[11px] uppercase tracking-wide text-slate-400">
              {{ t('addAccount.name') }}
              <span class="text-[10px] normal-case tracking-normal text-slate-500">
                {{ t('addAccount.nameHint') }}
              </span>
            </span>
            <input
              v-model="form.label"
              type="text"
              class="h-8 rounded border border-slate-700 bg-slate-950 px-2 text-xs text-slate-100 outline-none placeholder:text-slate-500 focus:border-blue-500"
              :placeholder="t('addAccount.namePlaceholder')"
              maxlength="40"
            >
          </label>

          <p
            class="flex items-start gap-2 rounded-md border border-blue-500/35 bg-blue-600/10 p-2 text-[11px] leading-snug text-slate-300"
          >
            <KeyRound
              :size="13"
              class="mt-0.5 shrink-0 text-blue-300"
            />
            {{ t('addAccount.noPassword') }}
          </p>
        </div>

        <footer class="flex shrink-0 items-center justify-end gap-2 border-t border-slate-800 px-4 py-3">
          <button
            type="button"
            class="h-8 rounded border border-slate-700 px-3 text-xs text-slate-200 transition-colors hover:bg-slate-800 disabled:opacity-60"
            :disabled="submitting"
            @click="emit('close')"
          >
            {{ t('addAccount.cancel') }}
          </button>
          <button
            type="submit"
            class="h-8 rounded bg-blue-600 px-4 text-xs font-medium text-white transition-colors hover:bg-blue-500 disabled:cursor-not-allowed disabled:opacity-60"
            :disabled="!canSubmit"
          >
            {{ t('addAccount.submit') }}
          </button>
        </footer>
      </form>
    </div>
  </Transition>
</template>

<style scoped>
.modal-enter-active,
.modal-leave-active {
  transition: opacity 0.15s ease;
}
.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}
</style>

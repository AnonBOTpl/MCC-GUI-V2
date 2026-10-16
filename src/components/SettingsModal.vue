<script setup lang="ts">
// Modal ustawień (Etap 4). Otwierany przyciskiem w pasku tytułu, a przy pierwszym
// uruchomieniu (brak ścieżki MCC) pokazuje się sam, żeby użytkownik od razu
// wskazał `MinecraftClient.exe`.
//
// Zmiany edytujemy na kopii roboczej i zapisujemy dopiero po „Zapisz", więc
// „Anuluj" (albo klik poza modalem) niczego nie psuje. Wybór pliku jest
// wyjątkiem: i tak waliduje go Rust, a wynik ląduje tylko w kopii roboczej.

import { onUnmounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { FileSearch, FolderOpen, X } from '@lucide/vue'
import { useSettingsStore } from '../stores/settings'

const props = defineProps<{ open: boolean }>()
const emit = defineEmits<{ (event: 'close'): void }>()

const { t } = useI18n()
const settings = useSettingsStore()

/** Kopia robocza: `null` = brak wybranej ścieżki. */
const draftMccPath = ref<string | null>(null)
const draftRestoreTabs = ref(true)
const draftConfirmClose = ref(true)
const draftDelay = ref('250')
const draftBuffer = ref('5000')
/** Logi czatu: `draftChatLogDir = null` oznacza folder domyślny (katalog danych/logs). */
const draftChatLogEnabled = ref(true)
const draftChatLogDir = ref<string | null>(null)
const draftChatLogRetention = ref('30')
const saving = ref(false)

/** Przepisuje bieżące ustawienia do kopii roboczej przy każdym otwarciu. */
function resetDraft(): void {
  draftMccPath.value = settings.appSettings.mccPath
  draftRestoreTabs.value = settings.appSettings.restoreTabs
  draftConfirmClose.value = settings.appSettings.confirmClose
  draftDelay.value = String(settings.appSettings.sendAllDelayMs)
  draftBuffer.value = String(settings.appSettings.logBufferSize)
  draftChatLogEnabled.value = settings.appSettings.chatLogEnabled
  draftChatLogDir.value = settings.appSettings.chatLogDir
  draftChatLogRetention.value = String(settings.appSettings.chatLogRetentionDays)
  // Efektywny folder pokazujemy też wtedy, gdy użytkownik nie wybrał własnego.
  void settings.loadChatLogDir()
}

watch(
  () => props.open,
  (open) => {
    if (open) resetDraft()
  },
)

/** Zamyka modal z klawiatury (Escape), bez zmieniania czegokolwiek. */
function onKeydown(event: KeyboardEvent): void {
  if (event.key === 'Escape' && props.open) emit('close')
}
window.addEventListener('keydown', onKeydown)
onUnmounted(() => window.removeEventListener('keydown', onKeydown))

/** Wybór pliku od razu trafia do kopii roboczej (ścieżkę waliduje Rust). */
async function browse(): Promise<void> {
  const picked = await settings.pickMccExecutable()
  if (picked) draftMccPath.value = picked
}

/** Wybór folderu logów czatu - także walidowany po stronie Rusta. */
async function browseChatLogDir(): Promise<void> {
  const picked = await settings.pickChatLogFolder()
  if (picked) draftChatLogDir.value = picked
}

/** Otwiera folder logów czatu w Eksploratorze (efektywny folder, nie kopia robocza). */
async function openChatLogFolder(): Promise<void> {
  await settings.openChatLogFolder()
}

/** Zapisuje kopię roboczą. Przy błędzie modal zostaje otwarty. */
async function confirm(): Promise<void> {
  const delay = Number(draftDelay.value)
  const buffer = Number(draftBuffer.value)
  const retention = Number(draftChatLogRetention.value)
  if (
    !Number.isFinite(delay) ||
    delay < 0 ||
    !Number.isFinite(buffer) ||
    buffer < 1 ||
    !Number.isFinite(retention) ||
    retention < 0
  ) {
    resetDraft()
    return
  }

  saving.value = true
  const ok = await settings.save({
    mccPath: draftMccPath.value,
    restoreTabs: draftRestoreTabs.value,
    confirmClose: draftConfirmClose.value,
    sendAllDelayMs: Math.round(delay),
    logBufferSize: Math.round(buffer),
    chatLogEnabled: draftChatLogEnabled.value,
    chatLogDir: draftChatLogDir.value,
    chatLogRetentionDays: Math.round(retention),
  })
  saving.value = false

  if (ok) emit('close')
  else resetDraft()
}
</script>

<template>
  <Transition name="modal">
    <div
      v-if="open"
      class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4"
      @click.self="emit('close')"
    >
      <div class="flex w-full max-w-xl flex-col rounded-lg border border-slate-700 bg-slate-900 shadow-2xl">
        <header class="flex shrink-0 items-center justify-between border-b border-slate-800 px-4 py-3">
          <h2 class="text-sm font-semibold text-slate-100">
            {{ t('settings.title') }}
          </h2>
          <button
            type="button"
            class="flex h-7 w-7 items-center justify-center rounded text-slate-400 transition-colors hover:bg-slate-800 hover:text-slate-100"
            :title="t('settings.close')"
            @click="emit('close')"
          >
            <X :size="16" />
          </button>
        </header>

        <div class="flex flex-col gap-5 px-4 py-4">
          <!-- Ścieżka do MinecraftClient.exe -->
          <section class="flex flex-col gap-2">
            <span class="text-[11px] uppercase tracking-wide text-slate-400">
              {{ t('settings.mccTitle') }}
            </span>
            <div class="flex items-center gap-2">
              <code
                class="min-w-0 flex-1 truncate rounded border border-slate-700 bg-slate-950 px-2 py-1.5 font-mono text-xs"
                :class="draftMccPath ? 'text-slate-200' : 'text-slate-500'"
                :title="draftMccPath ?? t('settings.mccNotSet')"
              >
                {{ draftMccPath ?? t('settings.mccNotSet') }}
              </code>
              <button
                type="button"
                class="flex h-8 shrink-0 items-center gap-1.5 rounded bg-blue-600 px-3 text-xs font-medium text-white transition-colors hover:bg-blue-500"
                @click="browse"
              >
                <FileSearch :size="14" />
                {{ t('settings.browse') }}
              </button>
            </div>
            <p class="text-xs text-slate-500">
              {{ t('settings.mccHint') }}
            </p>
          </section>

          <!-- Przywracanie zakładek -->
          <section class="flex items-start justify-between gap-4">
            <div class="min-w-0">
              <p class="text-xs font-medium text-slate-200">
                {{ t('settings.restoreTabs') }}
              </p>
              <p class="mt-0.5 text-xs text-slate-500">
                {{ t('settings.restoreTabsHint') }}
              </p>
            </div>
            <button
              type="button"
              role="switch"
              :aria-checked="draftRestoreTabs"
              :aria-label="t('settings.restoreTabs')"
              class="relative mt-0.5 h-5 w-9 shrink-0 rounded-full transition-colors"
              :class="draftRestoreTabs ? 'bg-blue-600' : 'bg-slate-700'"
              @click="draftRestoreTabs = !draftRestoreTabs"
            >
              <span
                class="absolute top-[2px] h-4 w-4 rounded-full bg-white transition-all"
                :class="draftRestoreTabs ? 'left-[18px]' : 'left-[2px]'"
              />
            </button>
          </section>

          <!-- Pytanie o potwierdzenie przy zamykaniu okna -->
          <section class="flex items-start justify-between gap-4">
            <div class="min-w-0">
              <p class="text-xs font-medium text-slate-200">
                {{ t('settings.confirmClose') }}
              </p>
              <p class="mt-0.5 text-xs text-slate-500">
                {{ t('settings.confirmCloseHint') }}
              </p>
            </div>
            <button
              type="button"
              role="switch"
              :aria-checked="draftConfirmClose"
              :aria-label="t('settings.confirmClose')"
              class="relative mt-0.5 h-5 w-9 shrink-0 rounded-full transition-colors"
              :class="draftConfirmClose ? 'bg-blue-600' : 'bg-slate-700'"
              @click="draftConfirmClose = !draftConfirmClose"
            >
              <span
                class="absolute top-[2px] h-4 w-4 rounded-full bg-white transition-all"
                :class="draftConfirmClose ? 'left-[18px]' : 'left-[2px]'"
              />
            </button>
          </section>

          <!-- Opóźnienie „wyślij do wszystkich" -->
          <section class="flex items-start justify-between gap-4">
            <div class="min-w-0">
              <p class="text-xs font-medium text-slate-200">
                {{ t('settings.sendDelay') }}
              </p>
              <p class="mt-0.5 text-xs text-slate-500">
                {{ t('settings.sendDelayHint') }}
              </p>
            </div>
            <div class="flex shrink-0 items-center gap-1.5">
              <input
                v-model="draftDelay"
                type="number"
                min="0"
                step="50"
                class="h-8 w-24 rounded border border-slate-700 bg-slate-950 px-2 text-right text-xs text-slate-100 outline-none focus:border-blue-500"
              >
              <span class="text-xs text-slate-500">{{ t('settings.sendDelayUnit') }}</span>
            </div>
          </section>

          <!-- Rozmiar bufora logu na konto -->
          <section class="flex items-start justify-between gap-4">
            <div class="min-w-0">
              <p class="text-xs font-medium text-slate-200">
                {{ t('settings.logBuffer') }}
              </p>
              <p class="mt-0.5 text-xs text-slate-500">
                {{ t('settings.logBufferHint') }}
              </p>
            </div>
            <div class="flex shrink-0 items-center gap-1.5">
              <input
                v-model="draftBuffer"
                type="number"
                min="200"
                step="500"
                class="h-8 w-24 rounded border border-slate-700 bg-slate-950 px-2 text-right text-xs text-slate-100 outline-none focus:border-blue-500"
              >
              <span class="text-xs text-slate-500">{{ t('settings.logBufferUnit') }}</span>
            </div>
          </section>

          <!-- Logi czatu (Paczka 4) -->
          <section class="flex flex-col gap-3 border-t border-slate-800 pt-4">
            <div class="flex items-start justify-between gap-4">
              <div class="min-w-0">
                <p class="text-xs font-medium text-slate-200">
                  {{ t('settings.chatLogToggle') }}
                </p>
                <p class="mt-0.5 text-xs text-slate-500">
                  {{ t('settings.chatLogHint') }}
                </p>
              </div>
              <button
                type="button"
                role="switch"
                :aria-checked="draftChatLogEnabled"
                :aria-label="t('settings.chatLogToggle')"
                class="relative mt-0.5 h-5 w-9 shrink-0 rounded-full transition-colors"
                :class="draftChatLogEnabled ? 'bg-blue-600' : 'bg-slate-700'"
                @click="draftChatLogEnabled = !draftChatLogEnabled"
              >
                <span
                  class="absolute top-[2px] h-4 w-4 rounded-full bg-white transition-all"
                  :class="draftChatLogEnabled ? 'left-[18px]' : 'left-[2px]'"
                />
              </button>
            </div>

            <div class="flex flex-col gap-1.5">
              <p class="text-xs font-medium text-slate-200">
                {{ t('settings.chatLogDir') }}
              </p>
              <div class="flex items-center gap-2">
                <code
                  class="min-w-0 flex-1 truncate rounded border border-slate-700 bg-slate-950 px-2 py-1.5 font-mono text-xs"
                  :class="draftChatLogDir || settings.chatLogDir ? 'text-slate-200' : 'text-slate-500'"
                  :title="draftChatLogDir ?? settings.chatLogDir ?? ''"
                >
                  {{ draftChatLogDir ?? settings.chatLogDir ?? t('settings.chatLogDefault') }}
                </code>
                <button
                  type="button"
                  class="flex h-8 shrink-0 items-center gap-1.5 rounded border border-slate-700 px-3 text-xs text-slate-200 transition-colors hover:bg-slate-800"
                  @click="browseChatLogDir"
                >
                  {{ t('settings.chatLogChange') }}
                </button>
                <button
                  type="button"
                  class="flex h-8 shrink-0 items-center gap-1.5 rounded border border-slate-700 px-3 text-xs text-slate-200 transition-colors hover:bg-slate-800"
                  @click="openChatLogFolder"
                >
                  <FolderOpen :size="14" />
                  {{ t('settings.chatLogOpen') }}
                </button>
              </div>
              <p class="text-xs text-slate-500">
                {{ t('settings.chatLogDirHint') }}
              </p>
              <button
                v-if="draftChatLogDir"
                type="button"
                class="self-start text-xs text-blue-300 transition-colors hover:text-blue-200"
                @click="draftChatLogDir = null"
              >
                {{ t('settings.chatLogUseDefault') }}
              </button>
            </div>

            <div class="flex items-start justify-between gap-4">
              <div class="min-w-0">
                <p class="text-xs font-medium text-slate-200">
                  {{ t('settings.chatLogRetention') }}
                </p>
                <p class="mt-0.5 text-xs text-slate-500">
                  {{ t('settings.chatLogRetentionHint') }}
                </p>
              </div>
              <div class="flex shrink-0 items-center gap-1.5">
                <input
                  v-model="draftChatLogRetention"
                  type="number"
                  min="0"
                  step="1"
                  class="h-8 w-24 rounded border border-slate-700 bg-slate-950 px-2 text-right text-xs text-slate-100 outline-none focus:border-blue-500"
                >
                <span class="text-xs text-slate-500">{{ t('settings.chatLogRetentionUnit') }}</span>
              </div>
            </div>
          </section>
        </div>

        <footer class="flex shrink-0 items-center justify-end gap-2 border-t border-slate-800 px-4 py-3">
          <button
            type="button"
            class="h-8 rounded border border-slate-700 px-3 text-xs text-slate-200 transition-colors hover:bg-slate-800 disabled:opacity-60"
            :disabled="saving"
            @click="emit('close')"
          >
            {{ t('settings.cancel') }}
          </button>
          <button
            type="button"
            class="h-8 rounded bg-blue-600 px-4 text-xs font-medium text-white transition-colors hover:bg-blue-500 disabled:cursor-not-allowed disabled:opacity-60"
            :disabled="saving"
            @click="confirm"
          >
            {{ t('settings.save') }}
          </button>
        </footer>
      </div>
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

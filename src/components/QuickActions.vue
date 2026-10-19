<script setup lang="ts">
// Pasek szybkich akcji (Etap 6).
//
// Na dole okna jest tylko cienki uchwyt (17 px); klik rozwija panel z presetami,
// drugi klik zwija. Panel rośnie w normalnym przepływie (`grid-template-rows`
// 0fr -> 1fr), więc **zabiera wysokość logowi, a nie zasłania go**. Limit trzech
// linii akcji z wewnętrznym przewijaniem (decyzja użytkownika, 2026-10-10).
//
// Stanu rozwinięcia świadomie nie zapamiętujemy: pasek startuje zwinięty, więc
// po restarcie log ma pełną wysokość.
//
// Na pasku są tylko akcje **włączone** - wyłączone zostają w edytorze i znikają
// z paska. Zasięg wysyłki bierze się z checkboxa „Wyślij do wszystkich” przy polu
// wpisywania (wspólnego z wiadomościami), więc podpowiedź przycisku pokazuje, co
// naprawdę się stanie w danym momencie.

import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { ChevronUp, Pencil, RefreshCw } from '@lucide/vue'
import type { Preset } from '../api/contract'
import { enabledPresets } from '../presets/model'
import { presetColorClass } from '../presets/palette'
import { useAccountsStore } from '../stores/accounts'
import { usePresetsStore, type PresetNotice } from '../stores/presets'
import PresetEditor from './PresetEditor.vue'

const { t } = useI18n()
const presets = usePresetsStore()
const accounts = useAccountsStore()

/** Pasek startuje zwinięty (stanu nie zapamiętujemy). */
const open = ref(false)
const editorOpen = ref(false)

/** Komunikaty przy pasku: po przeładowaniu pliku albo gdy nie było gdzie wysłać. */
const NOTICE_KEYS: Record<PresetNotice['kind'], string> = {
  reloaded: 'quickActions.noticeReloaded',
  noneRunning: 'quickActions.noticeNoneRunning',
  noActive: 'quickActions.noticeNoActive',
}

const noticeText = computed(() => {
  const notice = presets.notice
  if (!notice) return ''
  if (notice.kind === 'reloaded') {
    return t(NOTICE_KEYS.reloaded, { count: notice.count })
  }
  return t(NOTICE_KEYS[notice.kind])
})

/** Akcje widoczne na pasku (wyłączone siedzą tylko w edytorze). */
const shown = computed(() => enabledPresets(presets.presets))
const hasAnyPreset = computed(() => presets.presets.length > 0)
const countText = computed(() => t('quickActions.count', { count: shown.value.length }))

/** Podpowiedź pokazuje **dokładny** tekst i aktualny zasięg wysyłki. */
function pillTitle(preset: Preset): string {
  return accounts.sendToAll
    ? t('quickActions.sendsToAll', { text: preset.text })
    : t('quickActions.sends', { text: preset.text })
}
</script>

<template>
  <section class="shrink-0 border-t border-slate-800 bg-slate-900/40">
    <!-- Uchwyt: kreska trzyma się środka, opis i strzałka po prawej -->
    <button
      type="button"
      class="group relative block h-[17px] w-full cursor-pointer text-slate-400 transition-colors hover:bg-blue-600/10 hover:text-slate-300"
      :aria-expanded="open"
      aria-controls="quick-actions-panel"
      :aria-label="open ? t('quickActions.collapse') : t('quickActions.expand')"
      :title="open ? t('quickActions.collapse') : t('quickActions.expand')"
      @click="open = !open"
    >
      <span
        class="absolute left-1/2 top-[7px] h-[3px] w-[54px] -translate-x-1/2 rounded-full bg-slate-700 transition-all duration-[180ms] ease-out group-hover:w-[76px] group-hover:bg-blue-500"
        :class="open ? 'bg-blue-500' : ''"
      />
      <span
        class="absolute right-3 top-[2px] flex items-center gap-1.5 text-[10.5px] uppercase tracking-wide transition-opacity duration-150"
        :class="open ? 'opacity-100' : 'opacity-0 group-hover:opacity-100'"
      >
        {{ countText }}
        <ChevronUp
          :size="12"
          class="transition-transform duration-[180ms] ease-out"
          :class="open ? 'rotate-180' : ''"
        />
      </span>
    </button>

    <!-- Panel: animowany przez `grid-template-rows`, więc działa przy każdej liczbie linii.
         Zwinięty dostaje `inert`, bo sama wysokość 0 nie usuwa go z klawiatury:
         bez tego `Tab` wchodził w pigułki i `Enter` wysyłał akcję z niewidocznego paska. -->
    <div
      id="quick-actions-panel"
      class="grid overflow-hidden transition-[grid-template-rows] duration-[180ms] ease-out"
      :style="{ gridTemplateRows: open ? '1fr' : '0fr' }"
      :inert="open ? undefined : true"
      :aria-hidden="open ? undefined : 'true'"
    >
      <div class="min-h-0">
        <!-- Maksymalnie trzy linie akcji (7+26+6+26+6+26+7 = 104 px); resztę się przewija -->
        <div class="qa-scroll max-h-[104px] overflow-y-auto overscroll-contain border-t border-dashed border-slate-800">
          <div class="flex flex-wrap items-center gap-1.5 px-3 py-[7px]">
            <span class="text-[10.5px] uppercase tracking-wide text-slate-400">
              {{ t('quickActions.title') }}
            </span>

            <span
              v-if="noticeText"
              class="text-[11px] text-amber-300"
            >
              {{ noticeText }}
            </span>

            <span
              v-if="presets.loaded && shown.length === 0 && !noticeText"
              class="text-xs text-slate-400"
            >
              {{ hasAnyPreset ? t('quickActions.allOff') : t('quickActions.empty') }}
            </span>

            <button
              v-for="preset in shown"
              :key="preset.id"
              type="button"
              class="relative inline-flex h-[26px] shrink-0 items-center overflow-hidden rounded-full border border-slate-700 bg-slate-900 pl-[9px] pr-2.5 text-xs text-slate-100 transition-colors hover:border-blue-500 hover:bg-slate-800"
              :title="pillTitle(preset)"
              @click="presets.run(preset)"
            >
              <span
                v-if="preset.color"
                class="absolute inset-y-0 left-0 w-[3px]"
                :class="presetColorClass(preset.color)"
              />
              <span class="ml-1">{{ preset.label }}</span>
            </button>

            <span class="flex-1" />

            <button
              v-if="!hasAnyPreset"
              type="button"
              class="flex h-[26px] shrink-0 items-center gap-1.5 rounded border border-slate-700 bg-blue-600 px-2.5 text-xs text-white transition-colors hover:bg-blue-700 active:bg-blue-800"
              @click="editorOpen = true"
            >
              <Pencil :size="12" />
              {{ t('quickActions.addActions') }}
            </button>

            <template v-else>
              <button
                type="button"
                class="flex h-[26px] shrink-0 items-center gap-1.5 rounded border border-slate-700 px-2.5 text-xs text-slate-200 transition-colors hover:bg-slate-800"
                @click="editorOpen = true"
              >
                <Pencil :size="12" />
                {{ t('quickActions.edit') }}
              </button>
              <button
                type="button"
                class="flex h-[26px] shrink-0 items-center gap-1.5 rounded border border-slate-700 px-2.5 text-xs text-slate-200 transition-colors hover:bg-slate-800"
                :title="t('quickActions.reloadHint')"
                @click="presets.reload()"
              >
                <RefreshCw :size="12" />
                {{ t('quickActions.reload') }}
              </button>
            </template>
          </div>
        </div>
      </div>
    </div>

    <PresetEditor
      :open="editorOpen"
      @close="editorOpen = false"
    />
  </section>
</template>

<style scoped>
/* Pasek przewijania jest cienki i w kolorach ramek; `scrollbar-gutter` pilnuje,
   żeby pigułki nie przesunęły się w bok, gdy pasek się pojawi. */
.qa-scroll {
  scrollbar-gutter: stable;
}
.qa-scroll::-webkit-scrollbar {
  width: 9px;
}
.qa-scroll::-webkit-scrollbar-track {
  background: transparent;
}
.qa-scroll::-webkit-scrollbar-thumb {
  background: rgb(51 65 85);
  border-radius: 999px;
  border: 2px solid transparent;
  background-clip: content-box;
}
.qa-scroll::-webkit-scrollbar-thumb:hover {
  background: rgb(100 116 139);
  background-clip: content-box;
}
</style>

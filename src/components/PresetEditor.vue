<script setup lang="ts">
// Edytor presetów szybkich akcji (Etap 6).
//
// Jedno miejsce na dodawanie, zmianę kolejności, kolor i usuwanie. Pracujemy na
// **kopii roboczej**: `Cancel` (albo klik poza modalem, albo Escape) porzuca
// wszystko, a dopiero `Save` wysyła listę do Rusta, który ją waliduje i zapisuje
// do `presets.json`. Dzięki temu usuwanie wiersza nie wymaga potwierdzenia -
// jest odwracalne do momentu zapisu.
//
// Kolejność wierszy to kolejność pigułek na pasku, więc przeciąganie uchwytu
// przestawia je na żywo. Używamy **zdarzeń wskaźnika**, a nie HTML5 drag & drop:
// `dragstart` na zwykłym elemencie bywa w WebView2 przechwytywany przez obsługę
// upuszczania plików i przeciąganie wtedy nie startuje (zgłoszenie użytkownika).
// `pointerdown` + `pointermove` + `pointerup` z przechwyceniem wskaźnika działają
// niezależnie od tego i dają się sprawdzić syntetycznymi zdarzeniami w testach.
//
// Przełącznik w wierszu włącza i wyłącza **akcję**: wyłączona znika z paska,
// ale zostaje tutaj. Zasięgu wysyłki (jedno połączenie czy wszystkie) nie ma
// w presecie - o tym decyduje checkbox przy polu wpisywania.

import { computed, onUnmounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { GripVertical, Trash2 } from '@lucide/vue'
import type { Preset, PresetColor } from '../api/contract'
import { PRESET_COLOR_OPTIONS, presetColorClass } from '../presets/palette'
import { moveItem, newPresetId, rowIndexAt } from '../presets/model'
import { usePresetsStore } from '../stores/presets'
import BaseModal from './BaseModal.vue'

const props = defineProps<{ open: boolean }>()
const emit = defineEmits<{ (event: 'close'): void }>()

const { t } = useI18n()
const presets = usePresetsStore()

/** Kopia robocza listy; zapis idzie dopiero po `Save`. */
const draft = ref<Preset[]>([])
const saving = ref(false)

/** Pola nowej akcji. */
const newLabel = ref('')
const newText = ref('')
const newColor = ref<PresetColor | null>(null)

/** Kontener wierszy - potrzebny do mierzenia ich pozycji przy przeciąganiu. */
const rowList = ref<HTMLElement | null>(null)

/** Indeks wiersza trzymanego kursorem (podświetlenie na czas przeciągania). */
const draggingIndex = ref<number | null>(null)

/** Wiersz aktualnie przesuwany - trzymany poza `ref`, bo to stan samego gestu. */
let dragFrom: number | null = null

/** Nazwy kolorów w polu wyboru (klucze i18n trzymamy jako literały). */
const COLOR_KEYS: Record<PresetColor, string> = {
  blue: 'presets.colorBlue',
  green: 'presets.colorGreen',
  amber: 'presets.colorAmber',
  red: 'presets.colorRed',
  violet: 'presets.colorViolet',
}

/** Gotowe przykłady dla pustej listy (`/send` jest wymagane dla komend serwera). */
const EXAMPLES: Array<{ label: string; text: string }> = [
  { label: '/home', text: '/send /home' },
  { label: '/bal', text: '/send /bal' },
  { label: '/reco', text: '/send /reco' },
]

const canSave = computed(
  () =>
    !saving.value &&
    draft.value.every((preset) => preset.label.trim() !== '' && preset.text.trim() !== ''),
)
const canAdd = computed(() => newLabel.value.trim() !== '' && newText.value.trim() !== '')

/** Przepisuje bieżące presety do kopii roboczej przy każdym otwarciu. */
function resetDraft(): void {
  draft.value = presets.presets.map((preset) => ({ ...preset }))
  newLabel.value = ''
  newText.value = ''
  newColor.value = null
  draggingIndex.value = null
  dragFrom = null
}

// `immediate` jest istotne: modal może zostać pokazany od razu przy montowaniu,
// a wtedy zmiana `open` nigdy nie zajdzie i kopia robocza zostałaby pusta.
watch(
  () => props.open,
  (open) => {
    if (open) resetDraft()
  },
  { immediate: true },
)

// `Esc` i pułapkę fokusu obsługuje `BaseModal`.
onUnmounted(() => {
  window.removeEventListener('pointermove', onPointerMove)
  window.removeEventListener('pointerup', onPointerUp)
  window.removeEventListener('pointercancel', onPointerUp)
})

function addNew(): void {
  if (!canAdd.value) return
  draft.value = [
    ...draft.value,
    {
      id: newPresetId(),
      label: newLabel.value.trim(),
      text: newText.value.trim(),
      color: newColor.value,
      enabled: true,
    },
  ]
  newLabel.value = ''
  newText.value = ''
  newColor.value = null
}

/** Dodaje gotowy przykład (tylko przy pustej liście). */
function addExample(example: { label: string; text: string }): void {
  draft.value = [
    ...draft.value,
    {
      id: newPresetId(),
      label: example.label,
      text: example.text,
      color: null,
      enabled: true,
    },
  ]
}

function removeAt(index: number): void {
  draft.value = draft.value.filter((_, position) => position !== index)
}

/** Podpis przełącznika mówi wprost, co zrobi kliknięcie. */
function toggleLabel(row: Preset): string {
  return row.enabled ? t('presets.toggleOff') : t('presets.toggleOn')
}

function onPointerDown(event: PointerEvent, index: number): void {
  // Tylko lewy przycisk (w jsdom zdarzenie może nie mieć `button`).
  if (event.button > 0) return
  event.preventDefault()

  dragFrom = index
  draggingIndex.value = index
  const grip = event.currentTarget as HTMLElement
  if (typeof grip.setPointerCapture === 'function') grip.setPointerCapture(event.pointerId)

  window.addEventListener('pointermove', onPointerMove)
  window.addEventListener('pointerup', onPointerUp)
  window.addEventListener('pointercancel', onPointerUp)
}

/**
 * Przeciąganie przestawia wiersze **na żywo**: po każdej zmianie pozycji
 * zapamiętujemy nowy indeks, więc można przesuwać dalej bez puszczania.
 */
function onPointerMove(event: PointerEvent): void {
  const from = dragFrom
  const list = rowList.value
  if (from === null || list === null) return

  const rows = [...list.querySelectorAll<HTMLElement>('[data-preset-row]')]
  const target = rowIndexAt(
    rows.map((row) => {
      const box = row.getBoundingClientRect()
      return { top: box.top, bottom: box.bottom }
    }),
    event.clientY,
  )
  if (target === null || target === from) return

  draft.value = moveItem(draft.value, from, target)
  dragFrom = target
  draggingIndex.value = target
}

function onPointerUp(): void {
  dragFrom = null
  draggingIndex.value = null
  window.removeEventListener('pointermove', onPointerMove)
  window.removeEventListener('pointerup', onPointerUp)
  window.removeEventListener('pointercancel', onPointerUp)
}

/** Zapisuje listę; przy błędzie modal zostaje otwarty (błąd leci w toast). */
async function confirm(): Promise<void> {
  if (!canSave.value) return
  saving.value = true
  const saved = await presets.save(draft.value)
  saving.value = false
  if (saved) emit('close')
}
</script>

<template>
  <BaseModal
    :open="open"
    :label="t('presets.title')"
    :close-label="t('presets.close')"
    width="max-w-2xl"
    body-class="gap-2 py-3"
    @close="emit('close')"
  >
    <template #header>
      <h2 class="text-sm font-semibold text-slate-100">
        {{ t('presets.title') }}
      </h2>
      <span class="text-xs text-slate-500">{{ t('presets.subtitle') }}</span>
    </template>

    <p class="text-[11px] text-slate-500">
      {{ t('presets.listHint') }}
    </p>

    <p
      v-if="draft.length === 0"
      class="text-xs text-slate-400"
    >
      {{ t('presets.emptyList') }}
    </p>

    <!-- Przykłady na start: żeby nie trzeba było zgadywać składni `/send` -->
    <div
      v-if="draft.length === 0"
      class="flex flex-col gap-1.5"
    >
      <span class="text-[11px] uppercase tracking-wide text-slate-500">
        {{ t('presets.examples') }}
      </span>
      <div
        v-for="example in EXAMPLES"
        :key="example.text"
        class="flex items-center gap-2 rounded-lg border border-slate-800 bg-slate-950/60 px-2 py-1.5"
      >
        <div class="min-w-0 flex-1">
          <p class="truncate text-xs text-slate-200">
            {{ example.label }}
          </p>
          <p class="truncate font-mono text-[11px] text-slate-500">
            {{ example.text }}
          </p>
        </div>
        <button
          type="button"
          class="h-[26px] shrink-0 rounded border border-slate-700 px-2.5 text-xs text-slate-200 transition-colors hover:bg-slate-800"
          @click="addExample(example)"
        >
          {{ t('presets.addExample') }}
        </button>
      </div>
    </div>

    <!-- Lista akcji -->
    <div
      ref="rowList"
      class="flex flex-col gap-2"
    >
      <div
        v-for="(row, index) in draft"
        :key="row.id"
        data-preset-row
        class="flex items-center gap-2 rounded-lg border bg-slate-950/60 px-2 py-1.5"
        :class="draggingIndex === index ? 'border-dashed border-blue-500 opacity-60' : 'border-slate-800'"
      >
        <span
          data-preset-grip
          class="shrink-0 cursor-grab touch-none select-none text-slate-500"
          :title="t('presets.dragHandle')"
          @pointerdown="onPointerDown($event, index)"
        >
          <GripVertical :size="14" />
        </span>

        <!-- Przełącznik włącza i wyłącza **akcję** (wyłączona znika z paska) -->
        <button
          type="button"
          role="switch"
          data-preset-toggle
          :aria-checked="row.enabled"
          :aria-label="toggleLabel(row)"
          :title="toggleLabel(row)"
          class="relative h-5 w-9 shrink-0 rounded-full transition-colors"
          :class="row.enabled ? 'bg-blue-600' : 'bg-slate-700'"
          @click="row.enabled = !row.enabled"
        >
          <span
            class="absolute top-[2px] h-4 w-4 rounded-full bg-white transition-all"
            :class="row.enabled ? 'left-[18px]' : 'left-[2px]'"
          />
        </button>

        <span
          class="h-[26px] w-[10px] shrink-0 rounded"
          :class="row.color ? presetColorClass(row.color) : 'swatch-none'"
        />

        <select
          v-model="row.color"
          class="h-[26px] w-[86px] shrink-0 rounded border border-slate-700 bg-slate-900 px-1.5 text-[11px] text-slate-200 outline-none focus:border-blue-500"
          :title="t('presets.color')"
        >
          <option :value="null">
            {{ t('presets.colorNone') }}
          </option>
          <option
            v-for="color in PRESET_COLOR_OPTIONS"
            :key="color"
            :value="color"
          >
            {{ t(COLOR_KEYS[color]) }}
          </option>
        </select>

        <!-- Zgaszone pola pokazują, że akcja jest wyłączona -->
        <div
          class="flex min-w-0 flex-1 flex-col gap-1 transition-opacity"
          :class="row.enabled ? '' : 'opacity-50'"
        >
          <input
            v-model="row.label"
            type="text"
            maxlength="40"
            class="h-[26px] rounded border border-slate-800 bg-slate-950 px-2 text-xs text-slate-100 outline-none placeholder:text-slate-600 focus:border-blue-500"
            :placeholder="t('presets.labelPlaceholder')"
            :aria-label="t('presets.labelLabel')"
          >
          <input
            v-model="row.text"
            type="text"
            maxlength="512"
            spellcheck="false"
            class="h-[26px] rounded border border-slate-800 bg-slate-950 px-2 font-mono text-[11px] text-slate-300 outline-none placeholder:text-slate-600 focus:border-blue-500"
            :placeholder="t('presets.textPlaceholder')"
            :aria-label="t('presets.textLabel')"
          >
        </div>

        <button
          type="button"
          class="flex h-6 w-6 shrink-0 items-center justify-center rounded text-slate-500 transition-colors hover:bg-slate-800 hover:text-red-400"
          :title="t('presets.delete')"
          @click="removeAt(index)"
        >
          <Trash2 :size="13" />
        </button>
      </div>
    </div>

    <!-- Dodawanie nowej akcji -->
    <div class="mt-1 flex flex-col gap-1.5 border-t border-slate-800 pt-3">
      <span class="text-[11px] uppercase tracking-wide text-slate-500">
        {{ t('presets.addSection') }}
      </span>
      <div class="flex items-center gap-2">
        <input
          v-model="newLabel"
          type="text"
          maxlength="40"
          class="h-8 w-32 shrink-0 rounded border border-slate-700 bg-slate-950 px-2 text-xs text-slate-100 outline-none placeholder:text-slate-600 focus:border-blue-500"
          :placeholder="t('presets.labelPlaceholder')"
          :aria-label="t('presets.labelLabel')"
        >
        <input
          v-model="newText"
          type="text"
          maxlength="512"
          spellcheck="false"
          class="h-8 min-w-0 flex-1 rounded border border-slate-700 bg-slate-950 px-2 font-mono text-xs text-slate-100 outline-none placeholder:text-slate-600 focus:border-blue-500"
          :placeholder="t('presets.textPlaceholder')"
          :aria-label="t('presets.textLabel')"
        >
        <button
          type="button"
          class="h-8 shrink-0 rounded bg-blue-600 px-3 text-xs font-medium text-white transition-colors hover:bg-blue-500 disabled:cursor-not-allowed disabled:bg-slate-700 disabled:text-slate-400"
          :disabled="!canAdd"
          @click="addNew"
        >
          {{ t('presets.add') }}
        </button>
      </div>

      <div class="flex items-center gap-2">
        <span class="text-xs text-slate-400">{{ t('presets.newColor') }}</span>
        <button
          type="button"
          class="swatch-none h-5 w-5 rounded border transition-colors"
          :class="newColor === null ? 'border-blue-500' : 'border-slate-700'"
          :title="t('presets.colorNone')"
          @click="newColor = null"
        />
        <button
          v-for="color in PRESET_COLOR_OPTIONS"
          :key="color"
          type="button"
          class="h-5 w-5 rounded border transition-colors"
          :class="[presetColorClass(color), newColor === color ? 'border-blue-400' : 'border-slate-700']"
          :title="t(COLOR_KEYS[color])"
          @click="newColor = color"
        />
      </div>

      <p class="text-[11px] text-slate-500">
        {{ t('presets.sendTip') }}
      </p>
    </div>

    <template #footer>
      <span
        class="mr-auto text-xs"
        :class="canSave ? 'text-slate-500' : 'text-amber-300'"
      >
        {{ canSave ? t('presets.hint') : t('presets.invalidHint') }}
      </span>
      <button
        type="button"
        class="h-8 rounded border border-slate-700 px-3 text-xs text-slate-200 transition-colors hover:bg-slate-800 disabled:opacity-60"
        :disabled="saving"
        @click="emit('close')"
      >
        {{ t('presets.cancel') }}
      </button>
      <button
        type="button"
        class="h-8 rounded bg-blue-600 px-4 text-xs font-medium text-white transition-colors hover:bg-blue-500 disabled:cursor-not-allowed disabled:opacity-60"
        :disabled="!canSave"
        @click="confirm"
      >
        {{ t('presets.save') }}
      </button>
    </template>
  </BaseModal>
</template>

<style scoped>
/* Brak koloru pokazujemy jako „przekątną kratkę”, żeby odróżnić go od czarnego
   kwadratu (mockup Etapu 6). */
.swatch-none {
  background: repeating-linear-gradient(45deg, #1e293b 0 4px, #0c1424 4px 8px);
}
</style>

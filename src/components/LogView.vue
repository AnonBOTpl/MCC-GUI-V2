<script setup lang="ts">
// Wirtualizowany widok logu: tylko widoczne wiersze są w DOM.
//
// Tekst zawija się o **granicę okna** (dynamicznie - przy zmianie rozmiaru okna
// zawija się na nowo). Aby wiersze nigdy się nie nakładały, renderujemy je w
// normalnym przepływie (blok pod blokiem) i wirtualizujemy za pomocą dwóch
// „spacerów": górnego i dolnego. Wysokość spacerów bierzemy z sumy wysokości
// wierszy; wysokość **renderowanych** wierszy mierzymy z DOM (są dokładne dla
// każdej szerokości), a dla wierszy poza oknem szacujemy z liczby kolumn.
//
// Formatowanie (`§` i ANSI) liczymy w dwóch krokach:
//   1. raz na partię linii powstaje tablica stanów formatowania (`formatStates`),
//   2. przy przewijaniu parsujemy tylko widoczne linie.

import { computed, nextTick, onMounted, onUnmounted, onUpdated, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { ArrowDown, ClipboardCopy, Copy } from '@lucide/vue'
import type { LogLine } from '../log/buffer'
import {
  formatStateAfter,
  initialFormat,
  parseSegments,
  stripFormatting,
  type FormatState,
  type TextSegment,
} from '../log/color'
import { diagCounters, setViewProbe, type ViewProbe } from '../log/diag'
import { followAfterScroll } from '../log/follow'
import {
  allLinesText,
  pruneSelection,
  selectionText,
  type RowSelection,
  type SeqBounds,
} from '../log/selection'
import { registerWakeHandlers } from '../log/wake'
import { wrapRowCount } from '../log/wrap'
import { useAccountsStore } from '../stores/accounts'
import { useLogsStore } from '../stores/logs'
import { useSelectionStore } from '../stores/selection'

const props = defineProps<{ accountId: string; follow: boolean }>()
const emit = defineEmits<{ (event: 'update:follow', value: boolean): void }>()

const { t } = useI18n()
const logs = useLogsStore()
const accounts = useAccountsStore()
const selections = useSelectionStore()

/** Wysokość jednego wiersza wizualnego — zgodna z `leading-[18px]`. */
const ROW_HEIGHT = 18
const OVERSCAN = 12
/** Poziomy padding wiersza (`px-3`) po obu stronach — w całości na tekst. */
const ROW_PADDING_X = 24
/** Domyślna szerokość znaku, dopóki nie zmierzymy jej w DOM. */
const FALLBACK_CHAR_WIDTH = 7.8

const viewport = ref<HTMLElement | null>(null)
const probe = ref<HTMLElement | null>(null)
const scrollTop = ref(0)
const viewportHeight = ref(0)
const viewportWidth = ref(0)
const charWidth = ref(FALLBACK_CHAR_WIDTH)

/**
 * Licznik zmian pozycji przewijania.
 *
 * `atBottom()` czyta DOM, więc nie jest reaktywne - a przycisk śledzenia musi
 * się schować także wtedy, gdy widok dociągamy **sami** (np. po kliknięciu tego
 * przycisku). Bez tego licznika `showJumpButton` przeliczyłby się tylko przy
 * zmianie `follow` i zostałby widoczny mimo widoku na samym dole.
 */
const scrollTick = ref(0)

/**
 * Zmierzone wysokości wierszy, kluczowane numerem (`seq`), bo indeksy przesuwają
 * się przy obcinaniu bufora. `measuredVersion` wymusza przeliczenie pozycji po
 * pomiarze (Map nie jest reaktywny).
 */
const measured = new Map<number, number>()
const measuredVersion = ref(0)

const total = computed(() => {
  void logs.activeRevision
  return logs.sizeFor(props.accountId)
})

const allLines = computed<readonly LogLine[]>(() => {
  void logs.activeRevision
  return logs.linesFor(props.accountId)
})

/**
 * Stan formatowania po każdej linii bufora. Liczony raz na partię linii, dzięki
 * czemu MCC może rozbijać komunikat na kilka linii bez resetu koloru, a my i tak
 * odtworzymy kolory poprawnie.
 */
const formatStates = computed<FormatState[]>(() => {
  const states: FormatState[] = []
  let state = initialFormat()
  for (const line of allLines.value) {
    state = formatStateAfter(line.raw, state)
    states.push(state)
  }
  return states
})

/** Liczba kolumn tekstu mieszczących się w wierszu (po odjęciu paddingu). */
const columns = computed(() => {
  const usable = viewportWidth.value - ROW_PADDING_X
  const width = charWidth.value
  if (usable <= 0 || !Number.isFinite(width) || width <= 0) return 1
  const cols = Math.floor(usable / width)
  return Number.isFinite(cols) && cols >= 1 ? cols : 1
})

/** Wysokość wiersza: zmierzona dla widocznych, oszacowana dla pozostałych. */
const lineHeights = computed<number[]>(() => {
  void measuredVersion.value
  const cols = columns.value
  return allLines.value.map((line) => {
    const cached = measured.get(line.seq)
    if (cached !== undefined) return cached
    const rows = wrapRowCount(stripFormatting(line.raw), cols)
    const safe = Number.isFinite(rows) && rows >= 1 ? rows : 1
    return safe * ROW_HEIGHT
  })
})

/** Skumulowane przesunięcia wierszy — do znalezienia widocznego zakresu i spacerów. */
const offsets = computed<number[]>(() => {
  const heights = lineHeights.value
  const result = new Array<number>(heights.length + 1)
  result[0] = 0
  for (let index = 0; index < heights.length; index += 1) {
    result[index + 1] = result[index] + heights[index]
  }
  return result
})

/** Zakres widocznych wierszy (z zapasem), znajduje go binary search. */
const range = computed(() => {
  const offs = offsets.value
  const count = total.value
  if (count === 0) return { first: 0, last: 0 }

  const top = scrollTop.value
  let low = 0
  let high = count
  while (low < high) {
    const mid = (low + high) >> 1
    if (offs[mid] <= top) low = mid + 1
    else high = mid
  }

  const first = Math.max(0, low - 1 - OVERSCAN)
  const limit = top + viewportHeight.value
  let last = first
  while (last < count && offs[last] < limit) last += 1
  last = Math.min(count, last + OVERSCAN)
  return { first, last }
})

const topSpacer = computed(() => offsets.value[range.value.first] ?? 0)
const bottomSpacer = computed(
  () => (offsets.value[total.value] ?? 0) - (offsets.value[range.value.last] ?? 0),
)

/** Widoczne wiersze — tylko one są parsowane na segmenty. */
const renderedRows = computed(() => {
  const lines = allLines.value
  const states = formatStates.value
  const { first, last } = range.value
  const rows: Array<{ seq: number; isError: boolean; segments: TextSegment[] }> = []

  for (let index = first; index < last; index += 1) {
    const line = lines[index]
    const before = index === 0 ? undefined : states[index - 1]
    rows.push({
      seq: line.seq,
      // `stderr` traktujemy jak błąd już teraz.
      isError: line.stream === 'stderr',
      segments: parseSegments(line.raw, before),
    })
  }
  return rows
})

/* -------------------------------------------------------------------------
 * Zaznaczanie i kopiowanie wierszy (Paczka 3)
 *
 * Zaznaczamy wiersze po numerze `seq`, a nie natywnym zaznaczaniem tekstu, bo
 * lista jest wirtualizowana (poza oknem nie ma elementów DOM). Logika siedzi
 * w `src/log/selection.ts`, a stan - osobno dla każdego połączenia - w store
 * `stores/selection.ts`, więc przełączenie zakładki nie gubi zaznaczenia.
 * ------------------------------------------------------------------------- */

/** Zaznaczenie tego połączenia; zmiana zakładki przestawia referencję. */
const selection = ref<RowSelection>(selections.forConnection(props.accountId))

/** Liczba zaznaczonych wierszy (reaktywna - `Set` w store jest reaktywny). */
const selectedCount = computed(() => selection.value.selected.size)

/** Pozycja menu kontekstowego (`null` = zamknięte). */
const menu = ref<{ x: number; y: number } | null>(null)
const menuElement = ref<HTMLElement | null>(null)

/** Przybliżony rozmiar menu - tylko po to, żeby nie wystawało poza okno. */
const MENU_WIDTH = 168
const MENU_HEIGHT = 72

function isSelected(seq: number): boolean {
  return selection.value.selected.has(seq)
}

/**
 * Klasy wiersza: zaznaczenie wygrywa z tłem błędu i z podświetleniem `hover`.
 * Zaznaczony wiersz opisuje klasa ze `<style>` - tło z kryciem i wewnętrzny pasek
 * z lewej nie zmieniają szerokości tekstu, więc zawijanie i pomiary wysokości
 * wierszy zostają bez zmian.
 */
function rowClass(row: { seq: number; isError: boolean }): string {
  if (isSelected(row.seq)) return 'log-row-selected'
  return row.isError ? 'bg-red-950/40 hover:bg-slate-900/60' : 'hover:bg-slate-900/60'
}

/** Zakres numerów linii, które są jeszcze w buforze (do przycinania zakresów). */
function seqBounds(): SeqBounds {
  const lines = allLines.value
  const first = lines[0]?.seq
  const last = lines[lines.length - 1]?.seq
  if (first === undefined || last === undefined) return { min: 0, max: -1 }
  return { min: first, max: last }
}

/** Fokus w widoku logu - skróty (`Ctrl+A`, `Ctrl+C`, `Esc`) działają tylko tutaj. */
function focusViewport(): void {
  viewport.value?.focus()
}

/**
 * Czyści natywne zaznaczenie tekstu, żeby nie kłóciło się z podświetleniem
 * wierszy (np. po wcześniejszym zaznaczeniu myszką).
 */
function clearNativeSelection(): void {
  const native = window.getSelection?.()
  if (native && !native.isCollapsed) native.removeAllRanges()
}

/** Klik w wiersz (z modyfikatorami `Ctrl` i `Shift` - patrz tabela w Paczce 3). */
function onRowClick(event: MouseEvent, seq: number): void {
  focusViewport()
  const id = props.accountId

  if (event.shiftKey) {
    if (event.ctrlKey || event.metaKey) selections.addRangeFromAnchor(id, seq, seqBounds())
    else selections.selectRangeFromAnchor(id, seq, seqBounds())
  } else if (event.ctrlKey || event.metaKey) {
    selections.toggleRow(id, seq)
  } else {
    selections.selectRow(id, seq)
  }
  clearNativeSelection()
}

/**
 * `Shift+klik` nie może uruchomić natywnego zaznaczania tekstu - inaczej
 * przeglądarka zaznaczałaby tekst obok naszego podświetlenia (blokujemy tylko
 * `Shift`, więc da się nadal zaznaczyć np. fragment adresu z logu).
 */
function onRowMouseDown(event: MouseEvent): void {
  if (!event.shiftKey) return
  event.preventDefault()
  focusViewport()
}

/** Menu kontekstowe: prawy przycisk myszy w widoku logu. */
function onContextMenu(event: MouseEvent): void {
  focusViewport()
  menu.value = {
    x: Math.max(0, Math.min(event.clientX, window.innerWidth - MENU_WIDTH)),
    y: Math.max(0, Math.min(event.clientY, window.innerHeight - MENU_HEIGHT)),
  }
}

function closeMenu(): void {
  menu.value = null
}

/**
 * Zapisuje w schowku tekst zaznaczonych wierszy (walidacja tekstu jest w Rustcie).
 * Zwraca `false`, gdy nic nie wysłano albo Rust odrzucił zapis - wtedy nie ruszamy
 * zaznaczenia, żeby użytkownik nie stracił go przez nieudane kopiowanie.
 */
async function copyText(text: string): Promise<boolean> {
  if (text === '') return false
  return accounts.copyText(text)
}

/**
 * Czyści zaznaczenie i **wraca do autoprzewijania**, dociągając widok do końca logu
 * (decyzja użytkownika: po skopiowaniu i po `Esc` widok jedzie za nowymi liniami).
 */
function clearSelectionAndFollow(): void {
  selections.clear(props.accountId)
  emit('update:follow', true)
  stickToBottom()
}

/** „Kopiuj”: zaznaczone wiersze z bieżącego bufora, bez kodów kolorów. */
async function copySelectedLines(): Promise<void> {
  // Linie, które obciął bufor, przestają być zaznaczone - kopiujemy to, co zostało.
  const lines = allLines.value
  pruneSelection(selection.value, lines)
  const copied = await copyText(selectionText(lines, selection.value))
  if (!copied) return
  // Po udanym skopiowaniu zaznaczenie znika, a widok wraca na koniec logu.
  clearSelectionAndFollow()
}

/** „Kopiuj” z menu kontekstowego. */
async function copySelected(): Promise<void> {
  closeMenu()
  focusViewport()
  await copySelectedLines()
}

/** „Kopiuj wszystko”: cały bufor połączenia. */
async function copyAll(): Promise<void> {
  closeMenu()
  focusViewport()
  await copyText(allLinesText(allLines.value))
}

/**
 * Skróty klawiszowe widoku logu - jedyne skróty w aplikacji (decyzja z Paczki 3).
 *
 * Wiszą na elemencie widoku logu, więc pole wpisywania komend (poza nim) ma
 * nietknięte `Ctrl+A` i `Ctrl+C`. Gdy nie mamy własnego zaznaczenia, `Ctrl+C`
 * przepuszczamy - działa wtedy natywne kopiowanie zaznaczonego tekstu.
 */
function onKeydown(event: KeyboardEvent): void {
  if (event.key === 'Escape') {
    if (menu.value) {
      closeMenu()
      event.preventDefault()
      return
    }
    if (selectedCount.value > 0) {
      clearSelectionAndFollow()
      event.preventDefault()
    }
    return
  }
  if (!event.ctrlKey && !event.metaKey) return

  const key = event.key.toLowerCase()
  if (key === 'a') {
    event.preventDefault()
    selections.selectAll(
      props.accountId,
      allLines.value.map((line) => line.seq),
    )
    clearNativeSelection()
    return
  }
  if (key === 'c') {
    if (selectedCount.value === 0) return
    event.preventDefault()
    void copySelectedLines()
  }
}

/** Zamknięcie menu przy kliknięciu poza nim (nasze własne pozycje klikamy normalnie). */
function onDocumentPointerDown(event: PointerEvent): void {
  if (!menu.value) return
  const element = menuElement.value
  if (element && event.target instanceof Node && element.contains(event.target)) return
  closeMenu()
}

/** Przewijanie zamyka menu - inaczej zostałoby w miejscu, gdzie już nic nie ma. */
function onDocumentScroll(): void {
  if (menu.value) closeMenu()
}

// Zaznaczenie wyłącza autoprzewijanie tej zakładki (flaga per połączenie),
// żeby nowe linie nie uciekały z widoku w trakcie kopiowania.
watch(selectedCount, (count) => {
  if (count > 0) emit('update:follow', false)
})

/**
 * Czy ostatnia zmiana pozycji to nasze własne dociągnięcie do dołu.
 *
 * Nasze przewinięcie też podnosi zdarzenie `scroll` i bez tego znacznika
 * autoprzewijanie wyłączałoby się samo: tuż po przerysowaniu pomiar wysokości
 * wierszy potrafi przesunąć treść o kilkadziesiąt pikseli, a wtedy widok
 * wygląda na odjechany od dołu, choć nikt go nie ruszył.
 */
let selfScroll = false

function onScroll(): void {
  const element = viewport.value
  if (!element) return
  scrollTop.value = element.scrollTop
  scrollTick.value += 1
  if (selfScroll) {
    selfScroll = false
    return
  }
  const next = followAfterScroll(props.follow, atBottom())
  if (next !== null) emit('update:follow', next)
}

/**
 * Realne wejście użytkownika unieważnia znacznik - na wypadek, gdyby nasze
 * własne zdarzenie `scroll` w ogóle nie przyszło (pozycja się nie zmieniła).
 */
function markUserScroll(): void {
  selfScroll = false
}

function scrollToBottom(): void {
  const element = viewport.value
  if (!element) return
  selfScroll = true
  element.scrollTop = element.scrollHeight
  scrollTop.value = element.scrollTop
  scrollTick.value += 1
}

/**
 * Dociąga widok na dół i powtarza próbę po ustaleniu wysokości wierszy.
 *
 * Jedno `scrollTop = scrollHeight` nie wystarcza: tuż po przerysowaniu
 * `measureRows` koryguje zmierzone wysokości (a więc i spacery), przez co treść
 * potrafi przesunąć się o kilka wierszy w dół. Druga próba - już po pomiarze -
 * wyrównuje to bez udziału użytkownika.
 */
function stickToBottom(): void {
  void nextTick(() => {
    scrollToBottom()
    void nextTick(scrollToBottom)
  })
}

/** Czy użytkownik jest na dole (z tolerancją dwóch wierszy). */
function atBottom(): boolean {
  const element = viewport.value
  if (!element) return true
  return element.scrollHeight - element.scrollTop - element.clientHeight < ROW_HEIGHT * 2
}

/**
 * Pierwsze zdarzenie wejścia po powrocie do okna: domyka zaległe odświeżenie
 * od razu, bez czekania na klatkę (patrz `src/log/wake.ts`). Przewinięcie na dół
 * robi `watch` na `logs.activeRevision`, gdy śledzimy.
 */
function onWake(): void {
  // Diagnostyka: licznik zdarzeń wejścia (patrz `src/log/diag.ts`).
  diagCounters.wake += 1
  logs.flushNow()
}

/**
 * Stan widoku dla diagnostyki (opcjonalnej - patrz `src/log/diag.ts`).
 * Dzięki temu raport pokazuje, czy dane są już w buforze, ile wierszy jest
 * w DOM i gdzie stoi przewijanie - czyli czy brakuje danych, czy malowania.
 */
function viewSnapshot(): ViewProbe {
  const element = viewport.value
  return {
    bufferSize: logs.sizeFor(props.accountId),
    rowsInDom: renderedRows.value.length,
    scrollHeight: element?.scrollHeight ?? 0,
    scrollTop: element?.scrollTop ?? 0,
    clientHeight: element?.clientHeight ?? 0,
    follow: props.follow,
  }
}

/** Mierzy szerokość okna i wzorcowego znaku (do szacowania wierszy poza oknem). */
function measureLayout(): void {
  const element = viewport.value
  if (element) {
    viewportHeight.value = element.clientHeight
    viewportWidth.value = element.clientWidth
  }
  const template = probe.value
  const length = template?.textContent?.length ?? 0
  if (template && length > 0 && template.offsetWidth > 0) {
    const width = template.offsetWidth / length
    if (Number.isFinite(width) && width > 0) charWidth.value = width
  }
}

/**
 * Mierzy rzeczywistą wysokość renderowanych wierszy. Dla tekstu zawijanego to
 * jedyny dokładny sposób — szacunek z liczby kolumn służy tylko wierszom poza
 * oknem. Po zmianie wysokości Vue przelicza pozycje (przez `measuredVersion`).
 */
function measureRows(): void {
  const container = viewport.value
  if (!container) return
  let changed = false
  container.querySelectorAll<HTMLElement>('[data-seq]').forEach((node) => {
    const seq = Number(node.dataset.seq)
    const height = node.offsetHeight
    if (!Number.isFinite(seq) || height <= 0) return
    const previous = measured.get(seq)
    if (previous === undefined || Math.abs(previous - height) > 0.5) {
      measured.set(seq, height)
      changed = true
    }
  })
  if (changed) measuredVersion.value += 1
}

let resizeObserver: ResizeObserver | undefined
let disposeWakeHandlers: (() => void) | undefined

onMounted(() => {
  measureLayout()
  disposeWakeHandlers = registerWakeHandlers(window, onWake)
  // Zamknięcie menu kontekstowego przy kliknięciu poza nim i przy przewijaniu
  // (nasłuch w fazie przechwytywania, bo przewija się wewnętrzny kontener).
  document.addEventListener('pointerdown', onDocumentPointerDown, true)
  window.addEventListener('scroll', onDocumentScroll, true)
  // Diagnostyka: widok zgłasza swój stan (patrz `src/log/diag.ts`).
  setViewProbe(viewSnapshot)
  const element = viewport.value
  if (element) {
    resizeObserver = new ResizeObserver(() => {
      measureLayout()
    })
    resizeObserver.observe(element)
  }
  void nextTick(() => {
    measureRows()
    stickToBottom()
  })
})

onUpdated(() => {
  // Diagnostyka: licznik przerysowań widoku (patrz `src/log/diag.ts`).
  diagCounters.renders += 1
  measureRows()
})

onUnmounted(() => {
  disposeWakeHandlers?.()
  document.removeEventListener('pointerdown', onDocumentPointerDown, true)
  window.removeEventListener('scroll', onDocumentScroll, true)
  setViewProbe(null)
  resizeObserver?.disconnect()
})

// Zmiana szerokości unieważnia zmierzone wysokości (tekst zawija się inaczej).
watch(columns, () => {
  measured.clear()
  measuredVersion.value += 1
})

// Nowe linie: gdy śledzimy, przewijamy na dół po aktualizacji DOM. Ta sama
// droga obsługuje odświeżenie domknięte przez `onWake()`.
watch(
  () => logs.activeRevision,
  () => {
    if (props.follow) stickToBottom()
  },
)

// Zmiana konta: czyścimy pomiary, zamykamy menu i idziemy na dół. Zaznaczenie
// jest trzymane per połączenie, więc przełączenie zakładki go nie gubi.
watch(
  () => props.accountId,
  (id) => {
    selection.value = selections.forConnection(id)
    closeMenu()
    measured.clear()
    measuredVersion.value += 1
    scrollTop.value = 0
    stickToBottom()
  },
)

// Włączenie śledzenia: dociągnij na dół.
watch(
  () => props.follow,
  (value) => {
    if (value) stickToBottom()
  },
)

/** Wznowienie autoprzewijania przyciskiem przy przewiniętym widoku. */
function resumeFollow(): void {
  emit('update:follow', true)
  stickToBottom()
}

const showJumpButton = computed(() => {
  // `scrollTick` to zależność tylko po to, żeby pozycja przewijania też
  // unieważniała ten `computed`.
  void scrollTick.value
  return !props.follow || !atBottom()
})
</script>

<template>
  <div class="relative min-h-0 flex-1">
    <!-- Niewidoczna sonda: mierzy szerokość znaku w tej samej czcionce. -->
    <span
      ref="probe"
      aria-hidden="true"
      class="log-text pointer-events-none absolute left-0 top-0 -z-10 whitespace-pre text-[13px] opacity-0"
    >MMMMMMMMMMMMMMMMMMMM</span>

    <!-- `tabindex` na widoku logu: `Ctrl+A`/`Ctrl+C`/`Esc` działają tylko wtedy,
         gdy fokus jest tutaj (pole wpisywania komend jest poza tym elementem). -->
    <div
      ref="viewport"
      class="log-text absolute inset-0 overflow-auto bg-slate-950 text-[13px] leading-[18px]"
      style="scrollbar-gutter: stable"
      tabindex="0"
      @scroll="onScroll"
      @wheel="markUserScroll"
      @pointerdown="markUserScroll"
      @keydown="onKeydown"
      @contextmenu.prevent="onContextMenu"
    >
      <div
        v-if="total === 0"
        class="p-4 text-xs text-slate-500"
      >
        {{ t('log.empty') }}
      </div>

      <template v-else>
        <div :style="{ height: `${topSpacer}px` }" />

        <!-- Normalny przepływ: wiersze zawijają się o granicę okna i nie mogą
             się na siebie nałożyć. Tylko widoczny zakres jest w DOM. -->
        <div
          v-for="row in renderedRows"
          :key="row.seq"
          :data-seq="row.seq"
          class="whitespace-pre-wrap break-all px-3"
          :class="rowClass(row)"
          @mousedown="onRowMouseDown"
          @click="onRowClick($event, row.seq)"
        >
          <span
            v-for="(segment, index) in row.segments"
            :key="index"
            :style="{
              color: segment.color,
              fontWeight: segment.bold ? 700 : 400,
              fontStyle: segment.italic ? 'italic' : 'normal',
              textDecoration: [
                segment.underline ? 'underline' : '',
                segment.strikethrough ? 'line-through' : '',
              ]
                .filter(Boolean)
                .join(' '),
            }"
          >{{ segment.text }}</span>
        </div>

        <div :style="{ height: `${bottomSpacer}px` }" />
      </template>
    </div>

    <!-- Menu kontekstowe (prawy przycisk myszy w widoku logu). Teksty przechodzą
         przez i18n, tak jak reszta interfejsu. -->
    <Transition name="fade">
      <div
        v-if="menu"
        ref="menuElement"
        class="fixed z-30 min-w-[168px] rounded border border-slate-700 bg-slate-900 py-1 shadow-lg"
        :style="{ left: `${menu.x}px`, top: `${menu.y}px` }"
        role="menu"
        @keydown="onKeydown"
      >
        <button
          type="button"
          role="menuitem"
          class="flex w-full items-center gap-2 px-3 py-1.5 text-left text-xs text-slate-200 transition-colors enabled:hover:bg-slate-800 disabled:cursor-default disabled:text-slate-500"
          :disabled="selectedCount === 0"
          @click="copySelected"
        >
          <Copy :size="13" />
          {{ t('log.menu.copy') }}
        </button>
        <button
          type="button"
          role="menuitem"
          class="flex w-full items-center gap-2 px-3 py-1.5 text-left text-xs text-slate-200 transition-colors enabled:hover:bg-slate-800 disabled:cursor-default disabled:text-slate-500"
          :disabled="total === 0"
          @click="copyAll"
        >
          <ClipboardCopy :size="13" />
          {{ t('log.menu.copyAll') }}
        </button>
      </div>
    </Transition>

    <Transition name="fade">
      <button
        v-if="showJumpButton"
        type="button"
        class="absolute bottom-3 right-4 z-10 flex items-center gap-1.5 rounded-full border border-slate-700 bg-slate-800/95 px-3 py-1.5 text-xs text-slate-100 shadow-lg transition-colors hover:bg-slate-700"
        @click="resumeFollow"
      >
        <ArrowDown :size="13" />
        {{ t('log.follow') }}
      </button>
    </Transition>
  </div>
</template>

<style scoped>
/* Zaznaczony wiersz: tło z kryciem jest czytelne na kolorach Minecrafta,
   a wewnętrzny pasek z lewej nie zmienia szerokości tekstu ani wysokości
   wiersza (żadnych obramowań ani paddingu), więc wirtualizacja działa jak dotąd. */
.log-row-selected {
  background-color: rgb(30 58 138 / 0.6);
  box-shadow: inset 2px 0 0 0 rgb(96 165 250);
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.15s ease;
}
.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>

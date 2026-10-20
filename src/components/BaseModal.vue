<script setup lang="ts">
// Wspólny szkielet okien modalnych (R-06).
//
// Cztery modale (dodawanie konta, ustawienia, potwierdzenie zamknięcia, edytor
// presetów) miały skopiowany ten sam overlay, nagłówek, stopkę i przejście.
// To jedno miejsce daje im:
// - semantykę `role="dialog"` + `aria-modal` + nazwę dla czytników ekranu,
// - pułapkę fokusu (`Tab`/`Shift+Tab` krąży w panelu, nie ucieka pod overlay),
// - `Esc` i klik poza panelem jako zamknięcie,
// - powrót fokusu na element, który modal otworzył,
// - przewijanie treści, gdy panel nie mieści się na ekranie.
//
// Treść (nagłówek, ciało, stopka) należy do modala - tutaj są tylko sloty, więc
// każdy modal zachowuje swój układ.

import { nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { X } from '@lucide/vue'
import { isTopModal, popModal, pushModal } from '../ui/modalStack'

const props = withDefaults(
  defineProps<{
    open: boolean
    /** Nazwa okna dla czytników ekranu (widoczny nagłówek jest w slocie). */
    label: string
    /** Podpowiedź przycisku X. */
    closeLabel: string
    /** `form` daje semantykę formularza: Enter w polu wysyła zdarzenie `submit`. */
    as?: 'div' | 'form'
    /** Szerokość panelu (klasa Tailwind). */
    width?: string
    /** Dodatkowe klasy ciała - każdy modal ma własne odstępy. */
    bodyClass?: string
    /** `false` blokuje `Esc` i klik poza panelem (np. w trakcie zamykania aplikacji). */
    dismissible?: boolean
  }>(),
  {
    as: 'div',
    width: 'max-w-md',
    bodyClass: 'gap-4 py-4',
    dismissible: true,
  },
)
const emit = defineEmits<{ (event: 'close'): void; (event: 'submit'): void }>()

const panel = ref<HTMLElement | null>(null)
/** Wpis w stosie modali; tylko wierzchni modal obsługuje `Esc` i `Tab` (S-02). */
const modalId = Symbol('modal')
/** Element, który miał fokus przed otwarciem - tu wraca po zamknięciu. */
let restoreFocus: HTMLElement | null = null

/** Wszystko, co da się sfokusować klawiszem Tab (kolejność dokumentu). */
const FOCUSABLE = [
  'a[href]',
  'button:not([disabled])',
  'input:not([disabled]):not([type="hidden"])',
  'select:not([disabled])',
  'textarea:not([disabled])',
  '[tabindex]:not([tabindex="-1"])',
].join(',')

function focusableItems(root: HTMLElement): HTMLElement[] {
  return Array.from(root.querySelectorAll<HTMLElement>(FOCUSABLE))
}

function setPanel(element: unknown): void {
  panel.value = element instanceof HTMLElement ? element : null
}

/** `Esc` zamyka, `Tab`/`Shift+Tab` nie wychodzą poza panel. */
function onKeydown(event: KeyboardEvent): void {
  if (!props.open) return
  // Modal pod wierzchem nie może reagować: jeden `Esc` zamykałby oba naraz,
  // a druga pułapka fokusu walczyłaby z tą na wierzchu.
  if (!isTopModal(modalId)) return
  if (event.key === 'Escape') {
    if (!props.dismissible) return
    event.preventDefault()
    emit('close')
    return
  }
  if (event.key !== 'Tab') return

  const root = panel.value
  if (!root) return
  const items = focusableItems(root)
  if (items.length === 0) {
    event.preventDefault()
    root.focus()
    return
  }

  const first = items[0]
  const last = items[items.length - 1]
  const active = document.activeElement
  const inside = active instanceof HTMLElement && root.contains(active)

  if (event.shiftKey) {
    if (inside && active !== first) return
    event.preventDefault()
    last.focus()
    return
  }
  if (inside && active !== last) return
  event.preventDefault()
  first.focus()
}

/** `Esc` i klik poza panelem respektują `dismissible`. */
function requestClose(): void {
  if (!props.dismissible) return
  emit('close')
}

// Fokus przenosimy dopiero po przerysowaniu (`v-if`), a przy zamykaniu wracamy
// na element, który modal otworzył (np. przycisk w pasku tytułu).
watch(
  () => props.open,
  async (open) => {
    if (open) {
      pushModal(modalId)
      restoreFocus = document.activeElement instanceof HTMLElement ? document.activeElement : null
    } else {
      popModal(modalId)
    }
    await nextTick()
    if (open) {
      const root = panel.value
      if (!root) return
      // Modal mógł już ustawić fokus w swoim polu (np. login) - nie odbieramy mu go.
      if (root.contains(document.activeElement)) return
      ;(focusableItems(root)[0] ?? root).focus()
      return
    }
    const target = restoreFocus
    restoreFocus = null
    if (target?.isConnected) target.focus()
  },
  { immediate: true },
)

onMounted(() => window.addEventListener('keydown', onKeydown))
onBeforeUnmount(() => {
  window.removeEventListener('keydown', onKeydown)
  // Modal zamknięty przez odmontowanie (np. koniec życia widoku) nie może
  // zostać w stosie i blokować klawiatury kolejnym oknom.
  popModal(modalId)
})
</script>

<template>
  <Transition name="modal">
    <div
      v-if="open"
      class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4"
      @click.self="requestClose"
    >
      <component
        :is="as"
        :ref="setPanel"
        role="dialog"
        aria-modal="true"
        :aria-label="label"
        tabindex="-1"
        class="flex max-h-full w-full flex-col rounded-lg border border-slate-700 bg-slate-900 shadow-2xl outline-none"
        :class="width"
        @submit.prevent="emit('submit')"
      >
        <header class="flex shrink-0 items-center gap-3 border-b border-slate-800 px-4 py-3">
          <slot name="header">
            <h2 class="text-sm font-semibold text-slate-100">
              {{ label }}
            </h2>
          </slot>
          <button
            type="button"
            class="ml-auto flex h-7 w-7 shrink-0 items-center justify-center rounded text-slate-400 transition-colors hover:bg-slate-800 hover:text-slate-100"
            :title="closeLabel"
            @click="emit('close')"
          >
            <X :size="16" />
          </button>
        </header>

        <div
          class="flex min-h-0 flex-1 flex-col overflow-y-auto px-4"
          :class="bodyClass"
        >
          <slot />
        </div>

        <footer class="flex shrink-0 items-center gap-2 border-t border-slate-800 px-4 py-3">
          <slot name="footer" />
        </footer>
      </component>
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

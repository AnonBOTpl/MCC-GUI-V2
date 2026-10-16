<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { Copy, Minus, Settings, Square, X } from '@lucide/vue'
import { useI18n } from 'vue-i18n'

const emit = defineEmits<{ (event: 'open-settings'): void }>()

const { t } = useI18n()
const appWindow = getCurrentWindow()
const maximized = ref(false)
let unlistenResize: (() => void) | undefined

onMounted(async () => {
  maximized.value = await appWindow.isMaximized()
  unlistenResize = await appWindow.onResized(() => {
    void appWindow.isMaximized().then((value) => {
      maximized.value = value
    })
  })
})

onUnmounted(() => {
  unlistenResize?.()
})

async function minimize(): Promise<void> {
  await appWindow.minimize()
}

async function toggleMaximize(): Promise<void> {
  await appWindow.toggleMaximize()
}

async function close(): Promise<void> {
  await appWindow.close()
}
</script>

<template>
  <!--
    Nagłówek jest kontenerem flex z `justify-end`, więc kontener przycisków
    zajmuje tylko swoją szerokość i siedzi po prawej. Bez tego `ml-auto` nic nie
    robi (pozycjonowanie w bloku), kontener rozciąga się na całą szerokość
    i zasłania region przeciągania okna.
  -->
  <header class="relative flex h-10 shrink-0 items-stretch justify-between border-b border-slate-800 bg-slate-900">
    <!-- Osobny region przeciągania; nigdy nie obejmuje przycisków. -->
    <div
      class="absolute inset-0 z-0"
      data-tauri-drag-region
    />

    <!--
      Warstwa z tytułem nie łapie kliknięć (`pointer-events-none`), więc okno
      da się przeciągać także po tekście. Przyciski włączają zdarzenia z powrotem
      przez `pointer-events-auto` i zatrzymują propagację.
    -->
    <div class="pointer-events-none relative z-10 flex h-full items-center gap-1 px-2">
      <button
        type="button"
        class="pointer-events-auto flex h-7 w-7 items-center justify-center rounded text-slate-300 transition-colors hover:bg-slate-800"
        :title="t('settings.open')"
        @click.stop="emit('open-settings')"
      >
        <Settings :size="15" />
      </button>
      <span class="text-sm font-semibold text-slate-200">{{ t('app.title') }}</span>
    </div>

    <div class="relative z-10 flex h-full items-stretch">
      <button
        type="button"
        class="flex w-11 items-center justify-center text-slate-300 transition-colors hover:bg-slate-800"
        :title="t('titleBar.minimize')"
        @click.stop="minimize"
      >
        <Minus :size="16" />
      </button>
      <button
        type="button"
        class="flex w-11 items-center justify-center text-slate-300 transition-colors hover:bg-slate-800"
        :title="maximized ? t('titleBar.restore') : t('titleBar.maximize')"
        @click.stop="toggleMaximize"
      >
        <Copy
          v-if="maximized"
          :size="14"
        />
        <Square
          v-else
          :size="14"
        />
      </button>
      <button
        type="button"
        class="flex w-11 items-center justify-center text-slate-300 transition-colors hover:bg-red-600 hover:text-white"
        :title="t('titleBar.close')"
        @click.stop="close"
      >
        <X :size="16" />
      </button>
    </div>
  </header>
</template>

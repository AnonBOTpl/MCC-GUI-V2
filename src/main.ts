import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'
import { i18n } from './i18n'
import { startDiagnostics } from './log/diag'
import './style.css'

createApp(App).use(createPinia()).use(i18n).mount('#app')

// Diagnostyka zamrożenia widoku logu (opcjonalna, `MCC_GUI_DIAG=1`). Bez niej
// kończy się na jednym pytaniu do Rusta - patrz `src/log/diag.ts`.
void startDiagnostics()

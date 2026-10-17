// Krótki komunikat stanu, który znika sam.
//
// Potrzebny tam, gdzie akcja **nie** została wykonana (np. nie było do kogo
// wysłać wpisu) i trzeba o tym powiedzieć bez wyskakującego błędu: karta błędu
// sugerowałaby awarię, a milczenie zostawiłoby użytkownika bez odpowiedzi.
// Komunikat mieszka przy swoim elemencie sterującym, więc kolejny stan kasuje
// poprzedni, a upływ czasu - ostatni.

import { onUnmounted, ref, type Ref } from 'vue'

/** Ile komunikat zostaje na ekranie. */
export const NOTICE_TIMEOUT_MS = 3000

export interface TransientNotice<T extends string> {
  /** Bieżący komunikat albo `null`, gdy nic nie ma być pokazane. */
  notice: Ref<T | null>
  /** Pokazuje komunikat i (ponownie) odmierza czas do jego schowania. */
  show: (value: T) => void
  /** Chowa komunikat od razu (np. po udanej akcji). */
  clear: () => void
}

export function useTransientNotice<T extends string>(
  timeoutMs: number = NOTICE_TIMEOUT_MS,
): TransientNotice<T> {
  const notice = ref<T | null>(null) as Ref<T | null>
  let timer: ReturnType<typeof setTimeout> | null = null

  function clear(): void {
    if (timer !== null) {
      clearTimeout(timer)
      timer = null
    }
    notice.value = null
  }

  function show(value: T): void {
    clear()
    notice.value = value
    timer = setTimeout(() => {
      timer = null
      notice.value = null
    }, timeoutMs)
  }

  // Komunikat nie może trzymać timera po zamknięciu widoku.
  onUnmounted(clear)

  return { notice, show, clear }
}

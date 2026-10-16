// Wybudzanie odświeżania logu pierwszym zdarzeniem wejścia.
//
// Po co to jest: dla okna zminimalizowanego albo zasłoniętego Chromium potrafi
// wstrzymać produkcję klatek i odłożyć zadania. Dane wtedy dochodzą do bufora,
// ale widok stoi do pierwszej klatki - czyli do pierwszego zdarzenia wejścia.
// Dokładnie to zgłaszał użytkownik: po kilku godzinach sesji `/bal` wykonywało
// się, a podsumowanie z serwera pojawiało się dopiero po ruszeniu kółkiem myszy.
// Zamiast tego odświeżamy widok sami, przy pierwszym takim zdarzeniu.
//
// Page Visibility API tutaj nie pomoże: w tej aplikacji `document.visibilityState`
// zawsze zwraca `visible`, a `visibilitychange` nigdy nie przychodzi
// (tauri-apps/tauri#10592), więc zostają zwykłe zdarzenia wejścia.

/**
 * Zdarzenia, po których warto natychmiast domknąć zaległe odświeżenie widoku.
 * `pointerdown` pokrywa też dotyk, `focus` łapie powrót do okna klikiem,
 * `keydown` - pierwszy klawisz wpisanej komendy, `wheel` - przewijanie.
 */
export const WAKE_EVENTS = ['focus', 'pointerdown', 'wheel', 'keydown'] as const

/**
 * Podłącza nasłuch zdarzeń budzących i zwraca funkcję odpinającą.
 *
 * Nasłuchy są pasywne i w fazie przechwytywania: nie zmieniają zachowania
 * elementów, które obsługują te zdarzenia same (np. przewijania logu), a zdjęcie
 * funkcji odpinającej w tym samym trybie gwarantuje, że nasłuch nie zostaje.
 */
export function registerWakeHandlers(target: EventTarget, handler: () => void): () => void {
  for (const event of WAKE_EVENTS) {
    target.addEventListener(event, handler, { capture: true, passive: true })
  }
  return () => {
    for (const event of WAKE_EVENTS) {
      target.removeEventListener(event, handler, { capture: true })
    }
  }
}

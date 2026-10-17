// Wspólna reguła dla przycisków akcji: dwuklik nie może wykonać drugiej akcji.
//
// Po pierwszym kliknięciu „Start” przycisk zmienia się w „Rozłącz” (panel
// serwerów) albo w „Rozłącz + Połącz ponownie” (nagłówek zakładki). Drugi klik
// dwukliku trafia więc w inną akcję, choć użytkownik chciał tylko uruchomić
// połączenie. `MouseEvent.detail` to licznik kliknięć w tym samym miejscu,
// więc odrzucamy wszystko powyżej pierwszego kliknięcia.

/** Czy zdarzenie kliknięcia jest pierwszym kliknięciem (nie drugim z dwukliku). */
export function isSingleClick(event: MouseEvent): boolean {
  return event.detail <= 1
}

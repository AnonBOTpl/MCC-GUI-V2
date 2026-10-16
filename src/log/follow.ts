// Reguła autoprzewijania logu.
//
// Widok logu tylko wtedy dociąga się do dołu, gdy użytkownik sam nie odjechał od
// niego wyżej. Decyzję podejmujemy przy każdym przewinięciu, ale sama reguła jest
// tutaj - bez zależności od DOM, bo jsdom nie liczy wysokości i nie dałoby się jej
// przetestować w widoku.

/**
 * Wynik decyzji po zdarzeniu przewijania.
 * `true` - włącz śledzenie, `false` - wyłącz, `null` - nic nie zmieniaj.
 */
export function followAfterScroll(following: boolean, atBottom: boolean): boolean | null {
  // Odjechał od dołu - przestajemy go ciągnąć za kursorem.
  if (following && !atBottom) return false
  // Wrócił na dół - wznawiamy sami, bez sięgania do przycisku.
  if (!following && atBottom) return true
  return null
}

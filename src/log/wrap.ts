// Liczenie szerokości tekstu i liczby wierszy po zawinięciu.
//
// Widok logu jest wirtualizowany i pozycjonuje wiersze ręcznie, więc musi znać
// wysokość każdego wiersza **przed** renderem. Ponieważ tekst zawija się w oknie
// (bez poziomego paska), wysokość wiersza zależy od liczby kolumn i szerokości
// znaków. Ten moduł liczy to deterministycznie i jest czysty (bez DOM i Vue),
// dzięki czemu da się w pełni przetestować.

/** Znaki o szerokości podwójnej: CJK, symbole i emoji (przybliżenie East Asian Width). */
const WIDE_CHAR =
  /[\u1100-\u115F\u2E80-\u303E\u3400-\u4DBF\u4E00-\u9FFF\uA000-\uA4CF\uAC00-\uD7A3\uF900-\uFAFF\uFE30-\uFE4F\uFF00-\uFF60\uFFE0-\uFFE6\u{1F000}-\u{1FAFF}]/u

/** Znaki zerowej szerokości: łączące i niewidoczne - nie zajmują kolumny. */
const ZERO_WIDTH = /[\u0300-\u036F\u200B-\u200F\u2060\uFE00-\uFE0F]/

/**
 * Przybliżona szerokość tekstu w kolumnach (znak = 1 kolumna, znak szeroki = 2).
 * Kody formatowania są już usunięte, zanim tekst tu trafi.
 */
export function displayWidth(text: string): number {
  let width = 0
  for (const char of text) {
    if (ZERO_WIDTH.test(char)) continue
    width += WIDE_CHAR.test(char) ? 2 : 1
  }
  return width
}

/**
 * Liczba wierszy wizualnych po zawinięciu tekstu w oknie o zadanej liczbie
 * kolumn. Zakłada zawijanie w dowolnym miejscu (`break-all`), więc dla pisma
 * o stałej szerokości wynik jest dokładny; pusty tekst zajmuje jeden wiersz.
 */
export function wrapRowCount(text: string, columns: number): number {
  if (columns <= 0) return 1
  const width = displayWidth(text)
  if (width === 0) return 1
  return Math.ceil(width / columns)
}

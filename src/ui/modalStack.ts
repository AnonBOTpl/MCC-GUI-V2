// Stos otwartych modali (S-02).
//
// Każdy modal wieszał własny nasłuch `keydown` na `window`, więc jeden `Esc`
// (albo `Tab`) obsługiwały wszystkie otwarte naraz: anulowanie pytania
// o zamknięcie zamykało też ustawienia pod nim, a dwie pułapki fokusu walczyły
// ze sobą. Tutaj trzymamy kolejność otwarcia - klawiaturę obsługuje wyłącznie
// modal z wierzchu stosu.

const stack: symbol[] = []

/** Dodaje modal na wierzch stosu (ponowne otwarcie przenosi go na wierzch). */
export function pushModal(id: symbol): void {
  popModal(id)
  stack.push(id)
}

/** Usuwa modal ze stosu (zamknięcie albo odmontowanie). */
export function popModal(id: symbol): void {
  const index = stack.lastIndexOf(id)
  if (index !== -1) stack.splice(index, 1)
}

/** Czy ten modal jest na wierzchu, czyli czy należy do niego klawiatura. */
export function isTopModal(id: symbol): boolean {
  return stack[stack.length - 1] === id
}

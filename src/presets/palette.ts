// Mapowanie nazw kolorów presetów na klasy Tailwind.
//
// Nazwy (a nie wartości HEX) są kontraktem z Rustem (`preset::PRESET_COLORS`),
// więc paleta żyje dokładnie w jednym miejscu: tutaj tłumaczymy nazwę na klasy,
// a w Rustcie nazwa jest walidowana przy zapisie.

import { PRESET_COLORS, type PresetColor } from '../api/contract'

/** Kolor paska przy etykiecie presetu (ten sam dla plakietki w edytorze). */
const COLOR_CLASSES: Record<PresetColor, string> = {
  blue: 'bg-blue-500',
  green: 'bg-green-500',
  amber: 'bg-amber-500',
  red: 'bg-red-500',
  violet: 'bg-violet-500',
}

/** Lista kolorów do wyboru w edytorze (bez pustego „brak koloru”). */
export const PRESET_COLOR_OPTIONS: readonly PresetColor[] = PRESET_COLORS

/**
 * Sprowadza dowolną wartość do koloru z palety.
 *
 * `presets.json` można edytować ręcznie, a wczytanie jest łagodne (`store.rs`),
 * więc nieznany kolor nie może wywalić widoku - po prostu nie ma wtedy paska.
 */
export function asPresetColor(value: unknown): PresetColor | null {
  if (typeof value !== 'string') return null
  const found = PRESET_COLORS.find((color) => color === value)
  return found ?? null
}

/** Klasy paska koloru; brak koloru to pusty łańcuch (paska po prostu nie ma). */
export function presetColorClass(color: PresetColor | null): string {
  return color ? COLOR_CLASSES[color] : ''
}

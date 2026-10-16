import { createI18n, type PluralizationRule } from 'vue-i18n'
import en from './locales/en.json'
import pl from './locales/pl.json'

export const SUPPORTED_LOCALES = ['en', 'pl'] as const
export type Locale = (typeof SUPPORTED_LOCALES)[number]

export const DEFAULT_LOCALE: Locale = 'en'

/**
 * Reguła liczby mnogiej dla polskiego (trzy formy: one / few / many).
 *
 * Domyślna reguła vue-i18n rozróżnia tylko „1” i „resztę”, więc trzeci wariant
 * z `pl.json` nigdy nie był używany - „1 połączenie” i „5 połączeń” brały tę samą
 * formę („połączenia”). Reguła wg CLDR:
 *   - `one`  - dokładnie 1,
 *   - `few`  - końcówka 2..4, ale nie 12..14 (2, 3, 4, 22, 23, 24, ale nie 12),
 *   - `many` - cała reszta (0, 5..21, 25..31, 100..).
 */
export const polishPluralRule: PluralizationRule = (choice, choicesLength) => {
  const n = Math.abs(choice)

  // Gdy wiadomość ma mniej niż trzy formy (np. tekst z fallbacku), nie ma czego
  // wybierać - zachowujemy się jak dla angielskiego.
  if (choicesLength < 3) return n === 1 ? 0 : 1

  if (n === 1) return 0

  const last = n % 10
  const lastTwo = n % 100
  if (last >= 2 && last <= 4 && (lastTwo < 12 || lastTwo > 14)) return 1

  return 2
}

/** Reguły liczb mnogich per język. `en` (dwie formy) obsługuje domyślna reguła. */
export const PLURAL_RULES = { pl: polishPluralRule }

// Aplikacja zawsze startuje po angielsku; `en` jest źródłem prawdy i fallbackiem.
export const i18n = createI18n({
  legacy: false,
  locale: DEFAULT_LOCALE,
  fallbackLocale: DEFAULT_LOCALE,
  pluralRules: PLURAL_RULES,
  messages: { en, pl },
})

#!/usr/bin/env node
// Sprawdza kompletność i poprawność tłumaczeń.
//
//   1. `en` i `pl` mają dokładnie ten sam zestaw kluczy (`en` to źródło prawdy).
//   2. Klucz jest neutralny: segmenty pisane od małej litery, oddzielone kropką.
//   3. Komunikat w liczbie mnogiej (formy rozdzielone `|`) ma tyle form, ile wymaga
//      reguła liczby mnogiej danego języka (`Intl.PluralRules`) - dla `en` to
//      one/other, dla `pl` one/few/many.
//   4. Tekst nie zawiera znaków, które vue-i18n traktuje jako składnię wiadomości
//      (`@` bez escapowania, nieparzyste nawiasy). Taki tekst wywala kompilację
//      całego słownika i aplikacja renderuje puste okno - taniej złapać to tutaj.
//
// Sprawdzanie użycia kluczy w komponentach i wykrywanie twardego tekstu to
// osobny krok (Etap 7) - tutaj pilnujemy samych słowników.

import { readFileSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

const here = dirname(fileURLToPath(import.meta.url))
const localesDir = join(here, '..', 'src', 'i18n', 'locales')

const LOCALES = ['en', 'pl']

const readLocale = (name) => JSON.parse(readFileSync(join(localesDir, `${name}.json`), 'utf8'))

/** Zamienia zagnieżdżony obiekt na mapę `klucz.kropkowany -> wartość` (liście tylko teksty). */
function flatten(value, prefix = '', out = new Map()) {
  if (typeof value === 'string') {
    out.set(prefix, value)
    return out
  }
  if (typeof value !== 'object' || value === null || Array.isArray(value)) {
    out.set(prefix, value)
    return out
  }
  for (const [key, child] of Object.entries(value)) {
    flatten(child, prefix ? `${prefix}.${key}` : key, out)
  }
  return out
}

// Segment klucza: od małej litery, dalej litery i cyfry (dozwolony camelCase).
const KEY_SEGMENT = /^[a-z][a-zA-Z0-9]*$/

// Liczba form liczby mnogiej wymagana przez język. Dla polskiego przyjmujemy
// one/few/many (3), a nie `other` z `Intl.PluralRules` (który dotyczy
// ułamków). Dla `en` reguła to one/other (2). Dla innych języków bierzemy liczbę
// kategorii wprost z `Intl.PluralRules`.
const PLURAL_FORMS = { en: 2, pl: 3 }

/** Liczba form liczby mnogiej wymagana przez język. */
function pluralFormCount(locale) {
  return PLURAL_FORMS[locale] ?? new Intl.PluralRules(locale).resolvedOptions().pluralCategories.length
}

/**
 * Sprawdza znaki specjalne vue-i18n. `@` (np. w adresie e-mail) musi być zapisany
 * jako `{'@'}`, a nawiasy klamrowe muszą się bilansować w obrębie wiadomości.
 */
function reservedCharacterIssues(key, value) {
  const problems = []

  // `@` jest znakiem składni (linked messages) - literał wymaga formy `{'@'}`.
  const escapedAt = value.split("{'@'}").join('')
  if (escapedAt.includes('@')) {
    problems.push(`zawiera \`@\` - użyj \`{'@'}\` (np. w adresie e-mail)`)
  }

  // Każdy `{` musi mieć parę `}` i odwrotnie (pomijamy poprawnie escapowane `{'@'}`).
  let depth = 0
  for (const char of escapedAt) {
    if (char === '{') depth += 1
    else if (char === '}') depth -= 1
    if (depth < 0) {
      problems.push('ma zamknięcie `}` bez otwarcia `{`')
      break
    }
  }
  if (depth > 0) {
    problems.push('ma otwarcie `{` bez zamknięcia `}`')
  }

  if (problems.length > 0) {
    return `[${key}] ${problems.join('; ')}`
  }
  return null
}

const errors = []
const byLocale = new Map()

for (const locale of LOCALES) {
  const entries = flatten(readLocale(locale))
  byLocale.set(locale, entries)

  for (const [key, value] of entries) {
    if (!key.split('.').every((segment) => KEY_SEGMENT.test(segment))) {
      errors.push(`[${locale}] niepoprawny klucz: "${key}" (segmenty od małej litery, kropki)`)
    }
    if (typeof value !== 'string') {
      errors.push(`[${locale}] "${key}" nie jest tekstem (${typeof value})`)
      continue
    }
    const reserved = reservedCharacterIssues(key, value)
    if (reserved) errors.push(`[${locale}] ${reserved}`)

    if (value.includes('|')) {
      const forms = value.split('|').length
      const required = pluralFormCount(locale)
      if (forms !== required) {
        errors.push(
          `[${locale}] "${key}" ma ${forms} form liczby mnogiej, a język wymaga ${required} ` +
            `(kategorie: ${new Intl.PluralRules(locale).resolvedOptions().pluralCategories.join('/')})`,
        )
      }
    }
  }
}

// Zgodność zestawów kluczy między językami.
const enKeys = new Set(byLocale.get('en').keys())
for (const locale of LOCALES) {
  if (locale === 'en') continue
  const keys = new Set(byLocale.get(locale).keys())
  for (const key of enKeys) {
    if (!keys.has(key)) errors.push(`[${locale}] brakuje klucza: "${key}"`)
  }
  for (const key of keys) {
    if (!enKeys.has(key)) errors.push(`[${locale}] zbędny klucz (nie ma go w en): "${key}"`)
  }
}

const totalKeys = enKeys.size
if (errors.length > 0) {
  console.error(`check:i18n failed — ${errors.length} problem(ów):\n  ${errors.join('\n  ')}`)
  process.exit(1)
}

console.log(`check:i18n OK — ${totalKeys} kluczy w synchronizacji (${LOCALES.join(', ')}).`)

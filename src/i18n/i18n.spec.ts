import { describe, expect, it } from 'vitest'
import { createI18n } from 'vue-i18n'
import en from './locales/en.json'
import pl from './locales/pl.json'
import { DEFAULT_LOCALE, PLURAL_RULES, SUPPORTED_LOCALES, polishPluralRule } from './index'

/**
 * Tłumacz na jeden język - potrzebny, żeby wymusić locale w teście.
 * Podajemy **te same** reguły liczb mnogich co aplikacja (`PLURAL_RULES`), inaczej
 * test sprawdzałby coś innego, niż działa w UI.
 */
function translator(locale: 'en' | 'pl') {
  return createI18n({
    legacy: false,
    locale,
    fallbackLocale: 'en',
    pluralRules: PLURAL_RULES,
    messages: { en, pl },
  }).global
}

function flattenKeys(value: unknown, prefix = ''): string[] {
  if (typeof value !== 'object' || value === null) {
    return [prefix]
  }
  return Object.entries(value as Record<string, unknown>).flatMap(([key, child]) =>
    flattenKeys(child, prefix ? `${prefix}.${key}` : key),
  )
}

describe('i18n', () => {
  it('starts in English by default', () => {
    expect(DEFAULT_LOCALE).toBe('en')
  })

  it('ships English and Polish', () => {
    expect([...SUPPORTED_LOCALES]).toEqual(['en', 'pl'])
  })

  it('keeps Polish and English key sets identical', () => {
    expect(flattenKeys(pl).sort()).toEqual(flattenKeys(en).sort())
  })

  it('odmienia polską liczbę mnogą we wszystkich trzech formach', () => {
    const t = translator('pl')

    // `check:i18n` pilnuje tylko zestawu kluczy - formy liczby mnogiej trzeba
    // sprawdzić osobno, bo łatwo je skleić w jedną.
    expect(t.t('sidebar.activeAccounts', 1)).toBe('1 aktywne połączenie')
    expect(t.t('sidebar.activeAccounts', 3)).toBe('3 aktywne połączenia')
    expect(t.t('sidebar.activeAccounts', 5)).toBe('5 aktywnych połączeń')
  })

  it('angielska liczba mnoga ma dwie formy', () => {
    const t = translator('en')

    expect(t.t('sidebar.activeAccounts', 1)).toBe('1 active account')
    expect(t.t('sidebar.activeAccounts', 3)).toBe('3 active accounts')
  })

  it('reguła polskiej liczby mnogiej trafia w progi CLDR', () => {
    // Wartości brzegowe: jedynka, dwójka, dwunastka, dwudziestka pierwsza.
    expect(polishPluralRule(1, 3)).toBe(0) // one
    expect(polishPluralRule(2, 3)).toBe(1) // few
    expect(polishPluralRule(12, 3)).toBe(2) // many - nastki to nie „few”
    expect(polishPluralRule(14, 3)).toBe(2)
    expect(polishPluralRule(22, 3)).toBe(1) // few - końcówka 2..4
    expect(polishPluralRule(21, 3)).toBe(2) // many
    expect(polishPluralRule(0, 3)).toBe(2) // many
    // Mniej niż trzy formy (np. tekst z fallbacku) - zachowanie jak angielskie.
    expect(polishPluralRule(1, 2)).toBe(0)
    expect(polishPluralRule(5, 2)).toBe(1)
  })

  it('odmienia liczbę pominiętych wpisów w pliku danych (R-16)', () => {
    const t = translator('pl')

    // Pierwszy komunikat błędu z liczbą mnogą - formy łatwo skleić w jedną.
    expect(t.t('errors.store.entriesSkipped', { count: 1, file: 'accounts.json' })).toContain(
      'Pominięto 1 wpis w pliku accounts.json',
    )
    expect(t.t('errors.store.entriesSkipped', { count: 3, file: 'accounts.json' })).toContain(
      'Pominięto 3 wpisy',
    )
    expect(t.t('errors.store.entriesSkipped', { count: 5, file: 'accounts.json' })).toContain(
      'Pominięto 5 wpisów',
    )
    expect(translator('en').t('errors.store.entriesSkipped', { count: 1, file: 'accounts.json' })).toContain(
      'Skipped 1 entry',
    )
  })

  it('ma własne teksty dla kodów plików danych (R-10)', () => {
    const codes = ['fileUnreadable', 'fileQuarantined', 'entriesSkipped']

    for (const locale of ['en', 'pl'] as const) {
      const t = translator(locale)
      const generic = t.t('errors.app.unexpectedError')
      for (const code of codes) {
        const text = t.t(`errors.store.${code}`, { file: 'accounts.json', count: 2 })
        expect(text, `${locale}.${code}`).not.toBe(generic)
        expect(text, `${locale}.${code}`).toContain('accounts.json')
      }
    }
  })

  it('wypisuje @ w podpowiedzi loginu, a nie znacznik escapingu', () => {
    // `@` ma w vue-i18n znaczenie specjalne, więc w plikach jest jako `{'@'}`.
    expect(translator('en').t('addAccount.loginPlaceholder')).toBe('you@example.com')
    expect(translator('pl').t('addAccount.loginPlaceholder')).toBe('ty@example.com')
  })
})

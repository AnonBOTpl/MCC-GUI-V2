import { describe, expect, it } from 'vitest'
import { readdirSync, readFileSync } from 'node:fs'
import { join } from 'node:path'

/**
 * Statyczne strażniki poprawek dostępności z audytu: R-30, R-42, R-48.
 *
 * Część z tych poprawek żyje w klasach Tailwinda i w globalnym CSS, których jsdom
 * nie wylicza (`getComputedStyle` nie zna arkusza Tailwinda), więc testy czytają
 * źródła: sprawdzają, że reguły globalne istnieją i że w komponentach nie wróciły
 * klasy o zbyt niskim kontraście. Zachowanie na żywo (obwódka fokusu, brak animacji
 * przy `prefers-reduced-motion`) było dodatkowo sprawdzone w działającej aplikacji.
 */

function walk(dir: string): string[] {
  const files: string[] = []
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const path = join(dir, entry.name)
    if (entry.isDirectory()) files.push(...walk(path))
    else if (path.endsWith('.vue')) files.push(path)
  }
  return files
}

const components = walk('src').map((path) => ({ path, text: readFileSync(path, 'utf8') }))
const styleSheet = readFileSync('src/style.css', 'utf8')

/** Wartości palety Tailwind v4 używane w klasach; brak koloru = pomijamy. */
const PALETTE: Record<string, string> = {
  'blue-600': '#2563eb',
  'blue-700': '#1d4ed8',
  'blue-800': '#1e40af',
  'sky-700': '#0369a1',
  'sky-800': '#075985',
  'red-600': '#dc2626',
  'red-700': '#b91c1c',
  'red-900': '#7f1d1d',
  'slate-700': '#334155',
  'slate-800': '#1e293b',
}

function luminance(hex: string): number {
  const channel = (offset: number): number => {
    const value = Number.parseInt(hex.slice(offset, offset + 2), 16) / 255
    return value <= 0.04045 ? value / 12.92 : ((value + 0.055) / 1.055) ** 2.4
  }
  return 0.2126 * channel(1) + 0.7152 * channel(3) + 0.0722 * channel(5)
}

function contrast(foreground: string, background: string): number {
  const a = luminance(foreground)
  const b = luminance(background)
  return (Math.max(a, b) + 0.05) / (Math.min(a, b) + 0.05)
}

describe('styl globalny', () => {
  it('wyłącza przejścia przy prefers-reduced-motion (R-48)', () => {
    expect(styleSheet).toMatch(/@media \(prefers-reduced-motion: reduce\)\s*\{[^@]*transition-duration/)
  })
})

describe('kontrast w komponentach', () => {
  it('teksty pomocnicze nie schodzą do slate-500/600 (R-42)', () => {
    const offenders = components
      .filter(({ text }) => /(?<!placeholder:)text-slate-(500|600)/.test(text))
      .map(({ path }) => path)
    expect(offenders).toEqual([])
  })

  it('tło pod białym tekstem ma co najmniej 4,5:1 (R-30)', () => {
    const failures: string[] = []
    for (const { path, text } of components) {
      // Każdy atrybut klasy osobno: oceniamy tylko te, które faktycznie ustawiają
      // biały tekst (przyciski i akcje), a nie kropki statusu bez tekstu.
      for (const match of text.matchAll(/(?:class|:class)="([^"]*)"/g)) {
        const classes = match[1]
        if (!/(^|[\s'"])text-white/.test(classes)) continue
        for (const bg of classes.matchAll(/(?:^|[\s'"])(?:[\w-]+:)*bg-([a-z]+-\d+)(?![\w/])/g)) {
          const hex = PALETTE[bg[1]]
          if (!hex) continue
          if (contrast('#ffffff', hex) < 4.5) failures.push(`${path}: bg-${bg[1]}`)
        }
      }
    }
    expect(failures).toEqual([])
  })
})

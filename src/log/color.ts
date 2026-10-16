// Parser kolorów Minecrafta (§) oraz sekwencji ANSI w jednej linii logu.
//
// Wzorce kolorów i kodów formatowania pochodzą z kodu MCC
// (`reference/Minecraft-Console-Client/MinecraftClient/Tui/McColorParser.cs`)
// oraz z realnego wyjścia MCC. Obsługujemy oba formaty, bo MCC
// w trybie `BasicIO` wysyła `§`, ale część serwerów wstawia własne sekwencje ANSI.
//
// Moduł jest czysty (bez DOM i bez Vue), więc daje się w pełni przetestować.

/** Standardowa paleta Minecrafta - te same wartości RGB co w MCC. */
export const MINECRAFT_COLORS: Record<string, string> = {
  '0': '#000000',
  '1': '#0000aa',
  '2': '#00aa00',
  '3': '#00aaaa',
  '4': '#aa0000',
  '5': '#aa00aa',
  '6': '#ffaa00',
  '7': '#aaaaaa',
  '8': '#555555',
  '9': '#5555ff',
  a: '#55ff55',
  b: '#55ffff',
  c: '#ff5555',
  d: '#ff55ff',
  e: '#ffff55',
  f: '#ffffff',
}

/** Domyślny kolor tekstu na ciemnym tle (odpowiednik `§f`). */
export const DEFAULT_FOREGROUND = '#e2e8f0'

/** Jeden fragment linii o jednolitym formatowaniu. */
export interface TextSegment {
  text: string
  color: string
  bold: boolean
  italic: boolean
  underline: boolean
  strikethrough: boolean
}

/** Stan formatowania przenoszony między fragmentami i liniami. */
export interface FormatState {
  color: string
  bold: boolean
  italic: boolean
  underline: boolean
  strikethrough: boolean
}

/** Stan początkowy: biały (a właściwie czytelny na ciemnym tle), bez stylów. */
export function initialFormat(): FormatState {
  return {
    color: DEFAULT_FOREGROUND,
    bold: false,
    italic: false,
    underline: false,
    strikethrough: false,
  }
}

function resetStyles(state: FormatState): void {
  state.bold = false
  state.italic = false
  state.underline = false
  state.strikethrough = false
}

/** Klasyczne ANSI 30-37 / 90-97 (paleta 4-bit przybliżona do kolorów Minecrafta). */
const ANSI_BASIC_COLORS: Record<number, string> = {
  30: '#000000',
  31: '#aa0000',
  32: '#00aa00',
  33: '#ffaa00',
  34: '#0000aa',
  35: '#aa00aa',
  36: '#00aaaa',
  37: '#aaaaaa',
  90: '#555555',
  91: '#ff5555',
  92: '#55ff55',
  93: '#ffff55',
  94: '#5555ff',
  95: '#ff55ff',
  96: '#55ffff',
  97: '#ffffff',
}

/** ANSI 40-47 / 100-107 - tła. Renderujemy tylko kolor tekstu, ale musimy je znać. */
const ANSI_BACKGROUND_CODES = new Set([
  40, 41, 42, 43, 44, 45, 46, 47, 100, 101, 102, 103, 104, 105, 106, 107,
])

function toHex(value: number): string {
  return Math.max(0, Math.min(255, Math.round(value))).toString(16).padStart(2, '0')
}

/** Zamienia indeks palety 256 kolorów ANSI na zapis `#rrggbb`. */
function ansi256ToHex(index: number): string {
  if (index < 8) {
    return ANSI_BASIC_COLORS[30 + index] ?? DEFAULT_FOREGROUND
  }
  if (index < 16) {
    return ANSI_BASIC_COLORS[90 + (index - 8)] ?? DEFAULT_FOREGROUND
  }
  if (index < 232) {
    const cube = index - 16
    const steps = [0, 95, 135, 175, 215, 255]
    const r = steps[Math.floor(cube / 36) % 6]
    const g = steps[Math.floor(cube / 6) % 6]
    const b = steps[cube % 6]
    return `#${toHex(r)}${toHex(g)}${toHex(b)}`
  }
  const gray = 8 + (index - 232) * 10
  return `#${toHex(gray)}${toHex(gray)}${toHex(gray)}`
}

/** Aplikuje jeden zestaw parametrów SGR do stanu formatowania. */
function applySgr(state: FormatState, raw: string): void {
  const params = raw === '' ? [0] : raw.split(';').map((part) => Number.parseInt(part, 10) || 0)

  for (let i = 0; i < params.length; i += 1) {
    const code = params[i]
    if (code === 0) {
      state.color = DEFAULT_FOREGROUND
      resetStyles(state)
    } else if (code === 1) {
      state.bold = true
    } else if (code === 3) {
      state.italic = true
    } else if (code === 4) {
      state.underline = true
    } else if (code === 9) {
      state.strikethrough = true
    } else if (code === 21 || code === 22) {
      state.bold = false
    } else if (code === 23) {
      state.italic = false
    } else if (code === 24) {
      state.underline = false
    } else if (code === 29) {
      state.strikethrough = false
    } else if (code === 39) {
      state.color = DEFAULT_FOREGROUND
    } else if (ANSI_BASIC_COLORS[code]) {
      state.color = ANSI_BASIC_COLORS[code]
    } else if (ANSI_BACKGROUND_CODES.has(code)) {
      // Tła ignorujemy - log ma własne tło, a kolory tekstu są czytelniejsze.
    } else if (code === 38 && params[i + 1] === 5) {
      state.color = ansi256ToHex(params[i + 2] ?? 7)
      i += 2
    } else if (code === 38 && params[i + 1] === 2) {
      const r = params[i + 2] ?? 255
      const g = params[i + 3] ?? 255
      const b = params[i + 4] ?? 255
      state.color = `#${toHex(r)}${toHex(g)}${toHex(b)}`
      i += 4
    }
  }
}

/**
 * Usuwa sekwencje ANSI (kolory i inne CSI) z tekstu, zostawiając samą treść.
 * Przydatne np. do dopasowywania wzorców (Etap 3) i do porównań w testach.
 */
export function stripAnsi(text: string): string {
  return text
    .replace(/\u001b\[[0-9;]*m/g, '')
    .replace(/\u001b\[[0-9;?]*[A-Za-z]/g, '')
    .replace(/\u001b\][^\u0007]*\u0007/g, '')
}

/** Usuwa kody `§` (w tym `§#rrggbb`), zostawiając samą treść. */
export function stripSectionCodes(text: string): string {
  let out = ''
  for (let i = 0; i < text.length; i += 1) {
    if (text[i] !== '§') {
      out += text[i]
      continue
    }
    if (text[i + 1] === '#' && i + 7 < text.length) {
      i += 7
    } else {
      i += 1
    }
  }
  return out
}

/** Usuwa kody `§` i sekwencje ANSI - pełny tekst bez formatowania. */
export function stripFormatting(text: string): string {
  return stripSectionCodes(stripAnsi(text))
}

function isHexDigit(char: string | undefined): boolean {
  return char !== undefined && /[0-9a-fA-F]/.test(char)
}

/**
 * Rozbija linię na fragmenty o jednolitym formatowaniu.
 *
 * `initial` pozwala przenieść formatowanie z poprzedniej linii (MCC czasem
 * rozbija jeden komunikat na kilka linii bez resetu koloru).
 */
export function parseSegments(line: string, initial?: FormatState): TextSegment[] {
  const state: FormatState = initial ? { ...initial } : initialFormat()
  const segments: TextSegment[] = []
  let buffer = ''

  const flush = (): void => {
    if (buffer === '') return
    segments.push({ text: buffer, ...state })
    buffer = ''
  }

  for (let i = 0; i < line.length; i += 1) {
    const char = line[i]

    // Sekwencja ANSI: ESC [ ... m
    if (char === '\u001b') {
      if (line[i + 1] === '[') {
        const match = /^\[([0-9;]*)m/.exec(line.slice(i + 1))
        if (match) {
          flush()
          applySgr(state, match[1])
          i += match[0].length
          continue
        }
      }
      // Nieznana sekwencja ESC - pomijamy sam znak sterujący.
      flush()
      continue
    }

    if (char !== '§') {
      buffer += char
      continue
    }

    // `§#rrggbb` - kolor w zapisie szesnastkowym (Minecraft 1.16+).
    if (line[i + 1] === '#') {
      const hex = line.slice(i + 2, i + 8)
      if (hex.length === 6 && [...hex].every(isHexDigit)) {
        flush()
        state.color = `#${hex.toLowerCase()}`
        resetStyles(state)
        i += 7
        continue
      }
    }

    const code = line[i + 1]?.toLowerCase()
    if (code === undefined) break

    flush()
    if (MINECRAFT_COLORS[code]) {
      state.color = MINECRAFT_COLORS[code]
      resetStyles(state)
    } else if (code === 'l') {
      state.bold = true
    } else if (code === 'o') {
      state.italic = true
    } else if (code === 'n') {
      state.underline = true
    } else if (code === 'm') {
      state.strikethrough = true
    } else if (code === 'r') {
      state.color = DEFAULT_FOREGROUND
      resetStyles(state)
    }
    i += 1
  }

  flush()

  if (segments.length === 0) {
    segments.push({ text: '', ...state })
  }
  return segments
}

/** Stan formatowania po przetworzeniu linii (do przekazania w kolejną linię). */
export function formatStateAfter(line: string, initial?: FormatState): FormatState {
  const segments = parseSegments(line, initial)
  const last = segments[segments.length - 1]
  return {
    color: last.color,
    bold: last.bold,
    italic: last.italic,
    underline: last.underline,
    strikethrough: last.strikethrough,
  }
}

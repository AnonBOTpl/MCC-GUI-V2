import { describe, expect, it } from 'vitest'
import {
  DEFAULT_FOREGROUND,
  formatStateAfter,
  initialFormat,
  parseSegments,
  stripAnsi,
  stripFormatting,
  stripSectionCodes,
} from './color'

describe('parser kolorów §', () => {
  it('zwraca tekst bez kodów jako jeden segment', () => {
    const segments = parseSegments('Hello world')
    expect(segments).toHaveLength(1)
    expect(segments[0].text).toBe('Hello world')
    expect(segments[0].color).toBe(DEFAULT_FOREGROUND)
  })

  it('rozdziela kolory i zachowuje treść bez kodów', () => {
    const segments = parseSegments('§8[MCC] §aOK §ftext')
    expect(segments.map((segment) => segment.text)).toEqual(['[MCC] ', 'OK ', 'text'])
    expect(segments.map((segment) => segment.color)).toEqual(['#555555', '#55ff55', '#ffffff'])
  })

  it('obsługuje kody formatowania (pogrubienie, kursywa, podkreślenie, przekreślenie)', () => {
    const [segment] = parseSegments('§l§o§n§mX')
    expect(segment.text).toBe('X')
    expect(segment.bold).toBe(true)
    expect(segment.italic).toBe(true)
    expect(segment.underline).toBe(true)
    expect(segment.strikethrough).toBe(true)
  })

  it('kolor resetuje style, tak jak w MCC (McColorParser)', () => {
    const bold = parseSegments('§lBold')[0]
    expect(bold.bold).toBe(true)

    // Nowy kod koloru zdejmuje pogrubienie - zachowanie zgodne z MCC.
    const colored = parseSegments('§l§cRed')[0]
    expect(colored.bold).toBe(false)
    expect(colored.color).toBe('#ff5555')

    const reset = parseSegments('§l§rNormal')[0]
    expect(reset.bold).toBe(false)
    expect(reset.color).toBe(DEFAULT_FOREGROUND)
  })

  it('obsługuje kolor szesnastkowy §#rrggbb', () => {
    const [segment] = parseSegments('§#12AB34Kolor')
    expect(segment.text).toBe('Kolor')
    expect(segment.color).toBe('#12ab34')
  })

  it('nie zjada treści przy niepoprawnym kolorze szesnastkowym', () => {
    const segments = parseSegments('§#zzzzzzText')
    expect(stripFormatting('§#zzzzzzText')).toBe('Text')
    expect(segments.every((segment) => segment.text !== '')).toBe(true)
  })

  it('przenosi formatowanie między liniami, gdy brak resetu', () => {
    const after = formatStateAfter('§cFirst')
    expect(after.color).toBe('#ff5555')

    const [second] = parseSegments('Second', after)
    expect(second.color).toBe('#ff5555')
  })
})

describe('parser sekwencji ANSI', () => {
  it('obsługuje kolory 4-bitowe', () => {
    const [segment] = parseSegments('\u001b[32mGreen')
    expect(segment.text).toBe('Green')
    expect(segment.color).toBe('#00aa00')
  })

  it('obsługuje paletę 256 kolorów i truecolor', () => {
    expect(parseSegments('\u001b[38;5;196mX')[0].color).toBe('#ff0000')
    expect(parseSegments('\u001b[38;2;18;52;86mX')[0].color).toBe('#123456')
  })

  it('obsługuje pogrubienie i reset', () => {
    const bold = parseSegments('\u001b[1mBold')[0]
    expect(bold.bold).toBe(true)
    const reset = parseSegments('\u001b[1;31mX')[0]
    expect(reset.bold).toBe(true)
    expect(reset.color).toBe('#aa0000')
    expect(parseSegments('\u001b[0mPlain')[0].bold).toBe(false)
  })

  it('łączy ANSI z kodami § w jednej linii', () => {
    const segments = parseSegments('\u001b[31m§aMixed')
    expect(segments.map((segment) => segment.text)).toEqual(['Mixed'])
    expect(segments[0].color).toBe('#55ff55')
  })
})

describe('usuwanie formatowania', () => {
  it('stripSectionCodes usuwa §x i §#rrggbb', () => {
    expect(stripSectionCodes('§8[MCC] §#ff0000Red §aOK')).toBe('[MCC] Red OK')
  })

  it('stripAnsi usuwa sekwencje kolorów i inne CSI', () => {
    expect(stripAnsi('\u001b[31mRed\u001b[0m \u001b[2Kdone')).toBe('Red done')
  })

  it('stripFormatting usuwa jedno i drugie', () => {
    expect(stripFormatting('\u001b[31m§8[MCC] \u001b[0m§aOK')).toBe('[MCC] OK')
  })
})

describe('przypadki brzegowe', () => {
  it('radzi sobie z pustą linią', () => {
    const segments = parseSegments('')
    expect(segments).toHaveLength(1)
    expect(segments[0].text).toBe('')
    expect(segments[0].color).toBe(DEFAULT_FOREGROUND)
  })

  it('nie wywraca się na kodzie § na końcu linii', () => {
    expect(() => parseSegments('Text §')).not.toThrow()
    expect(parseSegments('Text §')[0].text).toBe('Text ')
  })

  it('initialFormat zwraca czytelny kolor na ciemnym tle', () => {
    expect(initialFormat().color).toBe(DEFAULT_FOREGROUND)
  })
})

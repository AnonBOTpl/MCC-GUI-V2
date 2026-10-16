import { describe, expect, it } from 'vitest'
import { PRESET_COLORS } from '../api/contract'
import { asPresetColor, presetColorClass, PRESET_COLOR_OPTIONS } from './palette'

describe('asPresetColor', () => {
  it('przyjmuje każdą nazwę z palety', () => {
    for (const color of PRESET_COLORS) {
      expect(asPresetColor(color)).toBe(color)
    }
  })

  it('odrzuca nieznane wartości i inne typy', () => {
    expect(asPresetColor('magenta')).toBeNull()
    expect(asPresetColor('#3b82f6')).toBeNull()
    expect(asPresetColor(null)).toBeNull()
    expect(asPresetColor(3)).toBeNull()
  })
})

describe('presetColorClass', () => {
  it('daje klasę tła dla koloru z palety', () => {
    expect(presetColorClass('blue')).toBe('bg-blue-500')
    expect(presetColorClass('violet')).toBe('bg-violet-500')
  })

  it('bez koloru nie daje żadnej klasy', () => {
    expect(presetColorClass(null)).toBe('')
  })
})

describe('PRESET_COLOR_OPTIONS', () => {
  it('pokazuje wszystkie kolory palety', () => {
    expect(PRESET_COLOR_OPTIONS).toEqual([...PRESET_COLORS])
  })
})

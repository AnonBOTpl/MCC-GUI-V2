import { describe, expect, it } from 'vitest'
import { isSingleClick } from './click'

function click(detail: number): MouseEvent {
  return new MouseEvent('click', { detail })
}

describe('isSingleClick', () => {
  it('przepuszcza pojedyncze kliknięcie (także syntetyczne z detail = 0)', () => {
    expect(isSingleClick(click(0))).toBe(true)
    expect(isSingleClick(click(1))).toBe(true)
  })

  it('odrzuca drugi i kolejne klik dwukliku', () => {
    expect(isSingleClick(click(2))).toBe(false)
    expect(isSingleClick(click(3))).toBe(false)
  })
})

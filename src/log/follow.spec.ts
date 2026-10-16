import { describe, expect, it } from 'vitest'
import { followAfterScroll } from './follow'

describe('followAfterScroll', () => {
  it('wyłącza śledzenie, gdy użytkownik odjedzie od dołu', () => {
    expect(followAfterScroll(true, false)).toBe(false)
  })

  it('wznawia śledzenie po powrocie na dół', () => {
    expect(followAfterScroll(false, true)).toBe(true)
  })

  it('nie zmienia niczego, gdy stan zgadza się z pozycją', () => {
    expect(followAfterScroll(true, true)).toBeNull()
    expect(followAfterScroll(false, false)).toBeNull()
  })
})

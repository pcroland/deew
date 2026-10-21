import { describe, expect, it } from "vitest"

import { fittingFontSize } from "@/lib/fitting-font-size"

describe("fittingFontSize", () => {
  it("grows back to the max when there is room", () => {
    expect(fittingFontSize(13, 120)).toBe(13)
    expect(fittingFontSize(8, 200)).toBe(13)
  })
  it("keeps 13 when exactly 60 columns fit", () => {
    expect(fittingFontSize(13, 60)).toBe(13)
  })
  it("shrinks proportionally, in 0.5px steps rounded down", () => {
    expect(fittingFontSize(13, 40)).toBe(8.5) // 8.67 -> 8.5
    expect(fittingFontSize(13, 30)).toBe(6.5)
  })
  it("handles a larger-than-needed shrunken font", () => {
    expect(fittingFontSize(6.5, 60)).toBe(6.5)
    expect(fittingFontSize(6.5, 90)).toBe(9.5)
  })
  it("ignores bad input", () => {
    expect(fittingFontSize(13, 0)).toBe(13)
  })
})

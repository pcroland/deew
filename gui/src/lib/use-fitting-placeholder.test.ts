import { describe, expect, it } from "vitest"

import { pickFitting } from "@/lib/use-fitting-placeholder"

const measure = (t: string) => t.length * 8
const candidates = ["aaaaaaaa", "aaaa", "a"]

describe("pickFitting", () => {
  it("returns the full text when it fits", () => {
    expect(pickFitting(candidates, 100, measure)).toBe("aaaaaaaa")
  })

  it("returns the middle candidate when only it fits", () => {
    expect(pickFitting(candidates, 40, measure)).toBe("aaaa")
  })

  it("returns the last candidate when nothing fits", () => {
    expect(pickFitting(candidates, 1, measure)).toBe("a")
  })

  it("counts an exactly equal width as fitting", () => {
    expect(pickFitting(candidates, 64, measure)).toBe("aaaaaaaa")
    expect(pickFitting(candidates, 32, measure)).toBe("aaaa")
  })
})

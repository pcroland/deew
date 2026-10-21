import { describe, expect, it } from "vitest"
import {
  boundaryLeftover,
  momentumStep,
  pixelsToLines,
  releaseVelocity,
} from "./touch-scroll"

describe("pixelsToLines", () => {
  it("swipe up scrolls toward newer output", () => {
    expect(pixelsToLines(-35, 0, 10)).toEqual({ lines: 3, carry: 5 })
  })
  it("swipe down scrolls back", () => {
    expect(pixelsToLines(25, 0, 10)).toEqual({ lines: -2, carry: -5 })
  })
  it("carries the remainder", () => {
    const a = pixelsToLines(-6, 0, 10)
    expect(a.lines).toBe(0)
    expect(pixelsToLines(-6, a.carry, 10)).toEqual({ lines: 1, carry: 2 })
  })
  it("ignores a zero row height", () => {
    expect(pixelsToLines(-5, 0, 0).lines).toBe(0)
  })
})

describe("boundaryLeftover", () => {
  it("passes through inside the buffer", () => {
    expect(boundaryLeftover(3, 5, 20, 10)).toEqual({ lines: 3, leftoverPx: 0 })
  })
  it("leftover at the bottom", () => {
    expect(boundaryLeftover(5, 18, 20, 10)).toEqual({
      lines: 2,
      leftoverPx: 30,
    })
  })
  it("leftover at the top is negative", () => {
    expect(boundaryLeftover(-4, 1, 20, 10)).toEqual({
      lines: -1,
      leftoverPx: -30,
    })
  })
  it("everything is leftover when pinned", () => {
    expect(boundaryLeftover(2, 0, 0, 10)).toEqual({ lines: 0, leftoverPx: 20 })
  })
})

describe("releaseVelocity", () => {
  it("uses only the last 100ms", () => {
    const s = [
      { t: 0, y: 0 },
      { t: 950, y: 100 },
      { t: 1000, y: 150 },
    ]
    expect(releaseVelocity(s, 1000)).toBe(1)
  })
  it("zero with too few samples", () => {
    expect(releaseVelocity([{ t: 0, y: 0 }], 0)).toBe(0)
  })
})

describe("momentumStep", () => {
  it("decays and stops", () => {
    const s = momentumStep(1, 16)!
    expect(s.dy).toBe(16)
    expect(s.velocity).toBeCloseTo(0.95)
    expect(momentumStep(0.01, 16)).toBeNull()
  })
})

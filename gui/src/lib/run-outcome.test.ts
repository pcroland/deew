import { describe, expect, it } from "vitest"

import { runOutcome } from "@/lib/run-outcome"

describe("runOutcome", () => {
  it("is a success for exit code 0", () => {
    expect(runOutcome({ code: 0 })).toBe("success")
  })
  it("is a failure for a nonzero or unknown exit", () => {
    expect(runOutcome({ code: 1 })).toBe("failure")
    expect(runOutcome({ code: null })).toBe("failure")
    expect(runOutcome({ code: undefined })).toBe("failure")
  })
  it("is a failure when stopped, even with code 0", () => {
    expect(runOutcome({ code: 0, stopped: true })).toBe("failure")
  })
  it("is a failure for a start error", () => {
    expect(runOutcome({ error: true })).toBe("failure")
  })
})

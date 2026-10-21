import { describe, expect, it } from "vitest"

import {
  errorStatus,
  exitStatus,
  finishedStatus,
  formatDuration,
  prompt,
  stoppedStatus,
  successStatus,
} from "@/lib/terminal-format"

describe("prompt", () => {
  it("shows the version", () => {
    expect(prompt("3.2.2")).toBe("\x1b[97;48;5;33m deew@3.2.2 \x1b[0m ")
  })
  it("falls back without a version", () => {
    expect(prompt(null)).toBe("\x1b[97;48;5;33m deew \x1b[0m ")
  })
})

describe("status lines", () => {
  it("green on success", () => {
    expect(successStatus(5554)).toBe("\x1b[97;48;5;28m ran for 5.55sec \x1b[0m")
  })
  it("green without a duration for attached jobs", () => {
    expect(finishedStatus()).toBe("\x1b[97;48;5;28m finished \x1b[0m")
  })
  it("red on failure", () => {
    expect(exitStatus(2)).toBe("\x1b[97;48;5;160m exited with code 2 \x1b[0m")
    expect(exitStatus(null)).toBe(
      "\x1b[97;48;5;160m exited with code unknown \x1b[0m"
    )
  })
  it("stopped", () => {
    expect(stoppedStatus()).toBe("\x1b[97;48;5;160m stopped \x1b[0m")
  })
  it("one segment per error line", () => {
    expect(errorStatus("a\nb\r\nc")).toBe(
      [
        "\x1b[97;48;5;160m a \x1b[0m",
        "\x1b[97;48;5;160m b \x1b[0m",
        "\x1b[97;48;5;160m c \x1b[0m",
      ].join("\r\n")
    )
  })
})

describe("formatDuration", () => {
  it("formats", () => {
    expect(formatDuration(4)).toBe("0.00sec")
    expect(formatDuration(5554)).toBe("5.55sec")
    expect(formatDuration(365550)).toBe("6min 5.55sec")
    expect(formatDuration(3965500)).toBe("1h 6min 5.50sec")
  })
})

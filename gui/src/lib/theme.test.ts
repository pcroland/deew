import { describe, expect, it } from "vitest"

import { themeAfterOsChange, themeFromPrefersDark, toggleTheme } from "./theme"

describe("theme", () => {
  it("starts from the OS preference", () => {
    expect(themeFromPrefersDark(true)).toBe("dark")
    expect(themeFromPrefersDark(false)).toBe("light")
  })
  it("toggle flips light and dark", () => {
    expect(toggleTheme("light")).toBe("dark")
    expect(toggleTheme("dark")).toBe("light")
  })
  it("follows OS changes before a manual override", () => {
    expect(themeAfterOsChange("light", true, false)).toBe("dark")
    expect(themeAfterOsChange("dark", false, false)).toBe("light")
  })
  it("ignores OS changes after a manual override", () => {
    expect(themeAfterOsChange("light", true, true)).toBe("light")
    expect(themeAfterOsChange("dark", false, true)).toBe("dark")
  })
})

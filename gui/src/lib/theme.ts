import { useSyncExternalStore } from "react"

export type Theme = "light" | "dark"

/** Legacy persisted key; only ever removed now. */
const LEGACY_STORAGE_KEY = "deew-theme"

export function themeFromPrefersDark(prefersDark: boolean): Theme {
  return prefersDark ? "dark" : "light"
}

export function toggleTheme(theme: Theme): Theme {
  return theme === "dark" ? "light" : "dark"
}

/**
 * Theme after an OS preference change: followed only until the user has
 * toggled manually in this session.
 */
export function themeAfterOsChange(
  current: Theme,
  prefersDark: boolean,
  manualOverride: boolean
): Theme {
  return manualOverride ? current : themeFromPrefersDark(prefersDark)
}

const query = () => window.matchMedia("(prefers-color-scheme: dark)")

let theme: Theme = "light"
let manualOverride = false
const listeners = new Set<() => void>()

function apply() {
  document.documentElement.classList.toggle("dark", theme === "dark")
  listeners.forEach((l) => l())
}

/** Call once before rendering: starts from the OS preference and follows it. */
export function initTheme() {
  try {
    localStorage.removeItem(LEGACY_STORAGE_KEY)
  } catch {
    // Best effort cleanup.
  }
  theme = themeFromPrefersDark(query().matches)
  manualOverride = false
  apply()
  query().addEventListener("change", (e) => {
    theme = themeAfterOsChange(theme, e.matches, manualOverride)
    apply()
  })
}

/** Flip light/dark; stops following the OS for the rest of the session. */
export function toggleCurrentTheme() {
  manualOverride = true
  theme = toggleTheme(theme)
  apply()
}

export function useTheme(): Theme {
  return useSyncExternalStore(
    (l) => {
      listeners.add(l)
      return () => listeners.delete(l)
    },
    () => theme
  )
}

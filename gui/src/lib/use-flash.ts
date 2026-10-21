import { useCallback, useEffect, useState } from "react"

export type FlashFade = "fast" | "slow"
export const FLASH_FADE_MS: Record<FlashFade, number> = {
  fast: 1000,
  slow: 2000,
}

export type FlashKind = "success" | "failure"
type Flash = {
  kind: FlashKind
  phase: "on" | "fade"
  /** Bumped on every start() so a restart re-runs the phase effects. */
  id: number
} | null

/**
 * Colour flash: phase "on" (colour applied instantly), then "fade" (colour
 * removed, transitioned over FLASH_FADE_MS[fade]), then null. Pass `attach` as the `ref` to the
 * flashing element so styles are flushed between "on" and "fade".
 */
export function useFlash(fade: FlashFade = "slow") {
  const [flash, setFlash] = useState<Flash>(null)
  const [el, attach] = useState<HTMLElement | null>(null)

  const start = useCallback((kind: FlashKind) => {
    setFlash((f) => ({ kind, phase: "on", id: (f?.id ?? 0) + 1 }))
  }, [])
  const clear = useCallback(() => setFlash(null), [])

  const phase = flash?.phase
  const id = flash?.id
  useEffect(() => {
    if (phase === "on") {
      let raf2 = 0
      const raf1 = requestAnimationFrame(() => {
        if (el) void getComputedStyle(el).backgroundColor
        raf2 = requestAnimationFrame(() =>
          setFlash((f) => (f ? { ...f, phase: "fade" } : f))
        )
      })
      return () => {
        cancelAnimationFrame(raf1)
        cancelAnimationFrame(raf2)
      }
    }
    if (phase === "fade") {
      const end = window.setTimeout(() => setFlash(null), FLASH_FADE_MS[fade])
      return () => window.clearTimeout(end)
    }
  }, [phase, id, el, fade])

  return { phase, kind: flash?.kind, start, clear, attach }
}

/** Classes for a flashing button: instant colour, then a fade back (1s fast, 2s slow). */
export function flashClass(
  phase: "on" | "fade" | undefined,
  kind: FlashKind | undefined,
  fade: FlashFade = "slow"
): string {
  if (phase === "on")
    return kind === "failure"
      ? // hover/dark/aria-expanded variants of Button set their own bg/text and
        // tailwind-merge keeps them (different modifiers), so override each.
        "transition-none bg-red-600 text-white hover:bg-red-600 hover:text-white focus-visible:bg-red-600 dark:hover:bg-red-600 aria-expanded:bg-red-600 aria-expanded:text-white"
      : "transition-none bg-green-600 text-white hover:bg-green-600 hover:text-white focus-visible:bg-green-600 dark:hover:bg-green-600 aria-expanded:bg-green-600 aria-expanded:text-white"
  if (phase === "fade")
    return fade === "fast"
      ? "transition-colors duration-[1000ms]"
      : "transition-colors duration-[2000ms]"
  return ""
}

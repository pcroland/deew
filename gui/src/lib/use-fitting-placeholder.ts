import { useEffect, useRef, useState } from "react"

/**
 * The first candidate that is at most `available` wide, or the last one when
 * none fits. `measure` returns the rendered width of a text.
 */
export function pickFitting(
  candidates: string[],
  available: number,
  measure: (text: string) => number
): string {
  const fitting = candidates.find((text) => measure(text) <= available)
  return fitting ?? candidates[candidates.length - 1]
}

let measureContext: CanvasRenderingContext2D | null | undefined

/** Lazily created; `null` when canvas is unavailable (e.g. jsdom). */
function getMeasureContext(): CanvasRenderingContext2D | null {
  if (measureContext === undefined) {
    try {
      measureContext = document.createElement("canvas").getContext("2d")
    } catch {
      measureContext = null
    }
  }
  return measureContext
}

function fontOf(style: CSSStyleDeclaration): string {
  if (style.font) return style.font
  // Firefox returns an empty shorthand.
  return `${style.fontStyle} ${style.fontWeight} ${style.fontSize} ${style.fontFamily}`
}

/** Picks the first candidate that fits `el`'s content box; null if unknown. */
function choose(el: HTMLElement, candidates: string[]): string | null {
  const ctx = getMeasureContext()
  if (!ctx || el.clientWidth === 0) return null
  const style = getComputedStyle(el)
  ctx.font = fontOf(style)
  const spacing = parseFloat(style.letterSpacing)
  const letterSpacing = Number.isFinite(spacing) ? spacing : 0
  const available =
    el.clientWidth -
    (parseFloat(style.paddingLeft) || 0) -
    (parseFloat(style.paddingRight) || 0)
  return pickFitting(
    candidates,
    available,
    (text) => ctx.measureText(text).width + letterSpacing * text.length
  )
}

/**
 * A placeholder that is shortened only when the full text does not fit the
 * element. Attach `ref` to the input; `candidates` (longest first) must be
 * a stable array. Falls back to the first candidate when it cannot measure.
 */
export function useFittingPlaceholder<T extends HTMLElement>(
  candidates: string[]
) {
  const ref = useRef<T>(null)
  const [placeholder, setPlaceholder] = useState(candidates[0])

  useEffect(() => {
    const el = ref.current
    if (!el) return
    let active = true
    const update = () => {
      if (!active) return
      const chosen = choose(el, candidates)
      if (chosen !== null) setPlaceholder(chosen)
    }
    update()
    let observer: ResizeObserver | undefined
    if (typeof ResizeObserver !== "undefined") {
      observer = new ResizeObserver(update)
      observer.observe(el)
    }
    // The monospace web font may arrive after the first measurement.
    void document.fonts?.ready.then(update)
    return () => {
      active = false
      observer?.disconnect()
    }
  }, [candidates])

  return { ref, placeholder }
}

/** Pure math for our own touch scrolling of the terminal. */

export const TAP_SLOP_PX = 8
export const MOMENTUM_FRICTION = 0.95 // per 16ms frame
export const MOMENTUM_MIN_VELOCITY = 0.02 // px/ms
export const VELOCITY_WINDOW_MS = 100

/**
 * Adds a pixel delta to the carried remainder and returns whole lines to
 * scroll plus the new remainder. A finger moving down (positive dy) shows
 * earlier output, so it scrolls by negative lines.
 */
export function pixelsToLines(
  dy: number,
  carry: number,
  rowHeight: number
): { lines: number; carry: number } {
  if (!(rowHeight > 0)) return { lines: 0, carry: carry + dy }
  const total = carry - dy
  const lines = Math.trunc(total / rowHeight)
  return { lines, carry: total - lines * rowHeight }
}

/**
 * Splits a scroll request (in lines, positive = toward newer output) into
 * what the terminal can absorb and the leftover pixels for the page (positive
 * = page scrollTop increases).
 */
export function boundaryLeftover(
  lines: number,
  viewportY: number,
  baseY: number,
  rowHeight: number
): { lines: number; leftoverPx: number } {
  const target = Math.min(baseY, Math.max(0, viewportY + lines))
  const applied = target - viewportY
  return { lines: applied, leftoverPx: (lines - applied) * rowHeight }
}

export interface Sample {
  t: number
  y: number
}

/** Velocity in px/ms of the finger over the last VELOCITY_WINDOW_MS. */
export function releaseVelocity(samples: Sample[], now: number): number {
  const recent = samples.filter((s) => now - s.t <= VELOCITY_WINDOW_MS)
  if (recent.length < 2) return 0
  const first = recent[0]
  const last = recent[recent.length - 1]
  const dt = last.t - first.t
  return dt > 0 ? (last.y - first.y) / dt : 0
}

/** One momentum frame: pixel delta to apply and the decayed velocity. */
export function momentumStep(
  velocity: number,
  frameMs: number
): { dy: number; velocity: number } | null {
  if (Math.abs(velocity) < MOMENTUM_MIN_VELOCITY) return null
  return {
    dy: velocity * frameMs,
    velocity: velocity * Math.pow(MOMENTUM_FRICTION, frameMs / 16),
  }
}

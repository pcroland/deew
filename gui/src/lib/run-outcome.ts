export type RunOutcome = "success" | "failure"

/**
 * Whether a finished job counts as a success: exit code 0 and not stopped by
 * the user. An unknown exit (connection lost) or a start failure is a failure.
 */
export function runOutcome(opts: {
  stopped?: boolean
  code?: number | null
  error?: boolean
}): RunOutcome {
  if (opts.stopped || opts.error) return "failure"
  return opts.code === 0 ? "success" : "failure"
}

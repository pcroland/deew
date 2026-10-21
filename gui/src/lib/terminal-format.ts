// Pure helpers for the prompt and status lines written into the terminal.

const RESET = "\x1b[0m"
const WHITE_ON_BLUE = "\x1b[97;48;5;33m"
const WHITE_ON_GREEN = "\x1b[97;48;5;28m"
const WHITE_ON_RED = "\x1b[97;48;5;160m"

const segment = (style: string, text: string): string =>
  `${style} ${text} ${RESET}`

/** Prompt: ` deew@<version> ` segment and a plain space for the arguments. */
export function prompt(version: string | null): string {
  const name = version ? `deew@${version}` : "deew"
  return `${segment(WHITE_ON_BLUE, name)} `
}

export const successStatus = (durationMs: number): string =>
  segment(WHITE_ON_GREEN, `ran for ${formatDuration(durationMs)}`)

/** Success of a job whose start this tab did not see (so no duration). */
export const finishedStatus = (): string => segment(WHITE_ON_GREEN, "finished")

export const exitStatus = (code: number | null): string =>
  segment(WHITE_ON_RED, `exited with code ${code === null ? "unknown" : code}`)

export const stoppedStatus = (): string => segment(WHITE_ON_RED, "stopped")

/** One red segment per line of a (possibly multi-line) launch error. */
export function errorStatus(message: string): string {
  return message
    .split(/\r?\n/)
    .map((line) => segment(WHITE_ON_RED, line))
    .join("\r\n")
}

/** `5.55sec`, `6min 5.55sec` or `1h 6min 5.55sec`. */
export function formatDuration(ms: number): string {
  const total = Math.max(0, ms) / 1000
  const hours = Math.floor(total / 3600)
  const minutes = Math.floor((total - hours * 3600) / 60)
  const seconds = total - hours * 3600 - minutes * 60
  const sec = `${seconds.toFixed(2)}sec`
  if (hours > 0) return `${hours}h ${minutes}min ${sec}`
  if (minutes > 0) return `${minutes}min ${sec}`
  return sec
}

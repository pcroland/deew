// Pure helpers that turn the GUI option state into a deew argv and into a
// shell-quoted command line for display. No Tauri / DOM dependencies.

export const FORMATS = ["dd", "ddp", "ac4", "thd"] as const
export type Format = (typeof FORMATS)[number]

export const DOWNMIX_VALUES = ["none", "1", "2", "6"] as const
export type Downmix = (typeof DOWNMIX_VALUES)[number]

export const DRC_VALUES = [
  "film_light",
  "film_standard",
  "music_light",
  "music_standard",
  "speech",
] as const
export type Drc = (typeof DRC_VALUES)[number]

export const DEFAULT_BITRATE = "default"

export interface DeewOptions {
  inputs: string[]
  /** Integer text, so that the field can be edited freely. */
  trackIndex: string
  output: string
  format: Format
  /** `DEFAULT_BITRATE` or a bitrate in kbps as text. */
  bitrate: string
  downmix: Downmix
  delay: string
  drc: Drc
  /** Integer text between -31 and 0. */
  dialnorm: string
  instances: string
  keepTemp: boolean
  measureOnly: boolean
  forceStandard: boolean
  forceBluray: boolean
  longArgument: boolean
  noPrompt: boolean
}

export const DEFAULT_OPTIONS: Readonly<DeewOptions> = {
  inputs: [],
  trackIndex: "0",
  output: "",
  format: "ddp",
  bitrate: DEFAULT_BITRATE,
  downmix: "none",
  delay: "",
  drc: "music_light",
  dialnorm: "0",
  instances: "",
  keepTemp: false,
  measureOnly: false,
  forceStandard: false,
  forceBluray: false,
  longArgument: false,
  noPrompt: false,
}

export type BooleanOption =
  | "keepTemp"
  | "measureOnly"
  | "forceStandard"
  | "forceBluray"
  | "longArgument"
  | "noPrompt"

/**
 * Flags of the one-off commands that need no input, in menu order. Commands
 * with the same `group` are shown together (separated from other groups).
 */
export const INFO_COMMANDS = [
  { flag: "-h", label: "Help", group: 0 },
  { flag: "-v", label: "Version", group: 0 },
  { flag: "-cl", label: "Changelog", group: 0 },
  { flag: "-lb", label: "List bitrates", group: 1 },
  { flag: "-pl", label: "Print logos", group: 1 },
  { flag: "-c", label: "Show config", group: 2 },
  { flag: "-gc", label: "Generate config", group: 2 },
] as const
export type InfoFlag = (typeof INFO_COMMANDS)[number]["flag"]

export function infoArgs(flag: InfoFlag): string[] {
  return [flag]
}

// Bitrate tables, copied from src/data.rs (a test keeps them in sync).
export const BITRATE_TABLE: Readonly<Record<string, readonly number[]>> = {
  dd_10: [96, 112, 128, 160, 192, 224, 256, 320, 384, 448, 512, 576, 640],
  dd_20: [96, 112, 128, 160, 192, 224, 256, 320, 384, 448, 512, 576, 640],
  dd_51: [224, 256, 320, 384, 448, 512, 576, 640],
  ddp_10: [
    32, 40, 48, 56, 64, 72, 80, 88, 96, 104, 112, 120, 128, 144, 160, 176, 192,
    200, 208, 216, 224, 232, 240, 248, 256, 272, 288, 304, 320, 336, 352, 368,
    384, 400, 448, 512, 576, 640, 704, 768, 832, 896, 960, 1008, 1024,
  ],
  ddp_20: [
    96, 104, 112, 120, 128, 144, 160, 176, 192, 200, 208, 216, 224, 232, 240,
    248, 256, 272, 288, 304, 320, 336, 352, 368, 384, 400, 448, 512, 576, 640,
    704, 768, 832, 896, 960, 1008, 1024,
  ],
  ddp_51: [
    192, 200, 208, 216, 224, 232, 240, 248, 256, 272, 288, 304, 320, 336, 352,
    368, 384, 400, 448, 512, 576, 640, 704, 768, 832, 896, 960, 1008, 1024,
  ],
  ddp_71_standard: [384, 448, 576, 640, 704, 768, 832, 896, 960, 1008, 1024],
  ddp_71_bluray: [768, 1024, 1280, 1536, 1664],
  ddp_71_combined: [
    384, 448, 576, 640, 704, 768, 832, 896, 960, 1008, 1024, 1280, 1536, 1664,
  ],
  ac4_20: [64, 72, 112, 144, 256, 320],
}

const FORMAT_BITRATE_KEYS: Readonly<Record<Format, readonly string[]>> = {
  dd: ["dd_10", "dd_20", "dd_51"],
  ddp: ["ddp_10", "ddp_20", "ddp_51", "ddp_71_combined"],
  ac4: ["ac4_20"],
  thd: [],
}

/** Union of the allowed bitrates of every layout of `format`, ascending. */
export function bitrateOptions(format: Format): number[] {
  const all = new Set<number>()
  for (const key of FORMAT_BITRATE_KEYS[format]) {
    for (const b of BITRATE_TABLE[key]) all.add(b)
  }
  return [...all].sort((a, b) => a - b)
}

export interface BitrateContext {
  format: Format
  /** Channel count common to all probed inputs, `null` if unknown. */
  channels: number | null
  downmix: Downmix
  forceStandard: boolean
  forceBluray: boolean
}

export interface BitrateInfo {
  /**
   * `ok`: exact list for the probed layout; `fallback`: layout unknown, all
   * bitrates of the format; `na`: the format has no bitrate (TrueHD);
   * `invalid`: no encode possible, `bitrates` is empty.
   */
  status: "ok" | "fallback" | "na" | "invalid"
  bitrates: number[]
  /** Short text for the (disabled) select when there is no list. */
  placeholder?: string
  /** Problems the CLI would report for these inputs and options. */
  warnings: string[]
  /**
   * Key of the output layout in the config's `[default_bitrates]` (e.g.
   * `dd_5_1`); unset when the layout is unknown, TrueHD or invalid.
   */
  defaultKey?: string
}

export const CHANNELS_WARNING =
  "Input channels can only be 1, 2, 6 or 8 (1.0, 2.0, 5.1 or 7.1)."

/**
 * The bitrates deew accepts for the probed channel layout, mirroring
 * `resolve_layout` and `select_bitrate` of src/plan.rs.
 */
export function relevantBitrates(c: BitrateContext): BitrateInfo {
  const warnings: string[] = []
  if (c.format === "thd") {
    if (c.channels === 1) {
      warnings.push("Input channels for TrueHD can only be 2, 6 or 8.")
    }
    return {
      status: "na",
      bitrates: [],
      placeholder: "N/A for TrueHD",
      warnings,
    }
  }
  if (c.channels === null) {
    return {
      status: "fallback",
      bitrates: bitrateOptions(c.format),
      warnings,
    }
  }
  if (![1, 2, 6, 8].includes(c.channels)) {
    return {
      status: "invalid",
      bitrates: [],
      placeholder: "unsupported input",
      warnings: [CHANNELS_WARNING],
    }
  }
  const downmix = c.downmix === "none" ? null : Number(c.downmix)
  let outchannels = c.channels
  if (downmix !== null) {
    if (downmix >= c.channels) {
      warnings.push("Downmix has to be lower than the input channels.")
    } else {
      outchannels = downmix
    }
  } else if (c.format === "dd" && c.channels === 8) {
    outchannels = 6
  }
  if (c.format === "ac4") {
    if (c.channels !== 6) {
      return {
        status: "invalid",
        bitrates: [],
        placeholder: "AC4 needs 5.1 input",
        warnings: [...warnings, "Input channels for AC4 can only be 6 (5.1)."],
      }
    }
    return {
      status: "ok",
      bitrates: [...BITRATE_TABLE.ac4_20],
      warnings,
      defaultKey: "ac4_2_0",
    }
  }
  let key: string
  if (c.format === "ddp" && outchannels === 8) {
    key = c.forceStandard
      ? "ddp_71_standard"
      : c.forceBluray
        ? "ddp_71_bluray"
        : "ddp_71_combined"
  } else {
    const layout = { 1: "10", 2: "20", 6: "51" }[outchannels as 1 | 2 | 6]
    key = `${c.format}_${layout}`
  }
  const defaultLayout = { 1: "1_0", 2: "2_0", 6: "5_1", 8: "7_1" }[
    outchannels as 1 | 2 | 6 | 8
  ]
  return {
    status: "ok",
    bitrates: [...BITRATE_TABLE[key]],
    warnings,
    defaultKey: `${c.format}_${defaultLayout}`,
  }
}

/** Built-in `[default_bitrates]` of deew's config template. */
export const TEMPLATE_DEFAULT_BITRATES: Readonly<Record<string, number>> = {
  dd_1_0: 128,
  dd_2_0: 256,
  dd_5_1: 640,
  ddp_1_0: 128,
  ddp_2_0: 256,
  ddp_5_1: 1024,
  ddp_7_1: 1536,
  ac4_2_0: 320,
}

/** `find_closest_allowed` of src/util.rs: the first closest value wins. */
export function closestAllowed(value: number, allowed: readonly number[]) {
  let best: number | undefined
  for (const a of allowed) {
    if (best === undefined || Math.abs(a - value) < Math.abs(best - value)) {
      best = a
    }
  }
  return best
}

export interface BitrateChoice {
  value: string
  label: string
}

/**
 * The entries of the bitrate select. The default entry (value
 * `DEFAULT_BITRATE`) sits at the position of the bitrate the CLI would pick.
 */
export function bitrateChoices(
  info: BitrateInfo,
  configDefaults: Readonly<Record<string, number>>
): BitrateChoice[] {
  const hasList = info.status === "ok" || info.status === "fallback"
  const fromConfig = info.defaultKey
    ? configDefaults[info.defaultKey]
    : undefined
  const fallbackLabel =
    fromConfig === undefined
      ? "default (from config)"
      : `default (${fromConfig} from config)`
  if (!hasList) {
    return [
      { value: DEFAULT_BITRATE, label: info.placeholder ?? fallbackLabel },
    ]
  }
  const wanted = info.defaultKey
    ? (fromConfig ?? TEMPLATE_DEFAULT_BITRATES[info.defaultKey])
    : undefined
  const picked =
    wanted === undefined ? undefined : closestAllowed(wanted, info.bitrates)
  if (picked === undefined) {
    return [
      { value: DEFAULT_BITRATE, label: fallbackLabel },
      ...info.bitrates.map((b) => ({ value: String(b), label: `${b} kbps` })),
    ]
  }
  const suffix = fromConfig === undefined ? "default" : "default from config"
  return info.bitrates.map((b) =>
    b === picked
      ? { value: DEFAULT_BITRATE, label: `${b} kbps (${suffix})` }
      : { value: String(b), label: `${b} kbps` }
  )
}

/**
 * The downmix choices that make sense for the probed input channel count
 * (a downmix has to be lower than the input); all when it is unknown.
 */
export function relevantDownmixes(channels: number | null): Downmix[] {
  if (channels === null) return [...DOWNMIX_VALUES]
  return DOWNMIX_VALUES.filter((d) => d === "none" || Number(d) < channels)
}

/** Changes the format and drops a bitrate the new format cannot do. */
export function setFormat(o: DeewOptions, format: Format): DeewOptions {
  const bitrate =
    o.bitrate !== DEFAULT_BITRATE &&
    !bitrateOptions(format).includes(Number(o.bitrate))
      ? DEFAULT_BITRATE
      : o.bitrate
  return { ...o, format, bitrate }
}

/** Sets a checkbox; -fs and -fb exclude each other. */
export function setFlag(
  o: DeewOptions,
  key: BooleanOption,
  value: boolean
): DeewOptions {
  const next = { ...o, [key]: value }
  if (value && key === "forceStandard") next.forceBluray = false
  if (value && key === "forceBluray") next.forceStandard = false
  return next
}

const INT_RE = /^[+-]?\d+$/

function parseInteger(text: string): number | null {
  const t = text.trim()
  return INT_RE.test(t) ? Number.parseInt(t, 10) : null
}

/** Human readable problems; Run is disabled while there are any. */
export function validate(o: DeewOptions): string[] {
  const problems: string[] = []
  const ti = parseInteger(o.trackIndex)
  if (ti === null || ti < 0) problems.push("Track index must be 0 or more.")
  const dn = parseInteger(o.dialnorm)
  if (dn === null || dn < -31 || dn > 0) {
    problems.push("Dialnorm must be between -31 and 0.")
  }
  return problems
}

/**
 * Builds the deew arguments (without the program name). Only options that
 * differ from the defaults are included. The delay always uses the `-d=`
 * form, since the CLI cannot take a negative value as a separate argument.
 */
export function buildArgs(o: DeewOptions): string[] {
  const args: string[] = []
  if (o.inputs.length > 0) args.push("-i", ...o.inputs)

  const ti = parseInteger(o.trackIndex)
  if (ti !== null && ti !== 0) args.push("-ti", String(ti))
  if (o.output.trim() !== "") args.push("-o", o.output.trim())
  if (o.format !== DEFAULT_OPTIONS.format) args.push("-f", o.format)
  if (o.bitrate !== DEFAULT_BITRATE && o.bitrate.trim() !== "") {
    args.push("-b", o.bitrate.trim())
  }
  if (o.downmix !== "none") args.push("-dm", o.downmix)
  if (o.delay.trim() !== "") args.push(`-d=${o.delay.trim()}`)
  if (o.drc !== DEFAULT_OPTIONS.drc) args.push("-r", o.drc)
  const dn = parseInteger(o.dialnorm)
  if (dn !== null && dn !== 0) args.push("-dn", String(dn))
  if (o.instances.trim() !== "") args.push("-in", o.instances.trim())

  if (o.keepTemp) args.push("-k")
  if (o.measureOnly) args.push("-mo")
  if (o.forceStandard) args.push("-fs")
  if (o.forceBluray) args.push("-fb")
  if (o.longArgument) args.push("-la")
  if (o.noPrompt) args.push("-np")
  return args
}

export type Platform = "posix" | "windows"

export function detectPlatform(userAgent: string): Platform {
  return /windows/i.test(userAgent) ? "windows" : "posix"
}

const POSIX_SAFE = /^[A-Za-z0-9_@%+=:,./-]+$/
const WINDOWS_SAFE = /^[A-Za-z0-9_@+=:,./\\-]+$/

/** Quotes one argument for display in a POSIX shell or on Windows. */
export function quoteArg(arg: string, platform: Platform): string {
  if (platform === "posix") {
    if (POSIX_SAFE.test(arg)) return arg
    return `'${arg.replaceAll("'", `'\\''`)}'`
  }
  if (WINDOWS_SAFE.test(arg)) return arg
  // CommandLineToArgvW rules: double the backslashes in front of a quote
  // (or of the closing quote) and escape the quote itself.
  let out = '"'
  let backslashes = 0
  for (const ch of arg) {
    if (ch === "\\") {
      backslashes++
    } else if (ch === '"') {
      out += "\\".repeat(backslashes * 2 + 1) + '"'
      backslashes = 0
    } else {
      out += "\\".repeat(backslashes) + ch
      backslashes = 0
    }
  }
  return out + "\\".repeat(backslashes * 2) + '"'
}

export function formatCommand(
  args: readonly string[],
  platform: Platform,
  program = "deew"
): string {
  return [program, ...args.map((a) => quoteArg(a, platform))].join(" ")
}

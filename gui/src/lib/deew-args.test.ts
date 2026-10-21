import { readFileSync } from "node:fs"
import { resolve } from "node:path"
import { describe, expect, it } from "vitest"

import {
  BITRATE_TABLE,
  CHANNELS_WARNING,
  DEFAULT_OPTIONS,
  INFO_COMMANDS,
  TEMPLATE_DEFAULT_BITRATES,
  bitrateChoices,
  bitrateOptions,
  buildArgs,
  detectPlatform,
  formatCommand,
  infoArgs,
  quoteArg,
  relevantBitrates,
  relevantDownmixes,
  setFlag,
  setFormat,
  validate,
  type BitrateContext,
  type DeewOptions,
} from "./deew-args"

const opts = (patch: Partial<DeewOptions>): DeewOptions => ({
  ...DEFAULT_OPTIONS,
  ...patch,
})

describe("buildArgs", () => {
  it("is empty for the defaults", () => {
    expect(buildArgs(DEFAULT_OPTIONS)).toEqual([])
  })

  it("omits options equal to their defaults", () => {
    expect(
      buildArgs(
        opts({
          inputs: ["a.mkv"],
          trackIndex: "0",
          format: "ddp",
          bitrate: "default",
          downmix: "none",
          drc: "music_light",
          dialnorm: "0",
          output: "  ",
          delay: " ",
          instances: "",
        })
      )
    ).toEqual(["-i", "a.mkv"])
  })

  it("emits every flag", () => {
    expect(
      buildArgs({
        inputs: ["a.mkv", "/b/c"],
        trackIndex: "2",
        output: "/out",
        format: "dd",
        bitrate: "640",
        downmix: "2",
        delay: "+5ms",
        drc: "speech",
        dialnorm: "-27",
        instances: "50%",
        keepTemp: true,
        measureOnly: true,
        forceStandard: true,
        forceBluray: true,
        longArgument: true,
        noPrompt: true,
      })
    ).toEqual([
      "-i",
      "a.mkv",
      "/b/c",
      "-ti",
      "2",
      "-o",
      "/out",
      "-f",
      "dd",
      "-b",
      "640",
      "-dm",
      "2",
      "-d=+5ms",
      "-r",
      "speech",
      "-dn",
      "-27",
      "-in",
      "50%",
      "-k",
      "-mo",
      "-fs",
      "-fb",
      "-la",
      "-np",
    ])
  })

  it("passes a negative delay in the -d=<value> form", () => {
    expect(buildArgs(opts({ delay: "-5ms" }))).toEqual(["-d=-5ms"])
    expect(buildArgs(opts({ delay: " -24@pal " }))).toEqual(["-d=-24@pal"])
    expect(buildArgs(opts({ delay: "+10@24000/1001" }))).toEqual([
      "-d=+10@24000/1001",
    ])
  })

  it("keeps paths with spaces and quotes as single arguments", () => {
    const inputs = ["/m/My Movie.mkv", '/m/it\'s "x".mkv']
    expect(buildArgs(opts({ inputs }))).toEqual(["-i", ...inputs])
  })

  it("ignores unparsable integers (validate reports them)", () => {
    const o = opts({ trackIndex: "x", dialnorm: "" })
    expect(buildArgs(o)).toEqual([])
    expect(validate(o)).toHaveLength(2)
  })

  it("only includes the selected bitrate and thd format", () => {
    expect(buildArgs(opts({ format: "thd" }))).toEqual(["-f", "thd"])
    expect(buildArgs(opts({ bitrate: "448" }))).toEqual(["-b", "448"])
  })
})

describe("validate", () => {
  it("accepts the defaults and the dialnorm range", () => {
    expect(validate(DEFAULT_OPTIONS)).toEqual([])
    expect(validate(opts({ dialnorm: "-31" }))).toEqual([])
    expect(validate(opts({ trackIndex: "3" }))).toEqual([])
  })

  it("rejects out of range values", () => {
    expect(validate(opts({ dialnorm: "-32" }))).toHaveLength(1)
    expect(validate(opts({ dialnorm: "1" }))).toHaveLength(1)
    expect(validate(opts({ trackIndex: "-1" }))).toHaveLength(1)
    expect(validate(opts({ trackIndex: "1.5" }))).toHaveLength(1)
  })
})

describe("-fs / -fb exclusivity", () => {
  it("turning one on turns the other off", () => {
    let o = setFlag(DEFAULT_OPTIONS, "forceStandard", true)
    expect(buildArgs(o)).toEqual(["-fs"])
    o = setFlag(o, "forceBluray", true)
    expect(o.forceStandard).toBe(false)
    expect(buildArgs(o)).toEqual(["-fb"])
    o = setFlag(o, "forceStandard", true)
    expect(buildArgs(o)).toEqual(["-fs"])
  })

  it("turning one off keeps the other", () => {
    const o = setFlag(
      setFlag(DEFAULT_OPTIONS, "forceBluray", true),
      "forceStandard",
      false
    )
    expect(o.forceBluray).toBe(true)
  })

  it("does not touch unrelated flags", () => {
    const o = setFlag(
      setFlag(DEFAULT_OPTIONS, "keepTemp", true),
      "forceBluray",
      true
    )
    expect(buildArgs(o)).toEqual(["-k", "-fb"])
  })
})

describe("info commands", () => {
  it("are single flags without input", () => {
    expect(INFO_COMMANDS.map((c) => c.flag)).toEqual([
      "-h",
      "-v",
      "-cl",
      "-lb",
      "-pl",
      "-c",
      "-gc",
    ])
    const groups = INFO_COMMANDS.map((c) => c.group)
    expect(groups).toEqual([...groups].sort())
    for (const { flag } of INFO_COMMANDS) {
      expect(infoArgs(flag)).toEqual([flag])
    }
  })
})

describe("bitrates", () => {
  it("returns the union per format", () => {
    const dd = bitrateOptions("dd")
    expect(dd[0]).toBe(96)
    expect(dd.at(-1)).toBe(640)
    expect(dd).toContain(224)
    const ddp = bitrateOptions("ddp")
    expect(ddp[0]).toBe(32)
    expect(ddp.at(-1)).toBe(1664)
    expect(ddp).toContain(1280)
    expect(new Set(ddp).size).toBe(ddp.length)
    expect(ddp).toEqual([...ddp].sort((a, b) => a - b))
    expect(bitrateOptions("ac4")).toEqual([64, 72, 112, 144, 256, 320])
    expect(bitrateOptions("thd")).toEqual([])
  })

  it("resets an impossible bitrate when the format changes", () => {
    expect(setFormat(opts({ bitrate: "1024" }), "dd").bitrate).toBe("default")
    expect(setFormat(opts({ bitrate: "448" }), "dd").bitrate).toBe("448")
    expect(setFormat(opts({ bitrate: "448" }), "thd").bitrate).toBe("default")
    expect(setFormat(opts({ bitrate: "default" }), "ac4").bitrate).toBe(
      "default"
    )
  })

  it("matches the tables in src/data.rs", () => {
    const source = readFileSync(
      resolve(import.meta.dirname, "../../../src/data.rs"),
      "utf8"
    )
    const block = source.slice(
      source.indexOf("pub const BITRATES"),
      source.indexOf("pub fn allowed_bitrates")
    )
    const parsed: Record<string, number[]> = {}
    for (const m of block.matchAll(/"([a-z0-9_]+)",\s*&\[([^\]]*)\]/g)) {
      parsed[m[1]] = m[2]
        .split(",")
        .map((s) => s.trim())
        .filter(Boolean)
        .map(Number)
    }
    expect(Object.keys(parsed).length).toBeGreaterThan(5)
    expect(parsed).toEqual(
      Object.fromEntries(
        Object.entries(BITRATE_TABLE).map(([k, v]) => [k, [...v]])
      )
    )
  })
})

describe("quoteArg / formatCommand", () => {
  it("leaves simple arguments alone", () => {
    expect(quoteArg("-d=-5ms", "posix")).toBe("-d=-5ms")
    expect(quoteArg("/a/b-c_d.mkv", "posix")).toBe("/a/b-c_d.mkv")
    expect(quoteArg("50%", "posix")).toBe("50%")
    expect(quoteArg("C:\\a\\b.mkv", "windows")).toBe("C:\\a\\b.mkv")
  })

  it("single-quotes for POSIX", () => {
    expect(quoteArg("/m/My Movie.mkv", "posix")).toBe("'/m/My Movie.mkv'")
    expect(quoteArg("it's", "posix")).toBe(`'it'\\''s'`)
    expect(quoteArg("", "posix")).toBe("''")
    expect(quoteArg("a$b", "posix")).toBe("'a$b'")
    expect(quoteArg('say "hi"', "posix")).toBe(`'say "hi"'`)
    expect(quoteArg("-d=+1,52s", "posix")).toBe("-d=+1,52s")
    expect(quoteArg("-d=+10@24000/1001", "posix")).toBe("-d=+10@24000/1001")
  })

  it("double-quotes for Windows", () => {
    expect(quoteArg("C:\\My Movies\\a.mkv", "windows")).toBe(
      '"C:\\My Movies\\a.mkv"'
    )
    expect(quoteArg("", "windows")).toBe('""')
    expect(quoteArg('say "hi"', "windows")).toBe('"say \\"hi\\""')
    expect(quoteArg("C:\\dir with space\\", "windows")).toBe(
      '"C:\\dir with space\\\\"'
    )
    expect(quoteArg('a\\"b c', "windows")).toBe('"a\\\\\\"b c"')
    expect(quoteArg("a&b", "windows")).toBe('"a&b"')
  })

  it("formats a full command", () => {
    const args = buildArgs(
      opts({ inputs: ["/m/My Movie.mkv"], delay: "-5ms", noPrompt: true })
    )
    expect(formatCommand(args, "posix")).toBe(
      "deew -i '/m/My Movie.mkv' -d=-5ms -np"
    )
    expect(formatCommand(["-lb"], "windows")).toBe("deew -lb")
    expect(formatCommand(["-i", "C:\\My Movies\\a.mkv"], "windows")).toBe(
      'deew -i "C:\\My Movies\\a.mkv"'
    )
  })

  it("detects the platform from the user agent", () => {
    expect(detectPlatform("Mozilla/5.0 (Windows NT 10.0; Win64; x64)")).toBe(
      "windows"
    )
    expect(detectPlatform("Mozilla/5.0 (X11; Linux x86_64)")).toBe("posix")
    expect(detectPlatform("Mozilla/5.0 (Macintosh; Intel Mac OS X)")).toBe(
      "posix"
    )
  })
})

describe("relevantBitrates", () => {
  const ctx = (patch: Partial<BitrateContext>): BitrateContext => ({
    format: "ddp",
    channels: 6,
    downmix: "none",
    forceStandard: false,
    forceBluray: false,
    ...patch,
  })
  const table = BITRATE_TABLE

  it("picks the table of the input layout", () => {
    for (const [format, channels, key] of [
      ["dd", 1, "dd_10"],
      ["dd", 2, "dd_20"],
      ["dd", 6, "dd_51"],
      ["ddp", 1, "ddp_10"],
      ["ddp", 2, "ddp_20"],
      ["ddp", 6, "ddp_51"],
      ["ddp", 8, "ddp_71_combined"],
    ] as const) {
      const info = relevantBitrates(ctx({ format, channels }))
      expect(info.status).toBe("ok")
      expect(info.bitrates).toEqual(table[key])
      expect(info.warnings).toEqual([])
    }
  })

  it("downmixes DD from 7.1 to 5.1 by default", () => {
    expect(
      relevantBitrates(ctx({ format: "dd", channels: 8 })).bitrates
    ).toEqual(table.dd_51)
    // An explicit downmix wins.
    expect(
      relevantBitrates(ctx({ format: "dd", channels: 8, downmix: "2" }))
        .bitrates
    ).toEqual(table.dd_20)
  })

  it("uses the downmix as the output layout", () => {
    expect(
      relevantBitrates(ctx({ channels: 8, downmix: "6" })).bitrates
    ).toEqual(table.ddp_51)
    expect(
      relevantBitrates(ctx({ channels: 6, downmix: "1" })).bitrates
    ).toEqual(table.ddp_10)
  })

  it("ignores a downmix that is not lower than the input channels", () => {
    const info = relevantBitrates(ctx({ channels: 2, downmix: "6" }))
    expect(info.bitrates).toEqual(table.ddp_20)
    expect(info.warnings).toHaveLength(1)
    expect(
      relevantBitrates(ctx({ channels: 2, downmix: "2" })).warnings
    ).toHaveLength(1)
  })

  it("uses the 7.1 profile of the force flags, only for DDP without downmix", () => {
    expect(
      relevantBitrates(ctx({ channels: 8, forceStandard: true })).bitrates
    ).toEqual(table.ddp_71_standard)
    expect(
      relevantBitrates(ctx({ channels: 8, forceBluray: true })).bitrates
    ).toEqual(table.ddp_71_bluray)
    // -fs wins like in select_bitrate (the GUI never sets both).
    expect(
      relevantBitrates(
        ctx({ channels: 8, forceStandard: true, forceBluray: true })
      ).bitrates
    ).toEqual(table.ddp_71_standard)
    expect(
      relevantBitrates(ctx({ channels: 8, downmix: "6", forceStandard: true }))
        .bitrates
    ).toEqual(table.ddp_51)
    expect(
      relevantBitrates(ctx({ format: "dd", channels: 8, forceBluray: true }))
        .bitrates
    ).toEqual(table.dd_51)
  })

  it("AC4 needs 5.1 input and always encodes stereo", () => {
    for (const downmix of ["none", "2"] as const) {
      const info = relevantBitrates(
        ctx({ format: "ac4", channels: 6, downmix })
      )
      expect(info.status).toBe("ok")
      expect(info.bitrates).toEqual(table.ac4_20)
    }
    for (const channels of [1, 2, 8]) {
      const info = relevantBitrates(ctx({ format: "ac4", channels }))
      expect(info.status).toBe("invalid")
      expect(info.bitrates).toEqual([])
      expect(info.placeholder).toBe("AC4 needs 5.1 input")
      expect(info.warnings.length).toBeGreaterThan(0)
    }
  })

  it("has no bitrate for TrueHD", () => {
    const info = relevantBitrates(ctx({ format: "thd" }))
    expect(info).toMatchObject({
      status: "na",
      bitrates: [],
      placeholder: "N/A for TrueHD",
    })
    expect(info.warnings).toEqual([])
    expect(
      relevantBitrates(ctx({ format: "thd", channels: 1 })).warnings
    ).toHaveLength(1)
  })

  it("rejects unsupported channel counts", () => {
    for (const channels of [0, 3, 4, 5, 7, 9]) {
      const info = relevantBitrates(ctx({ channels }))
      expect(info.status).toBe("invalid")
      expect(info.bitrates).toEqual([])
      expect(info.warnings).toEqual([CHANNELS_WARNING])
    }
  })

  it("falls back to all bitrates of the format without a layout", () => {
    for (const format of ["dd", "ddp", "ac4"] as const) {
      const info = relevantBitrates(ctx({ format, channels: null }))
      expect(info.status).toBe("fallback")
      expect(info.bitrates).toEqual(bitrateOptions(format))
    }
  })

  it("every result is a subset of the format's bitrates", () => {
    for (const format of ["dd", "ddp", "ac4"] as const) {
      for (const channels of [1, 2, 6, 8]) {
        for (const downmix of ["none", "1", "2", "6"] as const) {
          for (const forceStandard of [false, true]) {
            const all = bitrateOptions(format)
            const info = relevantBitrates(
              ctx({ format, channels, downmix, forceStandard })
            )
            for (const b of info.bitrates) expect(all).toContain(b)
          }
        }
      }
    }
  })
})

describe("relevantBitrates defaultKey", () => {
  const key = (patch: Partial<BitrateContext>) =>
    relevantBitrates({
      format: "ddp",
      channels: 6,
      downmix: "none",
      forceStandard: false,
      forceBluray: false,
      ...patch,
    }).defaultKey

  it("names the output layout", () => {
    expect(key({ format: "dd", channels: 1 })).toBe("dd_1_0")
    expect(key({ format: "dd", channels: 2 })).toBe("dd_2_0")
    expect(key({ format: "dd", channels: 6 })).toBe("dd_5_1")
    expect(key({ format: "ddp", channels: 8 })).toBe("ddp_7_1")
    expect(key({ format: "ddp", channels: 8, forceBluray: true })).toBe(
      "ddp_7_1"
    )
    expect(key({ format: "ac4", channels: 6 })).toBe("ac4_2_0")
  })

  it("follows the downmix and the DD 7.1 reduction", () => {
    expect(key({ format: "dd", channels: 8 })).toBe("dd_5_1")
    expect(key({ channels: 8, downmix: "2" })).toBe("ddp_2_0")
    expect(key({ channels: 6, downmix: "1" })).toBe("ddp_1_0")
  })

  it("is unset without a known valid layout", () => {
    expect(key({ channels: null })).toBeUndefined()
    expect(key({ channels: 3 })).toBeUndefined()
    expect(key({ format: "thd", channels: 6 })).toBeUndefined()
    expect(key({ format: "ac4", channels: 2 })).toBeUndefined()
  })
})

describe("relevantDownmixes", () => {
  it("offers only lower channel counts", () => {
    expect(relevantDownmixes(1)).toEqual(["none"])
    expect(relevantDownmixes(2)).toEqual(["none", "1"])
    expect(relevantDownmixes(6)).toEqual(["none", "1", "2"])
    expect(relevantDownmixes(8)).toEqual(["none", "1", "2", "6"])
  })
  it("offers everything when the channels are unknown", () => {
    expect(relevantDownmixes(null)).toEqual(["none", "1", "2", "6"])
  })
})

describe("bitrateChoices", () => {
  const ctx = {
    format: "ddp" as const,
    channels: 6,
    downmix: "none" as const,
    forceStandard: false,
    forceBluray: false,
  }
  const info = relevantBitrates(ctx)

  it("puts a config value at its position", () => {
    const c = bitrateChoices(info, { ddp_5_1: 768 })
    const i = c.findIndex((x) => x.value === "default")
    expect(c[i].label).toBe("768 kbps (default from config)")
    expect(c[i - 1].label).toBe("704 kbps")
    expect(c[i + 1].label).toBe("832 kbps")
    expect(c).toHaveLength(info.bitrates.length)
    expect(c.filter((x) => x.label.startsWith("768 "))).toHaveLength(1)
    expect(c.some((x) => x.value === "768")).toBe(false)
  })

  it("rounds the config value to the closest, first wins ties", () => {
    // 260 is between 256 and 272 (closer to 256); 280 ties 272/288.
    const a = bitrateChoices(info, { ddp_5_1: 260 })
    expect(a.find((x) => x.value === "default")?.label).toContain("256 kbps")
    const b = bitrateChoices(info, { ddp_5_1: 280 })
    expect(b.find((x) => x.value === "default")?.label).toContain("272 kbps")
  })

  it("uses the template default without config", () => {
    const c = bitrateChoices(info, {})
    expect(c.find((x) => x.value === "default")?.label).toBe(
      "1024 kbps (default)"
    )
  })

  it("has a template key for every default key", () => {
    for (const format of ["dd", "ddp", "ac4"] as const)
      for (const channels of [1, 2, 6, 8])
        for (const downmix of ["none", "1", "2", "6"] as const)
          for (const [fs, fb] of [
            [false, false],
            [true, false],
            [false, true],
          ]) {
            const i = relevantBitrates({
              format,
              channels,
              downmix,
              forceStandard: fs,
              forceBluray: fb,
            })
            if (i.defaultKey)
              expect(TEMPLATE_DEFAULT_BITRATES).toHaveProperty(i.defaultKey)
          }
  })

  it("keeps a top default entry without a known default", () => {
    const fb = relevantBitrates({ ...ctx, channels: null })
    const c = bitrateChoices(fb, { ddp_5_1: 768 })
    expect(c[0]).toEqual({ value: "default", label: "default (from config)" })
    expect(c).toHaveLength(fb.bitrates.length + 1)
    const na = bitrateChoices(relevantBitrates({ ...ctx, format: "thd" }), {})
    expect(na).toEqual([{ value: "default", label: "N/A for TrueHD" }])
  })
})

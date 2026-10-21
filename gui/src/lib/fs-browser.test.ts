import { describe, expect, it } from "vitest"

import {
  fsRequestPath,
  sortEntries,
  toggleSelected,
  visibleEntries,
  type FsEntry,
} from "./fs-browser"

const entry = (name: string, dir = false, hidden = false): FsEntry => ({
  name,
  path: `/x/${name}`,
  dir,
  hidden,
})

describe("sortEntries", () => {
  it("puts directories first and ignores case", () => {
    const sorted = sortEntries([
      entry("b.wav"),
      entry("Zeta", true),
      entry("A.wav"),
      entry("alpha", true),
    ])
    expect(sorted.map((e) => e.name)).toEqual([
      "alpha",
      "Zeta",
      "A.wav",
      "b.wav",
    ])
  })

  it("does not modify its input", () => {
    const input = [entry("b"), entry("a")]
    sortEntries(input)
    expect(input.map((e) => e.name)).toEqual(["b", "a"])
  })
})

describe("visibleEntries", () => {
  const all = [
    entry(".git", true, true),
    entry("a", true),
    entry(".rc", false, true),
  ]

  it("hides hidden entries by default", () => {
    expect(visibleEntries(all, false).map((e) => e.name)).toEqual(["a"])
  })

  it("shows them on request", () => {
    expect(visibleEntries(all, true).map((e) => e.name)).toEqual([
      ".git",
      "a",
      ".rc",
    ])
  })

  it("treats a missing hidden flag as visible", () => {
    const e: FsEntry = { name: "a", path: "/a", dir: false }
    expect(visibleEntries([e], false)).toEqual([e])
  })
})

describe("toggleSelected", () => {
  it("adds and removes", () => {
    expect(toggleSelected(["/a"], "/b")).toEqual(["/a", "/b"])
    expect(toggleSelected(["/a", "/b"], "/a")).toEqual(["/b"])
  })
})

describe("fsRequestPath", () => {
  it("omits an empty path and encodes others", () => {
    expect(fsRequestPath()).toBe("/api/fs")
    expect(fsRequestPath("")).toBe("/api/fs")
    expect(fsRequestPath("C:\\My Music&x")).toBe(
      "/api/fs?path=C%3A%5CMy%20Music%26x"
    )
  })
})

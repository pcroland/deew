import { describe, expect, it } from "vitest"
import { cn } from "cn"
import { buttonVariants } from "@/components/ui/button"
import { flashClass } from "./use-flash"

describe("flashClass", () => {
  for (const variant of ["ghost", "default"] as const)
    for (const kind of ["success", "failure"] as const)
      it(`beats ${variant} hover/dark/expanded colours (${kind})`, () => {
        const out = cn(buttonVariants({ variant }), flashClass("on", kind))
        expect(out).not.toMatch(
          /bg-muted|text-foreground|bg-primary|text-primary-foreground/
        )
      })

  for (const kind of ["success", "failure"] as const)
    it(`beats the copy button's own colours (${kind})`, () => {
      const out = cn(
        "group flex w-7 shrink-0 items-start justify-center self-stretch pt-[7px] text-muted-foreground transition-colors outline-none hover:bg-muted hover:text-foreground focus-visible:bg-muted",
        flashClass("on", kind, "fast")
      )
      expect(out).not.toMatch(/bg-muted|text-foreground|text-muted-foreground/)
      expect(out).toContain("transition-none")
    })

  it("fade and idle", () => {
    expect(flashClass("fade", "success")).toContain("transition-colors")
    expect(flashClass(undefined, undefined)).toBe("")
  })

  it("fade duration follows the speed", () => {
    expect(flashClass("fade", "success")).toContain("duration-[2000ms]")
    expect(flashClass("fade", "success", "slow")).toContain("duration-[2000ms]")
    expect(flashClass("fade", "success", "fast")).toContain("duration-[1000ms]")
    expect(flashClass("fade", "success", "fast")).not.toContain("2000")
  })
})

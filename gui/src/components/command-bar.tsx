import { useEffect, useRef } from "react"
import { Copy, Play, Square } from "lucide-react"

import { Button } from "@/components/ui/button"
import { flashClass, useFlash } from "@/lib/use-flash"
import { cn } from "@/lib/utils"

interface CommandBarProps {
  command: string
  problems: string[]
  running: boolean
  /** Drives the Run button colour. */
  runState: "idle" | "running" | "success" | "failure"
  /** Changes every time a job ends, so two equal outcomes both flash. */
  endCount: number
  canRun: boolean
  onRun: () => void
  onStop: () => void
}

export function CommandBar({
  command,
  problems,
  running,
  runState,
  endCount,
  canRun,
  onRun,
  onStop,
}: CommandBarProps) {
  const {
    phase: copyPhase,
    kind: copyKind,
    start: startCopyFlash,
    attach: attachCopy,
  } = useFlash("fast")
  const copy = async () => {
    try {
      await navigator.clipboard.writeText(command)
      startCopyFlash("success")
    } catch (error) {
      console.error("copy failed", error)
    }
  }

  // Result flash: colour instantly, then fade out.
  const {
    phase: runPhase,
    kind: runKind,
    start: startRunFlash,
    clear: clearRunFlash,
    attach: attachRun,
  } = useFlash()
  const runStateRef = useRef(runState)
  useEffect(() => {
    runStateRef.current = runState
  })
  useEffect(() => {
    const kind = runStateRef.current
    if (running || endCount === 0 || (kind !== "success" && kind !== "failure"))
      return
    startRunFlash(kind)
    return clearRunFlash
  }, [endCount, running, startRunFlash, clearRunFlash])

  const runClass = running
    ? "bg-amber-400 text-black hover:bg-amber-400 disabled:opacity-100 dark:bg-amber-400 dark:text-black"
    : flashClass(runPhase, runKind)

  return (
    <div className="@container flex flex-col gap-1">
      <div className="flex flex-col gap-2 @min-[520px]:flex-row @min-[520px]:items-stretch">
        <div className="flex min-h-7 min-w-0 items-stretch overflow-hidden rounded-lg border border-input bg-muted/40 @min-[520px]:flex-1">
          <div
            className="ui-selectable max-h-40 min-w-0 flex-1 overflow-y-auto px-2.5 py-[5px] font-mono text-xs break-all whitespace-pre-wrap @min-[520px]:max-h-20"
            data-testid="command"
          >
            {command}
          </div>
          <div className="w-px shrink-0 self-stretch bg-border" />
          <button
            type="button"
            ref={attachCopy}
            className={cn(
              "group flex w-7 shrink-0 items-start justify-center self-stretch pt-[7px] text-muted-foreground transition-colors outline-none hover:bg-muted hover:text-foreground focus-visible:bg-muted",
              flashClass(copyPhase, copyKind, "fast")
            )}
            aria-label="Copy command"
            title="Copy command"
            onClick={copy}
          >
            <Copy className="size-3 group-active:translate-y-px" />
          </button>
        </div>
        <div className="flex items-stretch gap-2">
          <Button
            ref={attachRun}
            size="sm"
            className={cn("flex-1 @min-[520px]:flex-none", runClass)}
            onClick={onRun}
            disabled={!canRun || running}
          >
            <Play data-icon="inline-start" />
            Run
          </Button>
          <Button
            size="sm"
            className="flex-1 @min-[520px]:flex-none"
            variant="destructive"
            onClick={onStop}
            disabled={!running}
          >
            <Square data-icon="inline-start" />
            Stop
          </Button>
        </div>
      </div>
      {problems.length > 0 && (
        <p className="text-xs text-destructive">{problems.join(" ")}</p>
      )}
    </div>
  )
}

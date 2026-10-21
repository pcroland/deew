import type { ReactElement, ReactNode } from "react"

import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip"

/** Wraps `render` (the element that gets hovered) with a tooltip. */
export function Hint({
  text,
  render,
  children,
}: {
  text: string
  render: ReactElement
  children?: ReactNode
}) {
  return (
    <Tooltip>
      <TooltipTrigger render={render}>{children}</TooltipTrigger>
      <TooltipContent className="max-w-sm items-start whitespace-pre-line">
        {text}
      </TooltipContent>
    </Tooltip>
  )
}

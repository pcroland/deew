import { Fragment } from "react"
import { Wrench } from "lucide-react"

import { Button } from "@/components/ui/button"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import { INFO_COMMANDS, type InfoFlag } from "@/lib/deew-args"
import { HELP } from "@/lib/help-text"

export function UtilitiesMenu({
  disabled,
  onRun,
}: {
  disabled: boolean
  onRun: (flag: InfoFlag) => void
}) {
  return (
    <DropdownMenu>
      <DropdownMenuTrigger
        render={
          <Button
            variant="ghost"
            size="xs"
            className="h-5 px-1.5 text-[0.7rem]"
            disabled={disabled}
          />
        }
      >
        <Wrench data-icon="inline-start" />
        Utilities
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end" className="min-w-52">
        {INFO_COMMANDS.map(({ flag, label, group }, i) => (
          <Fragment key={flag}>
            {i > 0 && INFO_COMMANDS[i - 1].group !== group && (
              <DropdownMenuSeparator />
            )}
            <DropdownMenuGroup>
              <DropdownMenuItem
                title={HELP.info[flag]}
                onClick={() => onRun(flag)}
              >
                {label}
                <code className="ml-auto font-mono text-xs text-muted-foreground">
                  {flag}
                </code>
              </DropdownMenuItem>
            </DropdownMenuGroup>
          </Fragment>
        ))}
      </DropdownMenuContent>
    </DropdownMenu>
  )
}

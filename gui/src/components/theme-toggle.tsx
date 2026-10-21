import { Moon, Sun } from "lucide-react"

import { Button } from "@/components/ui/button"
import { toggleCurrentTheme, useTheme } from "@/lib/theme"

/** Two-state toggle; the icon shows the current theme. */
export function ThemeToggle() {
  const theme = useTheme()
  const Icon = theme === "dark" ? Moon : Sun
  const label = `Switch to ${theme === "dark" ? "light" : "dark"} theme`
  return (
    <Button
      variant="ghost"
      size="icon-xs"
      className="size-5"
      aria-label={label}
      title={label}
      tabIndex={-1}
      onClick={toggleCurrentTheme}
    >
      <Icon />
    </Button>
  )
}

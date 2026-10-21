import { useCallback, useEffect, useRef, useState } from "react"
import {
  ArrowUp,
  ChevronRight,
  File,
  Folder,
  Loader2,
  TriangleAlert,
} from "lucide-react"

import { Button } from "@/components/ui/button"
import { Checkbox } from "@/components/ui/checkbox"
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import {
  toggleSelected,
  visibleEntries,
  type FsEntry,
  type FsListing,
} from "@/lib/fs-browser"
import { listDirectory } from "@/lib/runner"
import { errorMessage } from "@/lib/web-api"
import { cn } from "@/lib/utils"

/** The directory last browsed in this session (web mode). */
let lastDirectory: string | undefined

export interface BrowseRequest {
  /** Pick the open directory instead of files. */
  directory: boolean
  /** Resolves with the chosen paths (empty if cancelled). */
  resolve: (paths: string[]) => void
}

function EntryRow({
  entry,
  selectable,
  selected,
  onToggle,
  onOpen,
}: {
  entry: FsEntry
  selectable: boolean
  selected: boolean
  onToggle: () => void
  onOpen: () => void
}) {
  const Icon = entry.dir ? Folder : File
  return (
    <li
      className={cn(
        "flex items-center gap-2 rounded px-1.5 py-1 select-none hover:bg-muted",
        selected && "bg-muted",
        !entry.dir && !selectable && "opacity-50"
      )}
      onDoubleClick={entry.dir ? onOpen : undefined}
    >
      {selectable && !entry.dir ? (
        <Checkbox
          checked={selected}
          onCheckedChange={onToggle}
          aria-label={`Select ${entry.name}`}
        />
      ) : (
        <span className="size-4 shrink-0" />
      )}
      <Icon
        className={cn(
          "size-4 shrink-0",
          entry.dir ? "text-primary" : "text-muted-foreground"
        )}
      />
      <span
        className={cn("min-w-0 flex-1 truncate", entry.hidden && "italic")}
        title={entry.name}
        onClick={selectable && !entry.dir ? onToggle : undefined}
      >
        {entry.name}
      </span>
      {entry.dir && (
        <button
          type="button"
          aria-label={`Open ${entry.name}`}
          className="rounded p-0.5 text-muted-foreground hover:text-foreground"
          onClick={onOpen}
        >
          <ChevronRight className="size-4" />
        </button>
      )}
    </li>
  )
}

function BrowserBody({
  directory,
  onDone,
}: {
  directory: boolean
  onDone: (paths: string[]) => void
}) {
  const [listing, setListing] = useState<FsListing | null>(null)
  const [pathInput, setPathInput] = useState("")
  const [error, setError] = useState<string | null>(null)
  const [loading, setLoading] = useState(true)
  const [showHidden, setShowHidden] = useState(false)
  const [selected, setSelected] = useState<string[]>([])
  const requestCounter = useRef(0)

  // Errors keep the current listing; only the newest request counts.
  const track = useCallback((request: Promise<FsListing>) => {
    const id = ++requestCounter.current
    request
      .then(
        (result) => {
          if (id !== requestCounter.current) return
          lastDirectory = result.path
          setListing(result)
          setPathInput(result.path)
          setError(null)
        },
        (e) => {
          if (id === requestCounter.current) setError(errorMessage(e))
        }
      )
      .finally(() => {
        if (id === requestCounter.current) setLoading(false)
      })
  }, [])

  const load = (path?: string) => {
    setLoading(true)
    track(listDirectory(path))
  }

  useEffect(() => {
    track(listDirectory(lastDirectory))
  }, [track])

  const entries = listing ? visibleEntries(listing.entries, showHidden) : []
  const roots = listing && listing.roots.length > 1 ? listing.roots : []
  const confirm = () => onDone(directory ? [listing!.path] : selected)

  return (
    <>
      <div className="flex flex-wrap items-center gap-1.5">
        <Button
          size="icon-sm"
          variant="outline"
          aria-label="Parent folder"
          title="Parent folder"
          disabled={!listing?.parent}
          onClick={() => listing?.parent && load(listing.parent)}
        >
          <ArrowUp />
        </Button>
        {roots.map((root) => (
          <Button
            key={root}
            size="sm"
            variant="outline"
            onClick={() => load(root)}
          >
            {root}
          </Button>
        ))}
        <Input
          className="h-7 min-w-40 flex-1 font-mono"
          aria-label="Path"
          value={pathInput}
          spellCheck={false}
          onChange={(e) => setPathInput(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter") load(pathInput.trim() || undefined)
          }}
        />
        {loading && <Loader2 className="size-4 animate-spin" />}
      </div>
      {error && (
        <p className="flex items-center gap-1 text-xs text-destructive">
          <TriangleAlert className="size-3 shrink-0" />
          {error}
        </p>
      )}
      <ul className="min-h-0 flex-1 overflow-y-auto rounded-lg border border-input p-1 text-sm">
        {entries.map((entry) => (
          <EntryRow
            key={entry.path}
            entry={entry}
            selectable={!directory}
            selected={selected.includes(entry.path)}
            onToggle={() => setSelected((s) => toggleSelected(s, entry.path))}
            onOpen={() => load(entry.path)}
          />
        ))}
        {listing && entries.length === 0 && (
          <li className="px-2 py-3 text-center text-xs text-muted-foreground">
            Nothing here.
          </li>
        )}
      </ul>
      <div className="flex items-center gap-2">
        <Checkbox
          id="fs-show-hidden"
          checked={showHidden}
          onCheckedChange={(checked) => setShowHidden(checked === true)}
        />
        <Label htmlFor="fs-show-hidden" className="text-xs">
          Show hidden
        </Label>
      </div>
      <DialogFooter>
        <Button variant="outline" onClick={() => onDone([])}>
          Cancel
        </Button>
        <Button
          disabled={directory ? !listing : selected.length === 0}
          onClick={confirm}
        >
          {directory
            ? "Select this folder"
            : selected.length > 0
              ? `Add ${selected.length} file${selected.length === 1 ? "" : "s"}`
              : "Add files"}
        </Button>
      </DialogFooter>
    </>
  )
}

/**
 * Server-side file browser (there are no native dialogs in a browser).
 * Files mode picks files; folder mode picks the directory that is open.
 */
export function FileBrowserDialog({
  request,
}: {
  request: BrowseRequest | null
}) {
  const directory = request?.directory ?? false
  return (
    <Dialog
      open={request !== null}
      onOpenChange={(open) => {
        if (!open) request?.resolve([])
      }}
    >
      <DialogContent className="h-[34rem] sm:max-w-2xl">
        <DialogHeader>
          <DialogTitle>
            {directory ? "Select a folder" : "Select files"}
          </DialogTitle>
        </DialogHeader>
        {request && (
          <BrowserBody directory={directory} onDone={request.resolve} />
        )}
      </DialogContent>
    </Dialog>
  )
}

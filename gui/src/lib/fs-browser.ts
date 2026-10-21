// Pure helpers for the server-side file browser (GET /api/fs of deew-web).

export interface FsEntry {
  name: string
  path: string
  dir: boolean
  hidden?: boolean
}

export interface FsListing {
  /** Canonical absolute path of the listed directory. */
  path: string
  parent: string | null
  /** `/`, or the drive letters on Windows. */
  roots: string[]
  entries: FsEntry[]
}

const byName = (a: FsEntry, b: FsEntry): number =>
  a.name.localeCompare(b.name, undefined, { sensitivity: "base" })

/** Directories first, then files, each sorted case-insensitively. */
export function sortEntries(entries: readonly FsEntry[]): FsEntry[] {
  return [...entries].sort((a, b) =>
    a.dir === b.dir ? byName(a, b) : a.dir ? -1 : 1
  )
}

/** The entries to list: sorted, hidden ones only on request. */
export function visibleEntries(
  entries: readonly FsEntry[],
  showHidden: boolean
): FsEntry[] {
  return sortEntries(entries.filter((e) => showHidden || !e.hidden))
}

/** `selected` with `path` added or removed. */
export function toggleSelected(
  selected: readonly string[],
  path: string
): string[] {
  return selected.includes(path)
    ? selected.filter((p) => p !== path)
    : [...selected, path]
}

/** Query string path of the `fs` request (the path is optional). */
export const fsRequestPath = (path?: string): string =>
  path ? `/api/fs?path=${encodeURIComponent(path)}` : "/api/fs"

import { useCallback, useEffect, useState } from "react"
import { Loader2 } from "lucide-react"

import { LoginCard } from "@/components/login-card"
import { Button } from "@/components/ui/button"
import { inTauri } from "@/lib/runner"
import {
  errorMessage,
  getSession,
  logout,
  onUnauthorized,
  type SessionInfo,
} from "@/lib/web-api"

type Gate =
  | { status: "loading" }
  | { status: "error"; message: string }
  | { status: "login" }
  | { status: "ready"; authRequired: boolean }

/**
 * Web mode: shows the login screen until the session is authenticated, then
 * the app (which gets `onLogout` when a login is required). Inside Tauri it
 * renders the app right away.
 */
export function AuthGate({
  children,
}: {
  children: (onLogout: (() => void) | undefined) => React.ReactNode
}) {
  const [gate, setGate] = useState<Gate>(
    inTauri() ? { status: "ready", authRequired: false } : { status: "loading" }
  )
  const [authRequired, setAuthRequired] = useState(false)

  const track = useCallback((request: Promise<SessionInfo>) => {
    request.then(
      (session) => {
        setAuthRequired(session.authRequired)
        setGate(
          session.authRequired && !session.authenticated
            ? { status: "login" }
            : { status: "ready", authRequired: session.authRequired }
        )
      },
      (e) => setGate({ status: "error", message: errorMessage(e) })
    )
  }, [])

  const retry = () => {
    setGate({ status: "loading" })
    track(getSession())
  }

  useEffect(() => {
    if (inTauri()) return
    track(getSession())
    // Any 401 (expired session, restarted server) goes back to the login.
    return onUnauthorized(() => setGate({ status: "login" }))
  }, [track])

  if (gate.status === "ready") {
    const onLogout = gate.authRequired
      ? () => {
          void logout().catch(console.error)
          setGate({ status: "login" })
        }
      : undefined
    return <>{children(onLogout)}</>
  }
  if (gate.status === "login") {
    return (
      <LoginCard
        onLoggedIn={() => setGate({ status: "ready", authRequired })}
      />
    )
  }
  return (
    <div className="flex h-full items-center justify-center gap-3 text-sm text-muted-foreground">
      {gate.status === "loading" ? (
        <Loader2 className="size-5 animate-spin" />
      ) : (
        <>
          <span>Cannot reach the server: {gate.message}</span>
          <Button size="sm" variant="outline" onClick={retry}>
            Retry
          </Button>
        </>
      )}
    </div>
  )
}

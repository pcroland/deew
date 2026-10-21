"use client"

import { useEffect } from "react"

const DEBUG_EVENT_NAME = "ctui:debug"
const EXTERNAL_PROTOCOLS = new Set(["http:", "https:", "mailto:", "tel:"])

function isModifiedClick(event: MouseEvent) {
  return (
    event.defaultPrevented ||
    event.button !== 0 ||
    event.metaKey ||
    event.ctrlKey ||
    event.shiftKey ||
    event.altKey
  )
}

function shouldOpenExternally(anchor: HTMLAnchorElement) {
  const href = anchor.getAttribute("href")

  if (!href || href.startsWith("#") || anchor.hasAttribute("download")) {
    return false
  }

  try {
    const url = new URL(anchor.href, window.location.href)

    if (!EXTERNAL_PROTOCOLS.has(url.protocol)) {
      return false
    }

    if (url.protocol === "mailto:" || url.protocol === "tel:") {
      return true
    }

    return url.origin !== window.location.origin
  } catch {
    return false
  }
}

async function openExternalLink(href: string) {
  const { openUrl } = await import("@tauri-apps/plugin-opener")
  await openUrl(href)
}

function emitExternalLinkDebugEvent(href: string) {
  if (typeof window === "undefined") {
    return
  }

  window.dispatchEvent(
    new CustomEvent(DEBUG_EVENT_NAME, {
      detail: {
        id: crypto.randomUUID(),
        kind: "external-link",
        href,
        timestamp: new Date().toISOString(),
      },
    })
  )
}

export function ExternalLinkGuard() {
  useEffect(() => {
    function handleClick(event: MouseEvent) {
      if (isModifiedClick(event)) {
        return
      }

      const target = event.target

      if (!(target instanceof Element)) {
        return
      }

      const anchor = target.closest("a[href]")

      if (!(anchor instanceof HTMLAnchorElement)) {
        return
      }

      if (!shouldOpenExternally(anchor)) {
        return
      }

      event.preventDefault()
      emitExternalLinkDebugEvent(anchor.href)
      void openExternalLink(anchor.href)
    }

    document.addEventListener("click", handleClick, true)

    return () => {
      document.removeEventListener("click", handleClick, true)
    }
  }, [])

  return null
}

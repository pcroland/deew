import { StrictMode } from "react"
import { createRoot } from "react-dom/client"

import "./index.css"
import { initTheme } from "./lib/theme.ts"
import App from "./App.tsx"
import { AuthGate } from "./components/auth-gate.tsx"
import { ExternalLinkGuard } from "./components/external-link-guard.tsx"

initTheme()

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <ExternalLinkGuard />
    <main data-ui-scroll-container>
      <AuthGate>{(onLogout) => <App onLogout={onLogout} />}</AuthGate>
    </main>
  </StrictMode>
)

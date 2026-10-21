// Apply the OS theme before first paint (mirror of src/lib/theme.ts), and
// stop iOS Safari from auto-zooming when a <16px input is focused.
// A separate classic script (not inline) so a strict CSP can allow it.
;(function () {
  document.documentElement.classList.toggle(
    "dark",
    window.matchMedia("(prefers-color-scheme: dark)").matches
  )

  // iOS only: Android would lose pinch zoom with maximum-scale=1, iOS keeps it.
  var ios =
    /iPad|iPhone|iPod/.test(navigator.userAgent) ||
    (navigator.platform === "MacIntel" && navigator.maxTouchPoints > 1)
  var meta = document.querySelector('meta[name="viewport"]')
  if (ios && meta) {
    meta.setAttribute(
      "content",
      meta.getAttribute("content") + ", maximum-scale=1"
    )
  }
})()

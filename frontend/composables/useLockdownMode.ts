// "Mode Terkunci" (focus/lockdown) enforcement for the Simulasi TO player — see backend
// `domain::simulation::SimulationRun::lockdown_enabled` doc comment for the scoping rationale
// (Simulasi TO only, never Drilling/Latihan/Mini). Purely client-side deterrence + detection:
// nothing here can make a determined, technically-savvy student's browser un-hackable — the
// goal is to make casual cheating (switching tabs, copying a question, opening devtools)
// inconvenient and, more importantly, LOGGED for Admin/Sekolah review afterward.
//
// Two moving parts:
//  1. A fullscreen "gate" — the Fullscreen API requires a user gesture, so lockdown can't be
//     force-entered on page load; `showGate` drives an overlay with a "Mulai" button that calls
//     `enterFullscreen()`.
//  2. A set of document/window listeners (tab switch, window blur, fullscreen exit, right-click,
//     copy/cut, common devtools/print/save shortcuts, page-leave) that call `onViolation` with a
//     short event_type string — the caller decides how to log/report/throttle it.
export interface UseLockdownModeOptions {
  /** Whether this run has lockdown enabled at all (immutable snapshot from the server). */
  enabled: Ref<boolean>
  /** Whether enforcement should currently be armed — false while loading/finished/on a break
   *  screen the product decided not to lock, true while the run is genuinely in progress. */
  active: Ref<boolean>
  onViolation: (eventType: string, detail?: string) => void
}

export function useLockdownMode(opts: UseLockdownModeOptions) {
  const isFullscreen = ref(false)
  const showGate = ref(false)

  function armed() {
    return opts.enabled.value && opts.active.value
  }

  function updateFullscreenState() {
    isFullscreen.value = !!document.fullscreenElement
  }

  async function enterFullscreen() {
    try {
      await document.documentElement.requestFullscreen()
    } catch {
      // Some browsers/embedded contexts (e.g. an iframe without allow="fullscreen") reject
      // programmatic fullscreen entirely — degrade gracefully. The other restrictions (hidden
      // nav, blocked copy/right-click/shortcuts, tab-switch/blur detection) still apply even
      // without real fullscreen, so lockdown isn't a total no-op here.
    }
    updateFullscreenState()
    showGate.value = false
  }

  function handleVisibility() {
    if (!armed()) return
    if (document.hidden) opts.onViolation('tab_switch', 'document.hidden')
  }
  function handleBlur() {
    if (!armed()) return
    opts.onViolation('window_blur')
  }
  function handleFullscreenChange() {
    updateFullscreenState()
    if (!armed()) return
    if (!document.fullscreenElement) {
      opts.onViolation('fullscreen_exit')
      showGate.value = true
    }
  }
  function handleContextMenu(e: MouseEvent) {
    if (!armed()) return
    e.preventDefault()
    opts.onViolation('context_menu_attempt')
  }
  function handleCopyOrCut(e: ClipboardEvent) {
    if (!armed()) return
    e.preventDefault()
    opts.onViolation('copy_attempt', e.type)
  }
  function handleKeydown(e: KeyboardEvent) {
    if (!armed()) return
    const key = e.key.toUpperCase()
    const isDevtools =
      key === 'F12' ||
      ((e.ctrlKey || e.metaKey) && e.shiftKey && ['I', 'J', 'C'].includes(key)) ||
      ((e.ctrlKey || e.metaKey) && e.altKey && ['I', 'J', 'C'].includes(key))
    const isViewSourceOrPrintOrSave = (e.ctrlKey || e.metaKey) && ['U', 'P', 'S'].includes(key)
    if (isDevtools || isViewSourceOrPrintOrSave) {
      e.preventDefault()
      opts.onViolation('devtools_attempt', e.key)
    }
  }
  function handleBeforeUnload(e: BeforeUnloadEvent) {
    if (!armed()) return
    e.preventDefault()
    e.returnValue = ''
  }

  // Best-effort browser-Back deterrent: while armed, a history entry is kept pushed on top so
  // the Back button fires `popstate` instead of actually navigating away — we immediately push
  // it right back and log a violation. Not foolproof (nothing client-side can be), but stops
  // the casual case.
  function handlePopState() {
    if (!armed()) return
    history.pushState(null, '', location.href)
    opts.onViolation('back_navigation_attempt')
  }

  function start() {
    document.addEventListener('visibilitychange', handleVisibility)
    window.addEventListener('blur', handleBlur)
    document.addEventListener('fullscreenchange', handleFullscreenChange)
    document.addEventListener('contextmenu', handleContextMenu)
    document.addEventListener('copy', handleCopyOrCut)
    document.addEventListener('cut', handleCopyOrCut)
    document.addEventListener('keydown', handleKeydown)
    window.addEventListener('beforeunload', handleBeforeUnload)
    window.addEventListener('popstate', handlePopState)
    if (opts.enabled.value) history.pushState(null, '', location.href)
    updateFullscreenState()
    if (opts.enabled.value && !document.fullscreenElement) showGate.value = true
  }

  function stop() {
    document.removeEventListener('visibilitychange', handleVisibility)
    window.removeEventListener('blur', handleBlur)
    document.removeEventListener('fullscreenchange', handleFullscreenChange)
    document.removeEventListener('contextmenu', handleContextMenu)
    document.removeEventListener('copy', handleCopyOrCut)
    document.removeEventListener('cut', handleCopyOrCut)
    document.removeEventListener('keydown', handleKeydown)
    window.removeEventListener('beforeunload', handleBeforeUnload)
    window.removeEventListener('popstate', handlePopState)
    if (document.fullscreenElement) {
      document.exitFullscreen().catch(() => {})
    }
  }

  return { isFullscreen, showGate, enterFullscreen, start, stop }
}

---
name: verify
description: Build, launch, and drive momPanel on Windows to verify UI/behavior changes end-to-end (screenshots + CDP).
---

# Verifying momPanel on Windows

## Gotchas that will waste your time

- **Kill the installed app first.** `C:\Program Files\momPanel\app.exe` autostarts at
  login. The single-instance plugin (keyed by identifier, shared with dev builds)
  makes `tauri dev` silently surface the INSTALLED app and exit — you'll be
  screenshotting 0.6.x thinking it's your build. `Stop-Process -Name app` first.
- **`target\debug\app.exe` alone shows a connection error.** Dev builds bake in the
  dev-server URL; the binary only works while `npm run tauri dev` is running.
- **Config is shared** between dev and installed builds:
  `%APPDATA%\momPanel\config.json` (`ui_scale`, `experimental_ui`,
  `companion_bg_opacity`, `window_x/y`). Note what you change; restore when done.
- Warp terminal tabs can also be titled "momPanel" — match windows by process
  (`Get-Process app`), never by title alone.
- The memory-warning banner/dialog can pop mid-test on a busy machine and steal
  `MainWindowTitle`. Dismiss it before relying on title lookups.

## Launch + drive

```powershell
# CDP gives you Runtime.evaluate inside the page (withGlobalTauri is on):
$env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = "--remote-debugging-port=9222"
npm run tauri dev
```

- List targets: `GET http://127.0.0.1:9222/json`; drive via WebSocket
  `Runtime.evaluate` (Node's global WebSocket works; no deps needed).
- Switch modes/scales without clicking:
  `window.__TAURI__.core.invoke('set_config', { cfg: { experimental_ui: false, ui_scale: 'normal' } })`
  then `location.reload()`. The arg key is `cfg`, not `patch`.
- Direct window JS APIs (`unminimize()` etc.) are blocked by capabilities;
  `invoke('open_main_window')` is the allowed restore path.
- Open the About panel: `document.querySelector('[data-info]').click()`.

## What to look at

- Classic grid at all three scales (normal/big/biggest): no tile content may
  overlap; Memory/Storage/CPU rings should be the same size.
- Companion mode at Companion background = Invisible: compare a screenshot of the
  window region against the same region with the app minimized — they must match
  (no tint, no ghost title bar).
- About panel in BOTH modes (companion's window is much smaller; columns reflow).

Screenshot approach: `System.Drawing` `CopyFromScreen` around the window rect
(`GetWindowRect` on the app process's `MainWindowHandle`), pad ~30px so frame
artifacts are visible.

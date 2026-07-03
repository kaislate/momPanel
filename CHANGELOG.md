# Changelog

All notable changes to momPanel. Dates are YYYY-MM-DD.

## 0.4.0 — 2026-07-03

### Added
- **High-memory warning banner.** A background watcher (`src-tauri/src/memwatch.rs`)
  polls RAM every 2s on its own thread — independent of the frontend tile loop, which
  pauses when the window is hidden — and reveals a pre-created, always-on-top, borderless
  banner window (`memwarn`, `src/warn.html` + `src/warn.js`) when usage crosses the
  threshold. Hidden again on recovery, with a 7-point hysteresis band to prevent flicker.
  A **Dismiss** button (`dismiss_mem_warn` command) suppresses it until memory recovers
  and spikes again. The banner names the biggest memory consumer (top process by RSS via
  sysinfo) so the user knows what to close.
- **Settings** (About panel): a "Warn me about high memory" toggle, a threshold selector
  (70–90% in 5% steps), and a banner **color** picker (text auto-contrasts). Persisted as
  `mem_warn_enabled` / `mem_warn_percent` / `mem_warn_color` in config.
- Capability `memwarn` scoping the banner window (event + window hide/close).

### Changed
- Closing the **main** window now explicitly quits the app (the persistent hidden banner
  window would otherwise keep the process alive).

## 0.3.5 — 2026-06-21

### Added
- **Windows volume** via Core Audio (`IAudioEndpointVolume` on the default playback
  device); the `windows` crate is a Windows-only dependency.

### Fixed
- **Windows Wi-Fi:** Windows 11 blocks `netsh` SSID/signal without Location permission.
  We now fall back to the connection profile's name (no Location needed); `signal_percent`
  is optional and the tile shows the SSID + "Connected" when strength is unknown (the full
  arcs return once Location is enabled).

## 0.3.4 — 2026-06-21

### Added
- **Windows Wi-Fi collector** (`netsh wlan show interfaces`) and **Windows printers
  collector** (PowerShell / `Win32_Printer`) — Wi-Fi and printers now report on Windows,
  not just Linux. Subprocesses use `CREATE_NO_WINDOW` to avoid console flashes.
- `os_info` command + a "Running on …" line in the About panel (runtime OS detection).

## 0.3.3 — 2026-06-21

### Added
- In-app **"What's New"** popup shown once after the app updates to a new version
  (tracked via `last_seen_version` in config); skipped on a fresh install.
- A **"What's New"** button in the About panel to re-open the latest update note.
- Plain-English in-app changelog (`src/changelog.js`); GitHub release notes are now
  sourced from this `CHANGELOG.md` by the release workflow.

## 0.3.2 — 2026-06-21

### Changed
- Weather now returns a **7-day** forecast (was 5); forecast rows use smaller fonts and
  a narrower day column so day descriptions and precip-% fit without ellipsis truncation.

### Fixed
- About-panel checkbox labels wrap instead of overflowing the card on Linux.

## 0.3.1 — 2026-06-21

### Fixed
- **Linux input/rendering:** set `WEBKIT_DISABLE_DMABUF_RENDERER=1` at startup to work
  around the WebKitGTK DMABUF bug that caused glitchy rendering and flaky clicking on
  many Wayland/Mesa setups (notably from AppImages).
- Dropped `will-change: transform` on tiles (could interfere with WebKitGTK hit-testing).

### Added
- **"Start automatically when I log in"** toggle in the About panel (`get_autostart` /
  `set_autostart`). Autostart now enables only once on first run, then respects the
  user's choice instead of force-enabling every launch.

## 0.3.0 — 2026-06-20

### Added
- First public release: a frameless, graphics-first desktop info panel with tiles for
  clock, date, CPU, memory, storage, Wi-Fi, internet, printers, volume, and a weather
  forecast.
- Per-tile "?" help, a "make everything bigger" size control, an eye toggle to hide
  controls, safe "open settings" shortcuts, and a first-run ZIP prompt (stored locally).
- Signed auto-update from GitHub Releases; autostart on login; Linux (AppImage/deb/rpm)
  and Windows (NSIS/MSI) builds via CI.

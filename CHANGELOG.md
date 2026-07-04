# Changelog

All notable changes to momPanel. Dates are YYYY-MM-DD.

## 0.4.1 — 2026-07-04

### Fixed
- **About panel layout.** The panel now widens to 480px and scrolls (`max-height: 85vh`),
  so the settings added in 0.4.0 (alert + Theme) are reachable — the Theme section was
  previously below the fold in a fixed 320px, non-scrolling card. Also fixed the checkbox
  rows: the generic `.modal-card input` rule (for the first-run ZIP field) was stretching
  `.info-auto` checkboxes to 100% width and pushing their labels off-panel. Scoped
  overrides restore correct sizing: `.modal-card.info-card` (width + scroll),
  `.info-card input[type="checkbox"]`, and `margin: 0` on the `.info-row` color swatch.

## 0.4.0 — 2026-07-03

### Added
- **High-memory alert.** A background watcher (`src-tauri/src/memwatch.rs`) polls RAM
  every 2s on its own thread — independent of the frontend tile loop, which pauses when
  the window is hidden — and fires a **native critical desktop notification**
  (`notify-send -u critical`, reliable on GNOME/Wayland where a client window cannot pin
  itself on top). Backend-driven audio: an **alert tone** (`canberra-gtk-play`) and a
  **spoken warning** naming the top memory consumer (`spd-say`, e.g. "Memory usage high.
  Opera is using 4.3 gigabytes."), played at a **60% volume floor** (`wpctl`, boosts if
  low, never lowers, restores after). A pure, tested escalation state machine
  (`memwatch::advance`): trigger → **pulse** every ~30s → **centered modal** (`memwarn`,
  `src/warn.html`/`warn.js`) if still unacknowledged. 7-point hysteresis; **Dismiss**
  (`dismiss_mem_warn`) suppresses until recovery; **Open momPanel** (`open_main_window`).
- **Theming / personalization.** Curated color slots via CSS variables — accent,
  background, tile, and a 3-color gauge palette (`gauge.js` refactored to read
  `--gauge-ok/warn/bad`). Text (`--ink`) auto-contrasts from background luminance so a
  light background stays readable. Named presets (Midnight/Warm/High-contrast) + per-slot
  color pickers + reset in the About panel; applied on boot via `src/theme.js`; persisted
  as a `theme` object in config.
- **Settings** (About panel): warn on/off, threshold 70–90% (5% steps), sound on/off +
  **tone picker** + **volume-floor** selector, speech on/off, pulse on/off, escalation
  on/off, and an **alert-dialog color** (styles the escalation modal, auto-contrast text).
  Persisted as `mem_warn_*` fields.
- Capability `memwarn` scoping the modal window (event + window hide/close).

### Changed
- Closing the **main** window now explicitly quits the app (the persistent hidden modal
  window would otherwise keep the process alive).

### Notes
- Linux is the v1 target for the alert's audio/notification; Windows toast + TTS are a
  follow-up. The updater/AppImage pipeline is unchanged.

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

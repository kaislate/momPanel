# Changelog

All notable changes to momPanel. Dates are YYYY-MM-DD.

## 0.6.2 — 2026-07-14

### Fixed
- **Real transparency on Linux.** 0.6.1's diagnosis was wrong: buttons stayed dead
  with an opaque window, so transparency never broke input. The actual artifact
  source is the 0.3.1-era `WEBKIT_DISABLE_DMABUF_RENDERER=1` workaround: on modern
  WebKitGTK (Zorin 18 / 2.48+) that forces a legacy render path that can't composite
  window alpha (black instead of see-through — the "Invisible shows black" report)
  and ghosts stale frames. The env override is now **opt-in**
  (`MOMPANEL_LEGACY_RENDERER=1`), `tauri.linux.conf.json` is removed (window
  transparent everywhere again), and `supports_transparency` returns true (the
  simulated-wallpaper fallback path remains in the frontend, dormant).
  Verified locally that the platform-config merge DOES apply (mirror test on
  Windows: override → opaque black window), so 0.6.1's Linux window really was
  opaque — matching the black-at-Invisible report.
- **Stuck unread badge (red dot) on the dock.** Memory-alert pulses spawned a new
  `critical` notification every ~30s with no id tracking; critical notifications
  never expire, so they piled up in the tray and pinned an unclearable badge.
  `notify-send -p -r <id>` now maintains ONE tracked notification per episode, and
  recovery/dismiss retract it via `CloseNotification` (safe to over-call).
- **Wallpaper fallback correctness** (dormant path): `picture-uri-dark` can point at
  a nonexistent file (seen on the target machine) — candidates are now
  existence-checked so a stale dark URI falls through to the light one; SVG
  wallpapers (Zorin defaults) gained a MIME mapping.

### Added
- `shortcuts.log` field trace (config dir, 20 KB cap): every `open_settings` press
  logs invoke + spawn result, so the still-open "Linux buttons do nothing" report
  can be pinpointed to frontend-vs-backend with one click on the real machine. The
  render-path fix above is itself a plausible cure (legacy-renderer input bugs).

## 0.6.1 — 2026-07-14

### Fixed
- **Linux buttons work again.** 0.6.0's `transparent: true` window broke input and
  produced ghost/stale-frame artifacts on Linux/WebKitGTK (known upstream:
  tauri-apps/tauri#14924, #13157 — the "you can see the About panel behind the sky"
  report was a stale-buffer ghost). Diagnosed on the target machine: spawning
  `gnome-control-center`/`gnome-disks` with the app's exact env works fine, so the
  webview, not the backend, was eating the clicks. A new `tauri.linux.conf.json`
  keeps the window **opaque on Linux only**; Windows/macOS keep real transparency.

### Added
- **Simulated see-through on Linux.** New `desktop_background` command returns the
  wallpaper (GNOME `picture-uri`, Windows registry, macOS System Events) as a data
  URL, cached by path+mtime; `supports_transparency` tells the frontend which mode
  the OS gets. In companion mode on Linux the wallpaper is drawn as the bottom
  layer, so "clear" skies reveal the desktop — and never other windows. A new
  **"Invisible — just the desktop"** option (opacity 0, backend clamp now 0.0–1.0)
  shows only the content over the desktop on every platform.
- **Peek cards.** Hovering a companion health row (Wi-Fi, Printer, Sound, Speed,
  Space, Internet) shows the corresponding classic tile — same renderer, same live
  data, working buttons — as a popover beside the card (200ms intent delay,
  Escape/mouse-away dismiss, reduced-motion honored).
- **Ink levels.** The printers collector (Linux/macOS) queries the default queue's
  IPP marker attributes via `ipptool` (pure `parse_marker_attrs` + fixture tests;
  verified against the real ET-3760: K 49 / C 77 / M 80 / Y 80, low at 15). The
  Printers tile renders per-color ink bars with a "running low" note, and
  companion's Printer row goes amber "Ink low" with a calm attention card.

## 0.6.0 — 2026-07-13

### Added
- **macOS support (best-effort tier, like Windows).** `open_settings` gains a macOS
  branch (`open x-apple.systempreferences:` extension URLs for wifi/sound/printers,
  Finder for storage) with the same allow-list pattern; `open_github` gains an `open`
  branch. Collectors: printers widened to `any(linux, macos)` (macOS ships CUPS — same
  `lpstat` output, device-URI probe included), volume via `osascript` (pure
  `parse_osascript` + fixture tests), internet uses the shared cached TCP probe,
  Wi-Fi stays a calm Unavailable (no reliable non-deprecated CLI). Release workflow
  builds a universal macOS bundle (`--target universal-apple-darwin`; minisign-signed,
  not Apple-notarized — first launch is right-click → Open). CI adds a `cargo check`
  job on macos-latest.

### Changed
- **Controls corner redesign.** The always-visible i/?/eye/A−/A+ pill is now a single
  faint gear (35% opacity at rest) that expands into the tray on demand and collapses
  on click-away — controls no longer crowd or overlap the tiles.
- **About panel is one wide pane.** `min(900px, 94vw)`, identity header with slim
  action buttons and an ✕ close, three columns (General / Memory alerts / Appearance);
  every setting visible with no scrolling. The Companion toggle lives under General.
- **Companion mode window is right-sized.** New `setPanelBase()` in scale.js lets the
  mode pick its canvas: 880×560 (vs the grid's 1100×680), with spacing/typography
  tuned to match and the attention row keeping clear of the controls corner. Late-night
  hours now greet with "Good night" instead of "Good evening".
- **Companion sky transparency.** The window is now created `transparent: true`
  (+ `macOSPrivateApi`/`macos-private-api` feature for mac); companion's time-of-day
  gradient moved to a `body::before` backdrop layer whose opacity is user-tunable
  (About → General → "Companion background", Solid→Very clear). Persisted as
  `companion_bg_opacity` (backend-clamped to 0.2–1.0 so the panel can't vanish);
  applied live. The classic grid keeps its opaque background — no visual change there.
- **Graceful attention cards.** The companion attention row animates open/closed
  (max-height + margin transition on the grid's auto row), so the hero/health content
  glides to make room instead of snapping; resolved cards collapse first and are
  removed from the DOM after the transition. Honors `prefers-reduced-motion`.
- In-app changelog gained the missing 0.5.1 entry (its What's New popup previously
  showed the generic fallback).

## 0.5.1 — 2026-07-10

### Fixed
- **Printer no longer shows "ready" when it's powered off.** A *permanent* CUPS queue
  (a driverless `ipp://` queue backed by ipp-usb — used so the printer is always
  available to print to) always reports `idle`, so the Printers collector always
  reported "ready" even with the printer physically off. On Linux, `read()` now looks
  up each queue's device URI via `lpstat -v` and, for network backends
  (`ipp`/`ipps`/`http`/`socket`), TCP-probes the endpoint; a printer reported "ready"
  whose endpoint is unreachable is downgraded to "offline". ipp-usb only listens on its
  port while the printer is on, so the probe doubles as a reliable presence check. Local
  backends (`usb://`, `hp:/`, …) are left as-is, and `out_of_paper`/`disabled` states are
  never overridden. Adds helpers `device_uris()`, `network_endpoint()` (unit-tested), and
  `endpoint_reachable()` in `collectors/printers.rs`.

## 0.5.0 — 2026-07-08

### Added
- **Companion mode (experimental preview).** A reimagined, status-by-exception panel
  (`src/preview/companion.js`/`.css`): a greeting and giant clock, the weather as the
  hero with a 4-day strip, and a single "All is well" health card (Internet / Wi-Fi /
  Printer / Sound / Speed / Space) that surfaces plain-language attention cards — with
  the right settings button — only when something needs attention. The background sky
  shifts with the time of day. Toggled in **About → Preview** (`experimental_ui` in
  config; the app reloads on switch). Same collectors, same cadences; DOM is patched
  in place, never rebuilt on a timer.
- **Remembered window position.** `WindowEvent::Moved` persists the outer position
  (throttled to ≥2s, flushed on close) to new `window_x`/`window_y` config fields and
  restores it via `set_position` before the now-created-hidden window is shown.
  Best-effort on GNOME Wayland (the compositor ignores client positioning); effective
  on X11 and Windows.
- **Single instance.** `tauri-plugin-single-instance`, registered first in the builder;
  a second launch shows/unminimizes/focuses the running panel instead of spawning a
  duplicate process — important for an app that autostarts on login.
- **CI.** `.github/workflows/ci.yml` runs `cargo test` + `cargo check` on PRs and
  pushes to main — the first automated Linux compile check for `cfg(linux)` code.

### Fixed
- **Locale-proof collectors.** Every parsing subprocess spawn (`nmcli`, `lpstat`,
  `wpctl`) now pins `LC_ALL=C`/`LANG=C`. Previously the parsers matched English
  literals, so on any non-English system Printers listed nothing and Wi-Fi read
  "unavailable" while connected.
- **Stale-response race.** The tile loop tags each fetch with a generation and
  discards out-of-order responses — a slow 20s poll can no longer repaint "Offline"
  over a fresh D-Bus-pushed "Online" after a reconnect.
- **First-run ZIP prompt lifecycle.** Opening a new prompt cleanly resolves the
  previous one (no hung promise, no leaked document keydown listener), and the
  automatic prompt fires once per session instead of on every window focus after a
  cancel. The explicit set/change-location affordances still always prompt.
- **Modal hygiene.** A shared `src/modal.js` closes the active overlay before another
  opens (no stacked keydown listeners), and `.modal-backdrop` now layers above the
  bottom-right control pill (the "i" button was clickable through the backdrop).
- **Clock/date rebuild churn.** Both tiles patch their DOM in place instead of
  innerHTML-rebuilding every second — the analog/digital toggle keeps focus and hover,
  and the month calendar rebuilds once a day, not 86,400 times. The analog second hand
  is omitted under `prefers-reduced-motion` (previously claimed, not implemented).
- **Weather robustness.** A failed refresh keeps the last good forecast on screen with
  an "as of" time; the ZIP geocode is cached in config and Open-Meteo's geocoding API
  backs up zippopotam.us (no longer a single point of failure); one shared reqwest
  client instead of a fresh TLS setup per fetch; a missing ZIP shows a friendly
  "Set up weather" button instead of a dead tile.
- **Memwatch efficiency.** The watcher reloads config only when a save-generation
  counter changes (was: read + parse `config.json` from disk every 2 seconds,
  forever). Config writes are atomic (tmp + rename) and serialized so overlapping
  saves can't drop a patch or expose half-written JSON. Alert tone/speech play on a
  detached thread so the poll loop keeps ticking during playback.
- **Internet probe.** The TCP reachability verdict is cached for 20s and tries 443
  before 53 (some networks block outbound DNS to Cloudflare).
- Gauges no longer render a stray colored dot at 0% (round linecap on an empty arc);
  storage reports **decimal GB** to match GNOME Files/Disks; the What's New card caps
  its height and scrolls (a long release note pushed "Got it" off-screen); theme
  colors — including High contrast — now reach the internet globe, Wi-Fi arcs, and
  printer dots; muted volume shows a big crossed speaker and "Sound is off"; the
  size control clamps to the monitor (font + window shrink together) and no longer
  re-centers the window on a size change.

### Changed
- Dead `src/mock.js` removed (it was imported nowhere); `package.json` version aligned
  with the app version; `libappindicator3-dev` dropped from the release workflow (no
  tray icon).

## 0.4.3 — 2026-07-04

### Added
- **Weather fallback provider.** When Open-Meteo is unreachable, the collector now falls
  back to the **US National Weather Service** (`api.weather.gov`, no key, US-only).
  `weather.rs` splits the forecast into `forecast_open_meteo()` (primary) and
  `forecast_nws()` (fallback); NWS day/night periods are aggregated into per-date
  highs/lows and its `shortForecast` text is mapped to WMO codes via `nws_code()`, so both
  providers flow through the same `condition()`/icon path. Adds a `User-Agent` (required by
  NWS). Geocoding (zippopotam.us) is unchanged.

### Fixed
- Weather no longer blanks the tile during an Open-Meteo outage (as happened 2026-07-04,
  when open-meteo's API was unreachable network-wide while everything else was healthy).

## 0.4.2 — 2026-07-04

### Fixed
- **App icons.** Replaced the default Tauri placeholder icons with the momPanel logo
  (source `assets/app-icon.png`, rendered from `assets/logo.svg`; regenerated via
  `tauri icon`), so the AppImage/deb/rpm bundles and the window carry the real brand mark
  instead of the Tauri/Vite default.
- **Settings dropdowns rendered white-on-white.** WebKitGTK was drawing native `<select>`
  controls (and their option popups) in the system light theme. Added `color-scheme: dark`
  on `:root`, a custom inline-SVG chevron, and explicit `.info-row select option` colors so
  the dropdowns are dark and readable and no longer look generic.

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

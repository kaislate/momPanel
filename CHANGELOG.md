# Changelog

All notable changes to momPanel. Dates are YYYY-MM-DD.

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

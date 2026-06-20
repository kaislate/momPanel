# momPanel — Desktop Info Panel — Design Spec

**Date:** 2026-06-20
**Status:** Approved design, pending spec review

## Summary

momPanel is a graphics-first desktop information panel for non-technical users
(the "mom" who finds control panels intimidating). It displays system status —
time, date, memory, Wi-Fi, internet connectivity, storage, printers, volume, and
weather — as large, friendly graphic tiles in a frameless window that cannot be
accidentally moved or closed. Each actionable tile offers a big friendly button
that opens the correct native settings screen (display + safe shortcuts; never
performs dangerous actions directly).

**Primary target:** Linux / Zorin OS (fully supported).
**Secondary target:** Windows 11 (best-effort, for development/testing only — must
run well enough to test UI and layout on the dev machine, not feature-complete).

## Goals

- Glanceable, calm, beautiful status display prioritizing graphics over text.
- Genuinely useful to someone who cannot navigate control panels.
- Low, honest resource footprint for an always-running app (it shows memory usage,
  so it must not be a memory hog itself).
- Smooth hover animations and an elegant startup animation.
- Auto-updates from GitHub Releases once installed.
- Starts automatically on login.

## Non-Goals (YAGNI)

- No direct system mutation (no connecting to Wi-Fi, deleting print jobs, etc.) —
  display + "open the right settings screen" only.
- No Battery tile for the first version (skipped by user; trivial to add later and
  can auto-hide on desktops).
- No Windows feature parity — Windows is test-only with graceful fallbacks.
- No multi-window / multi-monitor management beyond a single panel window.

## Framework & Stack

- **Tauri** (Rust backend + web frontend). Chosen for low always-on RAM (~30–80 MB
  via the system webview — WebKitGTK on Zorin) versus Electron (~80–200 MB, bundles
  Chromium), while still building cross-platform from one codebase and using the same
  web frontend for visuals.
- **Frontend:** HTML/CSS/JS. Owns ALL visuals — tile grid, gauges, graphics, hover
  animations, startup animation. Knows nothing about the OS.
- **Backend:** Rust. A set of small, independent **collector modules**, each
  answering one question and returning a small uniform data shape. The `sysinfo`
  crate provides in-process memory/storage reads.

## Architecture

Two clean layers with a narrow interface between them.

### Frontend (the panel)
- Renders a wide dashboard grid of big graphic tiles.
- Requests current stats from the backend; renders them with smooth transitions.
- Never sees OS differences — only uniform data shapes.

### Backend (the collectors)
One collector per concern, each with a Linux implementation (real) and a Windows
implementation (best-effort). The frontend and data shapes are identical across
platforms — only the inside of each collector differs. Adding/fixing a platform
never touches the UI.

Collectors:
- `memory` — in-process via `sysinfo`.
- `storage` — in-process via `sysinfo`.
- `wifi` — Linux: `nmcli` + NetworkManager D-Bus signals.
- `internet` — connectivity check + NetworkManager D-Bus signals.
- `printers` — Linux: CUPS / `lpstat`.
- `volume` — Linux: PipeWire `wpctl` + D-Bus events.
- `weather` — Open-Meteo HTTP API.
- `clock`/`date` — frontend-only (browser clock); no backend collector.

### Data flow
Frontend updates on a mix of **event-driven pushes** (D-Bus) and **timed polls**,
tuned to minimize subprocess spawns (see Resource Strategy). If a collector fails
(e.g. no printer connected), its tile shows a calm "not available" state rather
than breaking the panel.

## Window Behavior

- Single, frameless, non-resizable, non-movable window (cannot be accidentally
  dragged or closed).
- Wide dashboard form factor with big graphic tiles in a grid.
- Pauses polling when hidden/minimized or when the screen is locked.

## Tiles

| Tile | Graphics-first display | Linux source | Shortcut button |
|---|---|---|---|
| Clock | Toggle analog/digital; analog face shows all 12 numbers; mode remembered | browser clock | — |
| Date | Day-of-week + date, calendar-style | browser clock | — |
| Memory | Ring/arc gauge, % RAM used, color-coded | `sysinfo` | — |
| Storage | "How full" bar/donut, free GB | `sysinfo` | Open Files/Disks |
| Wi-Fi | Signal-strength arcs + network name | `nmcli` / D-Bus | Open Wi-Fi settings |
| Internet | Big green "Online" / red "Offline" globe | connectivity check / D-Bus | — |
| Printers | Printer icon per printer + status (ready/out of paper/offline); default highlighted | CUPS / `lpstat` | Open Printers settings |
| Volume | Speaker icon + level arc, muted state | `wpctl` / D-Bus | Open Sound settings |
| Weather | Condition icon + temp, today's high/low | Open-Meteo | (change location) |

### Shortcut buttons (safe, display-adjacent)
Each relevant tile has a big friendly button launching the correct native settings
screen. On Zorin these are GNOME Settings panels, e.g.:
- Wi-Fi: `gnome-control-center wifi`
- Printers: `gnome-control-center printers`
- Sound: `gnome-control-center sound`
- Storage: file manager / Disks

### Clock details
- User-facing toggle between analog and digital.
- Analog face renders all twelve numbers.
- Selected mode persisted in config.

### Weather details
- **First run prompts for a ZIP code**, stored in config and reused thereafter.
- ZIP → coordinates via a free no-key service (Zippopotam.us), then Open-Meteo for
  the forecast (Open-Meteo needs no API key — important since this installs on
  someone else's machine).
- A small "change location" affordance allows updating the ZIP later.

## Resource Strategy (low footprint)

The cost driver for always-on panels is repeatedly **spawning subprocesses**, not
reading values. Strategy minimizes spawns:

1. **Clock & Date cost nothing** — frontend reads the browser clock in JS; no
   backend call, no polling.
2. **Cheap in-process reads run often; expensive subprocess reads run rarely.**
   Memory/storage come from `sysinfo` inside our process (microseconds, no
   subprocess). Anything that shells out runs on a slow timer.
3. **Event-driven over polling where Zorin allows.** Wi-Fi, internet, and volume
   changes arrive via D-Bus signals (NetworkManager, PipeWire) — instant updates,
   near-zero idle cost. Polling is a fallback only.
4. **Pause when not visible** — hidden/minimized/locked stops polling entirely.

### Cadence

| Tile | Method | Cadence |
|---|---|---|
| Clock / Date | Frontend only | continuous, free |
| Memory / Storage | In-process `sysinfo` | every 3s / 30s |
| Wi-Fi / Internet | D-Bus events (poll fallback) | instant / ~20s fallback |
| Volume | D-Bus event (poll fallback) | instant / ~5s fallback |
| Printers | CUPS (poll) | ~30s |
| Weather | HTTP | ~20min |

## Delivery: Packaging, Auto-Update, Autostart, Startup Animation

### Packaging
- **Linux/Zorin:** ship as an **AppImage** — required for self-update (the `.deb`
  format cannot update itself).
- **Windows:** standard Tauri installer, for dev/test only.

### Auto-update (from GitHub Releases)
- Tauri **updater plugin** checks the project's GitHub Releases on launch.
- Newer versions are downloaded and installed silently.
- Updates are **cryptographically signed** with a keypair so only the project's own
  releases can install. The signing keypair is generated during setup; the public
  key ships in the app, the private key signs releases (kept secret, e.g. in CI).

### Autostart on login
- Tauri **autostart plugin** registers the panel to launch on login (a
  `~/.config/autostart` entry on Zorin; registry Run key on Windows).

### Startup animation
- Single frameless window fades/scales in, then tiles do a **staggered reveal**
  (each tile easing into place in sequence). Pure CSS; identical on both platforms.

### Hover animations
- Pure CSS transitions on tiles; identical cross-platform.

## Error Handling

- Each collector failure is isolated to its tile, which shows a calm "not available"
  state. The rest of the panel keeps working.
- Weather/network errors degrade gracefully (last-known value or a neutral state).
- Missing native tools (e.g. `nmcli` absent) → tile reports unavailable rather than
  crashing.

## Testing

- **Collectors:** each collector is independently testable against a known data
  shape; Linux and Windows implementations tested on their respective platforms.
- **Frontend:** renders from mock data shapes so visuals/animations can be verified
  without a live backend (and on the Windows dev machine).
- **Graceful degradation:** simulate missing tools / failed collectors and confirm
  tiles show "not available" without breaking the panel.
- **Resource check:** confirm idle CPU/RAM stays low (verifies the resource
  strategy), since the panel's own honesty depends on it.

## Open Items / To Confirm During Implementation

- Exact GNOME Settings invocations on the target Zorin version (panel names can vary
  by version) — verify on the real machine.
- D-Bus signal availability for volume on the target PipeWire/Zorin version; poll
  fallback covers gaps.
- Window default size/position for the dashboard (tune on the real screen).

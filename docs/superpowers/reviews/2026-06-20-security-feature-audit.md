# momPanel Audit — Security, Features, Correctness, Performance

**Date:** 2026-06-20
**Method:** Multi-agent workflow (`mompanel-audit`): four parallel reviewers (security,
features, correctness, performance) followed by adversarial verification of every
non-info security finding (each finding handed to a skeptic instructed to refute it).

## Security (verified)

### Confirmed — fix
- **HIGH — Stored XSS via Wi-Fi SSID.** `src/tiles/wifi.js` rendered the system-supplied
  SSID into `innerHTML` without escaping (printers/weather tiles escaped; this was an
  inconsistency, not intent). With `csp: null` and `withGlobalTauri: true`, a crafted
  SSID such as `<img src=x onerror=window.__TAURI__.core.invoke('open_settings',{target:'wifi'})>`
  broadcast by an AP in radio range executes in the webview and can reach every
  registered command. **Verified exploitable.**
  → *Fixed:* escape the SSID (shared `src/escape.js`); strict CSP added; `withGlobalTauri`
  retained but no longer the sole barrier.
- **MEDIUM — No CSP + `withGlobalTauri`.** `csp: null` allowed inline event handlers; the
  global exposed the full IPC surface. Together they upgraded any DOM-injection slip to
  command execution. **Confirmed.**
  → *Fixed:* explicit strict CSP (`default-src 'self'; script-src 'self'; …`).

### Refuted / downgraded (no action or defense-in-depth only)
- **Backend ZIP validation (was medium → info).** Only reachable *after* an XSS, so it is
  defense-in-depth, not a standalone hole. → *Still hardened:* backend now rejects any
  non-5-digit ZIP in `read_weather` and `set_config`.
- **Config file permissions (refuted).** Single-user home machine; contents are a ZIP +
  clock mode. No realistic trigger.
- **Process spawning (verified clean).** Every `std::process::Command` uses fixed binaries
  with hard-coded args; `open_settings` maps through a closed whitelist; no shell. Keep
  this pattern.
- **Updater (verified correct).** Pinned minisign pubkey over HTTPS; signature enforced.
- **`opener:default` capability (info).** Broader than needed. → *Fixed:* removed the
  unused `tauri-plugin-opener` and its capability.

## Correctness (top findings)
- **HIGH — `parse_nmcli` ignored escaped colons.** nmcli `-t` escapes `:` in an SSID as
  `\:`; naive split corrupted such SSIDs. → *Fixed:* `split_terse` honors `\:`/`\\` with
  tests.
- **HIGH — `read_weather` blocked the IPC thread** for a multi-second HTTP chain (~13s
  worst case). → *Fixed:* `read_tile` and `read_weather` are now async and offload the
  blocking work via `spawn_blocking` (also fixes the internet TCP-probe stall).
- **MEDIUM — Printers never reported `out_of_paper`** (frontend amber state was dead).
  → *Fixed:* detect `paper` / `media-empty` in the lpstat reason text, with a test.
- **MEDIUM — `net-changed` listened for but never emitted** (instant Wi-Fi/Internet update
  is dead code). → *Deferred:* requires the D-Bus push collector (see below); polling is
  the working fallback.
- Minor (noted, not yet changed): brittle `lpstat`/`wpctl` phrase matching; clock pauses
  when hidden (spec wanted it always ticking); `used_memory` semantics.

## Performance
- **D-Bus event-driven updates were never implemented** — the spec's headline resource
  optimization. The panel polls subprocesses instead. *Deferred enhancement.*
- **Volume re-spawns `wpctl` every 5s** — heaviest steady cost. Candidate for event-driven.
- `memory.rs` allocated a fresh `System` each tick. → *Fixed:* reuse one `System` via
  `OnceLock<Mutex<…>>`. (`storage.rs` left as-is; 30s cadence, lower impact.)
- Visibility pause skips the fetch but never clears timers (inert on a borderless panel).

## Deferred (need the Zorin machine or are larger enhancements)
- D-Bus push for Wi-Fi/Internet/Volume (instant updates; removes most polling).
- Build + verify the AppImage and the end-to-end release/auto-update pipeline on Zorin.
- Linux-only runtime confirmation of every collector (nmcli/CUPS/wpctl/GNOME panels).

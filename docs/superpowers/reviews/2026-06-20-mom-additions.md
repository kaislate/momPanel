# momPanel — Feature Additions for a Non-Technical User

**Date:** 2026-06-20
**Method:** Multi-agent workflow (`mompanel-mom-additions`): five parallel lenses
(accessibility, reassurance & safety, everyday tasks, clarity & error-proofing,
connection & delight) → synthesis that deduped and ranked 35 ideas by value vs. effort.

## Quick wins — low effort, near-zero risk (frontend/CSS/copy only)
1. **"Calm mode" + honor OS reduce-motion** — stop the sweeping second hand and
   hover/startup animation for users who find motion distracting/dizzying.
2. **High-contrast / larger type pass** — raise the dim caption color and enlarge the tiny
   all-caps titles so the panel reads from across the room.
3. **Plain-language alert line** — turn "87% full" / "offline" into one friendly sentence
   plus the right Open button.
4. **Soften scary states** — a calm "this is normal" subtitle under red Offline / printer-off
   so a routine blip doesn't trigger a panic call.
5. **Friendly unavailable states** — e.g. "No Wi-Fi found — you may be on a cable, that's
   okay" instead of bare "Not available."
6. **Buttons that say exactly what happens** — "Open Wi-Fi settings" + "opens a new window,
   nothing here changes."
7. **Add the missing Storage tile title**; **warmer ZIP prompt** ("Where do you live? we
   only use it for weather"); bundled **daily quote**, **"Good morning, \<name\>"** greeting,
   and a **sunrise/sunset** tile (free — Open-Meteo already returns it).

## High-impact — best for this persona (some are more effort)
1. **"Make everything bigger" master control** (Normal / Big / Biggest) — one pill scales
   all text, gauges, and buttons at once and persists. The single biggest accessibility win.
   *(Main effort: move tile/gauge sizes from fixed px to rem.)*
2. **"Everything's OK" hero health tile** — roll the gauges it already reads into one calm
   green/amber verdict ("is my computer happy?"); never a scary red.
3. **Color-blind-safe status** — add an icon *shape* and a *word* to every green/amber/red
   signal (~1 in 12 users can't rely on color alone).
4. **First-run welcome that names each tile once** — turns nine unfamiliar graphics into
   confidence, reusing the existing modal.
5. **Per-tile "What is this?" help dot** — a one-sentence explanation, learned at her pace.
6. **Reminders tile (medication & appointments, local only)** — most-requested everyday task.
7. **Alarms/timers on the clock tile**, an **"info for tech support" tile** (IP / name /
   network / version with copy buttons), **rotating family photos**, **upcoming birthdays**.

## Nice-to-have
Optional text labels under icons; bigger touch targets + visible focus rings; screen-reader
aria-labels + polite live regions; an "updates / you're protected" tile; gentle backup
reminder; a scam-awareness card; a **"Get help / start remote support"** button locked to one
pre-chosen app (never a free-text command).

## Synthesis guidance
- **Build one shared "comprehension layer":** a single help-text string per tile, reused for
  labels, tooltips, and the help dot — don't build it three times.
- **Calm-alert line, soften-scary-states, and friendly-unavailable copy are the same
  presentation helper** applied in different tiles.
- **Greeting tile + welcome overlay share** the one-time name/config prompt plumbing.

## Explicitly dropped / cautions
- "Back to desktop / list my windows" — no stable cross-platform command; breaks under
  Wayland (Zorin's likely compositor).
- Full Google/Outlook calendar OAuth — breaks the no-account simplicity and adds token/
  privacy weight. ICS-URL-only at most.
- Live quote / on-this-day APIs — bundle a static list; a network dependency adds
  unpredictability for no benefit.
- **Privacy:** keep all personal data (name, reminders, birthdays, photos, ZIP) strictly
  local in config. **Safety:** every actionable tile stays display-or-open-the-right-screen;
  any "get help" button launches only a pre-configured app.

## Sequencing
Do the frontend/CSS/copy quick wins first (no backend, no risk). The size control and
color-blind redundancy need the px→rem refactor. Tiles needing a new Rust collector
(updates, backup, tech-support info, photos) are higher effort and should be validated on
the real Zorin machine.

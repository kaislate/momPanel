# momPanel Layout-Stability Audit (pixel jitter on hover/refresh)

**Date:** 2026-06-21
**Method:** Multi-agent workflow (`mompanel-layout-audit`): four parallel reviewers
(shared layout/CSS, gauge tiles, text tiles, status tiles) → synthesis root-causing the
reported "elements shift a few pixels when moving the mouse."

## Root cause (fixed)
- **PRIMARY — hover `transform: scale(1.03)`** (`.tile:hover`). A *fractional* scale
  re-rasterizes each tile's text, inline SVG, and images onto a different sub-pixel grid
  on hover in/out, so detailed elements appear to shift a few pixels while the mouse
  moves. It's the only mouse-driven rule and the only thing that hits the **Date** tile
  (which has no image). Confirmed by it being disabled under `prefers-reduced-motion`
  (where the bug doesn't reproduce).
  → **Fix:** translate-only lift (`translateY(-6px)`), `will-change: transform` on
  `.tile` for clean compositing, and replaced the storage gauge's `scale(1.04)` hover
  with an opacity cue.
- **SECONDARY — `<img>` icons with `height: auto`** (`.device-icon`, `.printer-photo`).
  Tiles re-render by rebuilding `.tile-content.innerHTML` on a timer, which recreates the
  `<img>`; with no reserved height the box momentarily collapses then snaps to its
  aspect-ratio height, re-centering the flex `.gauge-row` (CPU/memory/storage every
  3–30s).
  → **Fix:** reserved explicit boxes (`width`+`height`+`object-fit: contain`) on
  `.device-icon`, `.ram-stack .device-icon`, and `.printer-photo`.
- **Numeric width stability** — added `font-variant-numeric: tabular-nums` to `.tile-big`
  and `.tile-sub` so the clock/percentages don't nudge centered/flex content as digits
  change.

## Other bugs
- **Fixed:** storage full/free toggle dropped keyboard focus (re-render destroyed the
  focused element) — now restores focus after re-render.
- **Deferred (noted, lower value):**
  - Render architecture: tiles rebuild the *entire* content (including static icons and
    the calendar) every tick. Reserving image boxes removes the visible reflow, but a
    larger win would be to build the static scaffold once and update only the dynamic
    nodes. Not done — higher risk, and the jitter is resolved without it.
  - Date tile re-renders the calendar every 1s; could update only when the day changes.
  - `wifiIcon()` SVG has no explicit width/height (safe today because it's only used in
    the muted placeholder, which is sized by CSS).
  - `.wx-forecast` has no overflow handling — low risk because the window grows with the
    size control, so the tile scales rather than overflowing.

export const meta = {
  name: 'mompanel-layout-audit',
  description: 'Find the cause of on-hover/refresh pixel jitter and sweep every tile for layout-stability and correctness bugs',
  phases: [
    { title: 'Audit', detail: 'parallel reviewers over shared layout + tile groups' },
    { title: 'Synthesize', detail: 'root-cause the jitter and prioritize fixes' },
  ],
}

const REPO = 'C:/Documents/NEw project/Project 11/momPanel'

const COMMON = `
You are auditing the momPanel frontend (vanilla HTML/CSS/JS in ${REPO}/src) for a
specific reported bug PLUS general layout/correctness quality. READ-ONLY: do not edit;
read the code and report findings with exact file:line and a concrete fix.

REPORTED BUG: when the user moves the mouse around the interface, some elements shift a
few pixels. MOST NOTICEABLE: the CPU tile (its computer-image icon and the ring/gauge
move slightly) and the Date tile (the date/calendar). Find the ROOT CAUSE.

How the UI works (so you reason correctly):
- Tiles are created once in src/tiles.js mountTiles(): each is a <section class="tile">
  containing a persistent ".tile-help" dot and a ".tile-content" div. Tiles re-render by
  setting .tile-content.innerHTML on a timer (Date every 1s, CPU every 3s, memory 3s,
  others 5-30s). So the inner HTML (including any <img>) is REBUILT every refresh.
- src/styles.css has ".tile:hover { transform: translateY(-6px) scale(1.03) }" with a
  0.28s transition. Tiles use flexbox: ".tile-graphic" centers content; memory/storage/
  cpu use ".gauge-row { justify-content: space-between }" with a left icon (an <img>
  ".device-icon" sized "width: 3.4rem; height: auto", or ".ram-stack" of two imgs) and a
  ".gauge-fixed" (7rem) ring on the right. Printers use ".printer-photo { max-height }".
- The gauges are inline SVG from src/gauge.js (re-generated each refresh).

Prime suspects to evaluate (confirm or refute each, with reasoning):
  1. The hover "scale(1.03)" re-rasterizing tile content -> sub-pixel text/SVG/img shifts.
  2. <img> icons with "height: auto" (no reserved height) reflowing each time the tile
     re-renders (the <img> is recreated, briefly has no/inferred size -> layout shift).
  3. flex "space-between" / "flex: 1" redistributing sub-pixel space differently per
     render or during the hover scale.
  4. Re-rendering the WHOLE tile content on a timer (including static icons) causing
     periodic reflow that coincides with mouse movement.
Recommend the most robust fix for each real issue (e.g. reserve image box with explicit
width+height+object-fit; drop scale on hover or promote with a compositor layer; avoid
rebuilding static parts; etc).
`

const FIND_SCHEMA = {
  type: 'object',
  additionalProperties: false,
  required: ['area', 'findings'],
  properties: {
    area: { type: 'string' },
    findings: {
      type: 'array',
      items: {
        type: 'object',
        additionalProperties: false,
        required: ['title', 'severity', 'location', 'cause', 'fix', 'confidence', 'jitter_related'],
        properties: {
          title: { type: 'string' },
          severity: { type: 'string', enum: ['high', 'medium', 'low', 'info'] },
          location: { type: 'string' },
          cause: { type: 'string' },
          fix: { type: 'string' },
          confidence: { type: 'string', enum: ['high', 'medium', 'low'] },
          jitter_related: { type: 'boolean', description: 'does this contribute to the reported pixel-shift bug?' },
        },
      },
    },
  },
}

const AREAS = [
  {
    key: 'shared-layout',
    prompt: `${COMMON}
YOUR AREA: the SHARED layout + CSS. Read src/styles.css (the .tile, .tile-content,
.tile-graphic, .tile-foot, .gauge-row, .gauge-fixed, .device-icon, .ram-stack,
.printer-photo, hover transform, and animations), src/tiles.js (mountTiles + render
loop), src/layout.js, and src/gauge.js. This is where the jitter root cause most likely
lives. Pin down exactly which rule(s) cause elements to shift on hover and/or on refresh.`,
  },
  {
    key: 'gauge-tiles',
    prompt: `${COMMON}
YOUR AREA: the gauge tiles with image icons — src/tiles/cpu.js, src/tiles/memory.js,
src/tiles/storage.js. The CPU tile is the headline symptom. Check how the icon <img> and
gauge are sized/laid out and whether re-rendering them causes reflow. Also check general
correctness (event listeners re-added each render, the storage full/free toggle, etc).`,
  },
  {
    key: 'text-tiles',
    prompt: `${COMMON}
YOUR AREA: src/tiles/clock.js (clock + date) and src/tiles/weather.js. The Date tile is a
named symptom and has NO image, so figure out why IT jitters (calendar grid? re-render
every 1s? the date-row flex?). Check weather's double-height custom layout too.`,
  },
  {
    key: 'status-tiles',
    prompt: `${COMMON}
YOUR AREA: src/tiles/wifi.js, src/tiles/internet.js, src/tiles/volume.js,
src/tiles/printers.js. Check layout stability (printers uses a photo <img>) and general
correctness. Note any element that could shift on hover or refresh.`,
  },
]

phase('Audit')
const audits = await parallel(
  AREAS.map((a) => () =>
    agent(a.prompt, { label: `audit:${a.key}`, phase: 'Audit', schema: FIND_SCHEMA })
  )
)
const all = audits.filter(Boolean)
const flat = all.flatMap((r) => (r.findings || []).map((f) => ({ ...f, area: r.area })))
log(`Audit done: ${flat.length} findings (${flat.filter((f) => f.jitter_related).length} jitter-related).`)

phase('Synthesize')
const SUMMARY_SCHEMA = {
  type: 'object',
  additionalProperties: false,
  required: ['root_cause', 'jitter_fixes', 'other_bugs', 'notes'],
  properties: {
    root_cause: { type: 'string', description: 'the single best explanation of the reported pixel-shift, with the exact rule/line' },
    jitter_fixes: { type: 'array', items: { type: 'string' }, description: 'ordered, concrete fixes to eliminate the jitter' },
    other_bugs: { type: 'array', items: { type: 'string' }, description: 'unrelated correctness/quality issues worth fixing, most important first' },
    notes: { type: 'string' },
  },
}
const summary = await agent(
  `${COMMON}
Synthesize these ${flat.length} findings (JSON) into a verdict. Decide the SINGLE most
likely root cause of the pixel-shift (reconcile why it hits the Date tile, which has no
image, as well as the CPU tile). Give an ordered list of concrete fixes to remove the
jitter, then a prioritized list of other real bugs. Be specific with file/rule names.

FINDINGS:
${JSON.stringify(flat, null, 2)}`,
  { label: 'synthesize', phase: 'Synthesize', effort: 'high', schema: SUMMARY_SCHEMA }
)

return { findings: flat, summary }

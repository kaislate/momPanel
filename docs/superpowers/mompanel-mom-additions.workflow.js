export const meta = {
  name: 'mompanel-mom-additions',
  description: 'Suggest momPanel additions for a non-technical user (the "mom" persona), then synthesize and rank by value vs effort',
  phases: [
    { title: 'Ideate', detail: 'parallel lenses, each proposing tailored additions' },
    { title: 'Synthesize', detail: 'dedup, rank by value/effort, pick the shortlist' },
  ],
}

const REPO = 'C:/Documents/NEw project/Project 11/momPanel'

const COMMON = `
momPanel is a graphics-first, always-on desktop info panel built for a NON-TECHNICAL user —
think someone's mom who finds Control Panel/Settings intimidating (hence "momPanel"). It runs
on Linux/Zorin (primary) and Windows (test only). It is a frameless, non-movable window of big
graphic tiles. CURRENT tiles: clock (analog/digital), date, memory, storage, Wi-Fi, internet
online/offline, printers, volume, weather (asks for a ZIP once). Each relevant tile has ONE big
friendly button that opens the correct native settings screen (it never changes settings
itself). Design values: prioritize GRAPHICS over text, calm, glanceable, safe, low resource use,
no jargon. Repo (for grounding in what exists): ${REPO} — you MAY read files but the goal is
IDEAS, not code review. READ-ONLY: do not edit anything.

For EVERY suggestion: tie it to the momPanel design (a tile, a button, or a gentle prompt),
keep it safe (display or open-the-right-screen, never risky automation), use plain language a
beginner would understand, and be honest about effort and Linux/Windows feasibility.
`

const IDEAS_SCHEMA = {
  type: 'object',
  additionalProperties: false,
  required: ['lens', 'suggestions'],
  properties: {
    lens: { type: 'string' },
    suggestions: {
      type: 'array',
      items: {
        type: 'object',
        additionalProperties: false,
        required: ['title', 'description', 'why_it_helps', 'form', 'effort', 'value', 'feasibility'],
        properties: {
          title: { type: 'string' },
          description: { type: 'string', description: 'what it shows/does, in plain terms' },
          why_it_helps: { type: 'string', description: 'the concrete benefit to a non-technical user' },
          form: { type: 'string', description: 'how it fits momPanel: a tile / a button / a gentle prompt / a setting' },
          effort: { type: 'string', enum: ['low', 'medium', 'high'] },
          value: { type: 'string', enum: ['high', 'medium', 'low'] },
          feasibility: { type: 'string', description: 'Linux/Windows notes, any tool/API needed, any privacy concern' },
        },
      },
    },
  },
}

const LENSES = [
  {
    key: 'accessibility',
    prompt: `${COMMON}
LENS: ACCESSIBILITY for older / less-able users — larger readable type, high-contrast and
color-blind-safe status colors, big touch/click targets, reduce-motion option for the
animations, optional text labels alongside icons, screen-reader friendliness, a simple
"make everything bigger" control. Propose concrete momPanel additions.`,
  },
  {
    key: 'reassurance',
    prompt: `${COMMON}
LENS: REASSURANCE & SAFETY — a non-technical user mostly wants to know "is my computer OK?"
Ideas like a single friendly overall "health" status, calm plain-language alerts (low disk,
no internet, printer out of paper), a gentle backup reminder, an at-a-glance "you're protected"
/ updates-installed indicator, and scam/phishing awareness cues. Nothing alarming or technical.`,
  },
  {
    key: 'everyday-tasks',
    prompt: `${COMMON}
LENS: EVERYDAY TASKS a parent actually does — big readable clock with optional alarms/reminders,
appointment or medication reminders, a giant "take a screenshot" or "show me my IP for support"
helper, an easy "call for help / start remote support" button, day/weather at a glance, a
"find my open windows" or "back to desktop" helper. Keep each safe and one-tap.`,
  },
  {
    key: 'clarity',
    prompt: `${COMMON}
LENS: CLARITY & ERROR-PROOFING — remove jargon, explain what each tile means in a tooltip or
plain caption, make the "open settings" buttons say exactly what will happen, add a tiny "What
is this?" help affordance, friendly empty/unavailable states, and a first-run welcome that
orients a beginner. Also: how to keep them from getting confused or stuck.`,
  },
  {
    key: 'connection-delight',
    prompt: `${COMMON}
LENS: CONNECTION & DELIGHT — gentle touches that make it feel personal and warm for a parent:
a family photo / rotating photos tile, upcoming family birthdays or a shared calendar glance,
a "good morning, <name>" greeting, day-length / sunrise-sunset, a quote or on-this-day tile.
Keep it lightweight, private by default, and optional.`,
  },
]

phase('Ideate')
const batches = await parallel(
  LENSES.map((l) => () =>
    agent(l.prompt, { label: `ideate:${l.key}`, phase: 'Ideate', schema: IDEAS_SCHEMA })
  )
)
const ideas = batches.filter(Boolean)
const flat = ideas.flatMap((b) => (b.suggestions || []).map((s) => ({ ...s, lens: b.lens })))
log(`Collected ${flat.length} suggestions across ${ideas.length} lenses. Synthesizing.`)

phase('Synthesize')
const SHORTLIST_SCHEMA = {
  type: 'object',
  additionalProperties: false,
  required: ['quick_wins', 'high_impact', 'nice_to_have', 'notes'],
  properties: {
    quick_wins: { type: 'array', items: { type: 'string' }, description: 'low-effort / high-or-medium-value, do these first' },
    high_impact: { type: 'array', items: { type: 'string' }, description: 'highest value for this persona, may be more effort' },
    nice_to_have: { type: 'array', items: { type: 'string' } },
    notes: { type: 'string', description: 'themes, duplicates merged, anything to avoid (privacy/complexity)' },
  },
}
const shortlist = await agent(
  `${COMMON}
You are synthesizing ${flat.length} raw suggestions (JSON below) from five lenses into a
practical shortlist for momPanel. Merge duplicates, drop anything unsafe or too technical for
the persona, and SORT by value-vs-effort. Produce: quick_wins (low effort, do first),
high_impact (best for a non-technical parent), nice_to_have, and notes (themes + what to avoid).
Each list item: a short imperative line naming the addition and its one-line benefit.

RAW SUGGESTIONS:
${JSON.stringify(flat, null, 2)}`,
  { label: 'synthesize', phase: 'Synthesize', effort: 'high', schema: SHORTLIST_SCHEMA }
)

return { total_suggestions: flat.length, by_lens: ideas.map((b) => ({ lens: b.lens, n: b.suggestions?.length || 0 })), all_suggestions: flat, shortlist }

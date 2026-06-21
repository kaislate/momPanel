export const meta = {
  name: 'mompanel-audit',
  description: 'Read-only audit of momPanel: security + feature + correctness + perf, with adversarial verification of security findings',
  phases: [
    { title: 'Review', detail: 'parallel reviewers, one per dimension' },
    { title: 'Verify', detail: 'adversarially confirm each security finding is real' },
  ],
}

const REPO = 'C:/Documents/NEw project/Project 11/momPanel'

const COMMON = `
You are auditing the momPanel project (a Tauri 2 desktop info panel; Rust backend in
src-tauri/, vanilla JS frontend in src/). Repo root: ${REPO}
READ-ONLY: do NOT edit any file. Read what you need (Read/Grep/Glob) and report.
Key files to inspect:
  src-tauri/src/lib.rs              (commands: read_tile, read_weather, set_config, open_settings; updater + autostart setup)
  src-tauri/src/shortcuts.rs        (spawns gnome-control-center / cmd)
  src-tauri/src/collectors/*.rs     (memory, storage, wifi, internet, volume, printers, weather)
  src-tauri/src/config.rs           (JSON config persistence)
  src-tauri/tauri.conf.json         (security.csp, withGlobalTauri, updater pubkey/endpoints, capabilities)
  src-tauri/capabilities/default.json
  src/*.js, src/tiles/*.js          (how system-derived strings are rendered into the DOM)
  docs/superpowers/specs/*          (the agreed design, for feature-gap analysis)
Context: Linux/Zorin is the real target; Windows is test-only. The panel is for a
non-technical user and runs always-on.
`

const FINDINGS_SCHEMA = {
  type: 'object',
  additionalProperties: false,
  required: ['dimension', 'findings'],
  properties: {
    dimension: { type: 'string' },
    findings: {
      type: 'array',
      items: {
        type: 'object',
        additionalProperties: false,
        required: ['title', 'severity', 'location', 'description', 'recommendation', 'confidence'],
        properties: {
          title: { type: 'string' },
          severity: { type: 'string', enum: ['high', 'medium', 'low', 'info'] },
          location: { type: 'string', description: 'file:line or file:area' },
          description: { type: 'string' },
          recommendation: { type: 'string' },
          confidence: { type: 'string', enum: ['high', 'medium', 'low'] },
        },
      },
    },
  },
}

const VERDICT_SCHEMA = {
  type: 'object',
  additionalProperties: false,
  required: ['title', 'verdict', 'reasoning', 'realistic_scenario', 'adjusted_severity'],
  properties: {
    title: { type: 'string' },
    verdict: { type: 'string', enum: ['confirmed', 'refuted', 'uncertain'] },
    reasoning: { type: 'string' },
    realistic_scenario: { type: 'string', description: 'concrete way it could actually be triggered in this app, or why it cannot be' },
    adjusted_severity: { type: 'string', enum: ['high', 'medium', 'low', 'info', 'none'] },
  },
}

const DIMENSIONS = [
  {
    key: 'security',
    prompt: `${COMMON}
DIMENSION: SECURITY. Hunt concretely, citing exact code. Consider at least:
- Command/argument injection where the backend spawns processes (shortcuts.rs open_settings;
  any collector using std::process::Command). Are inputs whitelisted or attacker-influenced?
- URL building in weather.rs: the ZIP is interpolated into a URL. Is it validated on the
  BACKEND (firstrun.js validates only on the frontend)? Path/host manipulation / SSRF potential?
- Webview injection: security.csp is null and withGlobalTauri is true. Do any tiles render
  SYSTEM-DERIVED strings (Wi-Fi SSID, printer names, weather place) via innerHTML without
  escaping? Could a crafted SSID/printer name execute script in the webview, which can then
  call exposed Tauri commands?
- Updater & signing: endpoint over HTTPS? pubkey present? any way an unsigned/MITM update installs?
- Config file: location/permissions, any injection when read back.
- Tauri capabilities/permissions: is the command surface minimal?
Report each as a finding with a real severity and a concrete recommendation.`,
  },
  {
    key: 'features',
    prompt: `${COMMON}
DIMENSION: FEATURE COMPLETENESS & GAPS vs the spec in docs/superpowers/specs/. Identify
spec items not implemented or only partially (e.g. D-Bus push events vs poll fallback,
battery tile intentionally omitted, AppImage packaging not yet built/verified). Also note
small high-value features that are clearly missing for the stated goal (graphics-first,
always-on, non-technical user). Keep each finding actionable; mark severity as info/low/medium.`,
  },
  {
    key: 'correctness',
    prompt: `${COMMON}
DIMENSION: CORRECTNESS & ROBUSTNESS. Look for real bugs and brittle spots: parsing
assumptions (nmcli escaping of ':' in SSIDs, lpstat status phrases, wpctl formats), sysinfo
usage (used vs available memory semantics), error paths that could panic or hang, the render
loop / visibility handling, the clock interval pausing when hidden, config merge correctness,
and any data-shape mismatch between Rust serialization and the JS that consumes it.`,
  },
  {
    key: 'performance',
    prompt: `${COMMON}
DIMENSION: PERFORMANCE & RESOURCE FOOTPRINT (this is an always-on panel that advertises low
usage). Evaluate polling cadences, subprocess-spawn frequency, whether work truly pauses when
hidden, timers that never clear, repeated System::new()/Disks allocation, and webview cost.
Suggest concrete, low-risk reductions. Severity info/low/medium.`,
  },
]

phase('Review')
const reviews = await parallel(
  DIMENSIONS.map((d) => () =>
    agent(d.prompt, { label: `review:${d.key}`, phase: 'Review', schema: FINDINGS_SCHEMA })
  )
)
const all = reviews.filter(Boolean)
// Pull the security review regardless of exactly how the model labeled `dimension`.
const secReview = all.find((r) => /sec/i.test(r.dimension || '')) || all[0]
const toVerify = (secReview?.findings || []).filter((f) => f.severity !== 'info')
log(`Reviews done. Verifying ${toVerify.length} security finding(s) adversarially.`)

phase('Verify')
const verdicts = await parallel(
  toVerify.map((f) => () =>
    agent(`${COMMON}
ADVERSARIAL VERIFICATION. A security reviewer claims this finding about momPanel:
  TITLE: ${f.title}
  LOCATION: ${f.location}
  CLAIM: ${f.description}
  PROPOSED FIX: ${f.recommendation}
Your job is to REFUTE it. Read the actual code at that location and decide whether the issue
is genuinely exploitable/relevant IN THIS APP (a local, single-user desktop panel). Default to
'refuted' if the attacker can't realistically influence the input, or 'uncertain' if unclear.
Give a concrete realistic_scenario (how it could actually be triggered) or explain why it can't.`,
      { label: `verify:${f.title.slice(0, 30)}`, phase: 'Verify', schema: VERDICT_SCHEMA, effort: 'high' })
  )
)

return {
  dimensions: all.map((r) => ({ dimension: r.dimension, count: r.findings?.length || 0 })),
  reviews: all,
  security_verdicts: verdicts.filter(Boolean),
}

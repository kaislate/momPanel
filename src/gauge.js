// Shared SVG arc/ring gauge, reused by memory, storage, and volume tiles.
// Ring color shifts ok -> warn -> bad as `percent` climbs; the three colors and the
// track are read from the theme's CSS variables so they follow the active theme.

// Pure threshold pick (exported for tests): >=90 bad, >=70 warn, else ok.
export function pickGauge(percent, ok, warn, bad) {
  if (percent >= 90) return bad;
  if (percent >= 70) return warn;
  return ok;
}

function cssVar(name, fallback) {
  if (typeof document === "undefined") return fallback;
  const v = getComputedStyle(document.documentElement).getPropertyValue(name).trim();
  return v || fallback;
}

export function gaugeColor(percent) {
  return pickGauge(
    percent,
    cssVar("--gauge-ok", "#5bd6a0"),
    cssVar("--gauge-warn", "#ffb347"),
    cssVar("--gauge-bad", "#ff5d5d")
  );
}

// percent: 0-100 (ring fill), label: big center text, sub: small caption under it.
// colorPercent (optional): basis for the color, when it differs from the fill.
export function arcGauge(percent, label, sub = "", colorPercent = percent) {
  const p = Math.max(0, Math.min(100, percent));
  const r = 42;
  const circ = 2 * Math.PI * r;
  const dash = (p / 100) * circ;
  const color = gaugeColor(colorPercent);
  const track = cssVar("--gauge-track", "#2a3146");
  // A round linecap on a zero-length dash still paints a stray dot at the ring start,
  // so when the fill rounds away to nothing we draw only the track.
  const arc =
    Math.round(dash) > 0
      ? `<circle cx="50" cy="50" r="${r}" fill="none" stroke="${color}" stroke-width="9"
      stroke-linecap="round" stroke-dasharray="${dash} ${circ}"
      transform="rotate(-90 50 50)" />`
      : "";
  return `
  <svg class="gauge" viewBox="0 0 100 100" role="img">
    <circle cx="50" cy="50" r="${r}" fill="none" stroke="${track}" stroke-width="9" />
    ${arc}
    <text x="50" y="48" text-anchor="middle" class="gauge-label">${label}</text>
    <text x="50" y="64" text-anchor="middle" class="gauge-sub">${sub}</text>
  </svg>`;
}

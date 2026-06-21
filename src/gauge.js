// Shared SVG arc/ring gauge, reused by memory, storage, and volume tiles.
// Returns an SVG string showing `percent` (0-100) as a filled ring with a
// color that shifts green -> amber -> red as it climbs.

export function gaugeColor(percent) {
  if (percent >= 90) return "#ff5d5d";
  if (percent >= 70) return "#ffb347";
  return "#5bd6a0";
}

// percent: 0-100 (ring fill), label: big center text, sub: small caption under it.
// colorPercent (optional): basis for the green/amber/red color, when it should differ
// from the fill — e.g. storage showing "% free" but colored by how full it is.
export function arcGauge(percent, label, sub = "", colorPercent = percent) {
  const p = Math.max(0, Math.min(100, percent));
  const r = 42;
  const circ = 2 * Math.PI * r;
  const dash = (p / 100) * circ;
  const color = gaugeColor(colorPercent);
  return `
  <svg class="gauge" viewBox="0 0 100 100" role="img">
    <circle cx="50" cy="50" r="${r}" fill="none" stroke="#2a3146" stroke-width="9" />
    <circle cx="50" cy="50" r="${r}" fill="none" stroke="${color}" stroke-width="9"
      stroke-linecap="round" stroke-dasharray="${dash} ${circ}"
      transform="rotate(-90 50 50)" />
    <text x="50" y="48" text-anchor="middle" class="gauge-label">${label}</text>
    <text x="50" y="64" text-anchor="middle" class="gauge-sub">${sub}</text>
  </svg>`;
}

// momPanel logo: a "mom" emoji with a tiny emoji-style momPanel (a mini dashboard of
// colored tiles) peeking over her shoulder. Pure SVG so it scales crisply; the emoji
// uses the system color-emoji font.
export function logoSvg(size = 120) {
  return (
    `<svg class="logo" viewBox="0 0 100 100" width="${size}" height="${size}" ` +
    `role="img" aria-label="momPanel logo">` +
    // mini momPanel behind the shoulder (drawn first = behind the mom)
    `<g transform="translate(54 12) rotate(-10)">` +
    `<rect width="40" height="30" rx="5" fill="#1b2030" stroke="#5b8cff" stroke-width="2"/>` +
    `<rect x="4" y="5" width="14" height="9" rx="2" fill="#5bd6a0"/>` +
    `<rect x="22" y="5" width="14" height="9" rx="2" fill="#ffb347"/>` +
    `<rect x="4" y="17" width="14" height="8" rx="2" fill="#5b8cff"/>` +
    `<rect x="22" y="17" width="14" height="8" rx="2" fill="#ff5d5d"/>` +
    `</g>` +
    // the mom, in front
    `<text x="42" y="82" font-size="62" text-anchor="middle">\u{1F469}</text>` +
    `</svg>`
  );
}

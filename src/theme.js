// Theme engine: named presets + a small set of curated color slots, applied by writing
// CSS custom properties onto :root. Text (--ink/--ink-dim) and a few helper colors are
// DERIVED so the user can't make the UI unreadable. Pure helpers avoid `document` at
// module top level so they can be unit-tested under Node.

export const PRESETS = {
  midnight: {
    accent: "#5b8cff",
    bg: "#0e1119",
    tile: "#1b2030",
    gauge_ok: "#5bd6a0",
    gauge_warn: "#ffb347",
    gauge_bad: "#ff5d5d",
  },
  warm: {
    accent: "#ff9e5b",
    bg: "#1a1410",
    tile: "#2a2018",
    gauge_ok: "#8fd66b",
    gauge_warn: "#ffb347",
    gauge_bad: "#ff5d5d",
  },
  "high-contrast": {
    accent: "#ffd400",
    bg: "#000000",
    tile: "#141414",
    gauge_ok: "#00e676",
    gauge_warn: "#ffb300",
    gauge_bad: "#ff1744",
  },
};

export function defaultTheme() {
  return { preset: "midnight", ...PRESETS.midnight };
}

function parseHex(hex) {
  const m = /^#?([0-9a-f]{6})$/i.exec(hex || "");
  if (!m) return null;
  const n = parseInt(m[1], 16);
  return { r: (n >> 16) & 255, g: (n >> 8) & 255, b: n & 255 };
}

// Perceptual luminance -> pick dark or light text for readability on `hex`.
export function contrastText(hex) {
  const c = parseHex(hex);
  if (!c) return "#eef2ff";
  const lum = (0.299 * c.r + 0.587 * c.g + 0.114 * c.b) / 255;
  return lum > 0.6 ? "#1a1a1a" : "#eef2ff";
}

// Shift a color toward white (amt 0..1) — used to derive tile-hover from tile.
function lighten(hex, amt) {
  const c = parseHex(hex);
  if (!c) return hex;
  const f = (v) => Math.round(v + (255 - v) * amt);
  return `#${[f(c.r), f(c.g), f(c.b)].map((v) => v.toString(16).padStart(2, "0")).join("")}`;
}

// Dim text = 72% blend of ink toward the background (matches the existing look).
function mix(a, b, t) {
  const x = parseHex(a);
  const y = parseHex(b);
  if (!x || !y) return a;
  const f = (p, q) => Math.round(p + (q - p) * t);
  return `#${[f(x.r, y.r), f(x.g, y.g), f(x.b, y.b)]
    .map((v) => v.toString(16).padStart(2, "0"))
    .join("")}`;
}

export function applyTheme(theme) {
  const t = theme && theme.accent ? theme : defaultTheme();
  const root = document.documentElement.style;
  root.setProperty("--accent", t.accent);
  root.setProperty("--bg", t.bg);
  root.setProperty("--tile", t.tile);
  root.setProperty("--tile-hover", lighten(t.tile, 0.12));
  root.setProperty("--gauge-ok", t.gauge_ok);
  root.setProperty("--gauge-warn", t.gauge_warn);
  root.setProperty("--gauge-bad", t.gauge_bad);
  const ink = contrastText(t.bg);
  root.setProperty("--ink", ink);
  root.setProperty("--ink-dim", mix(ink, t.bg, 0.28));
  // Keep the body radial-gradient cohesive: derive its top stop from the tile color.
  root.setProperty("--bg-glow", lighten(t.tile, 0.06));
}

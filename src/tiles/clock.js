// Clock + Date tiles. Frontend-only: there is no Rust collector — both tiles
// fetch null and render purely from the browser clock (Date). The clock tile
// supports two display modes (digital / analog) persisted via config so the
// chosen mode survives a restart.
import { getConfig, setConfig } from "../api.js";

// Module-level mode, hydrated from persisted config on register() and flipped
// by the in-tile toggle button.
let clockMode = "digital";

// Build the analog clock SVG for a given Date. 100x100 viewBox, all twelve
// numerals on a circle (radius ~38), tick marks, and three hands. Remember SVG
// y grows downward, so "12 o'clock" is the negative-y direction (top).
function analogSvg(now) {
  const cx = 50;
  const cy = 50;
  const numeralR = 38;
  const tickOuter = 46;
  const tickInner = 42;

  // Twelve numerals positioned at n*30deg. Angle 0 = top (12), increasing clockwise.
  let numerals = "";
  for (let n = 1; n <= 12; n++) {
    const angle = (n * 30 * Math.PI) / 180; // radians, 0 = 12 o'clock
    const x = cx + numeralR * Math.sin(angle);
    const y = cy - numeralR * Math.cos(angle);
    numerals +=
      `<text x="${x.toFixed(2)}" y="${y.toFixed(2)}" font-size="9" ` +
      `text-anchor="middle" dominant-baseline="central" ` +
      `fill="currentColor">${n}</text>`;
  }

  // Tick marks at every hour position.
  let ticks = "";
  for (let n = 0; n < 12; n++) {
    const angle = (n * 30 * Math.PI) / 180;
    const x1 = cx + tickInner * Math.sin(angle);
    const y1 = cy - tickInner * Math.cos(angle);
    const x2 = cx + tickOuter * Math.sin(angle);
    const y2 = cy - tickOuter * Math.cos(angle);
    ticks +=
      `<line x1="${x1.toFixed(2)}" y1="${y1.toFixed(2)}" ` +
      `x2="${x2.toFixed(2)}" y2="${y2.toFixed(2)}" ` +
      `stroke="currentColor" stroke-width="1" opacity="0.6"/>`;
  }

  const hours = now.getHours() % 12;
  const minutes = now.getMinutes();
  const seconds = now.getSeconds();

  const hourDeg = hours * 30 + minutes * 0.5;
  const minDeg = minutes * 6;
  const secDeg = seconds * 6;

  return (
    `<svg class="gauge" viewBox="0 0 100 100" width="100" height="100" ` +
    `xmlns="http://www.w3.org/2000/svg" role="img" aria-label="Analog clock">` +
    `<circle cx="${cx}" cy="${cy}" r="47" fill="none" ` +
    `stroke="currentColor" stroke-width="1.5" opacity="0.5"/>` +
    ticks +
    numerals +
    // Hour hand
    `<line x1="${cx}" y1="${cy}" x2="${cx}" y2="${cy - 22}" ` +
    `stroke="currentColor" stroke-width="3" stroke-linecap="round" ` +
    `transform="rotate(${hourDeg.toFixed(2)} ${cx} ${cy})"/>` +
    // Minute hand
    `<line x1="${cx}" y1="${cy}" x2="${cx}" y2="${cy - 32}" ` +
    `stroke="currentColor" stroke-width="2" stroke-linecap="round" ` +
    `transform="rotate(${minDeg.toFixed(2)} ${cx} ${cy})"/>` +
    // Second hand
    `<line x1="${cx}" y1="${cy}" x2="${cx}" y2="${cy - 36}" ` +
    `stroke="#e0564f" stroke-width="1" stroke-linecap="round" ` +
    `transform="rotate(${secDeg.toFixed(2)} ${cx} ${cy})"/>` +
    `<circle cx="${cx}" cy="${cy}" r="2.5" fill="currentColor"/>` +
    `</svg>`
  );
}

function renderClock(el) {
  const now = new Date();
  const toggleLabel = clockMode === "analog" ? "digital" : "analog";

  let body;
  if (clockMode === "analog") {
    body = `<div class="gauge">${analogSvg(now)}</div>`;
  } else {
    body = `<div class="tile-big">${now.toLocaleTimeString()}</div>`;
  }

  el.innerHTML =
    `<div class="tile-title">Clock</div>` +
    body +
    `<button class="tile-btn" type="button" data-action="toggle">${toggleLabel}</button>`;

  const btn = el.querySelector('[data-action="toggle"]');
  if (btn) {
    btn.addEventListener("click", async () => {
      clockMode = clockMode === "analog" ? "digital" : "analog";
      // Persist so the mode survives a restart; render immediately regardless.
      setConfig({ clock_mode: clockMode });
      renderClock(el);
    });
  }
}

function renderDate(el) {
  const now = new Date();
  const weekday = now.toLocaleDateString(undefined, { weekday: "long" });
  el.innerHTML =
    `<div class="tile-title">${weekday}</div>` +
    `<div class="tile-big">${now.toLocaleDateString()}</div>`;
}

export async function register(registerTile) {
  // Hydrate the persisted mode before the tiles start rendering.
  try {
    const cfg = await getConfig();
    if (cfg && (cfg.clock_mode === "analog" || cfg.clock_mode === "digital")) {
      clockMode = cfg.clock_mode;
    }
  } catch {
    clockMode = "digital";
  }

  registerTile({
    id: "clock",
    title: "Clock",
    intervalMs: 1000,
    fetch: async () => null,
    render(el) {
      renderClock(el);
    },
  });

  registerTile({
    id: "date",
    title: "Date",
    intervalMs: 1000,
    fetch: async () => null,
    render(el) {
      renderDate(el);
    },
  });
}

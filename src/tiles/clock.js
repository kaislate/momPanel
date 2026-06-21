// Clock + Date tiles (frontend only). Clock graphic is an analog face or a big
// digital time, with a Show analog/digital toggle in the foot (persisted). Date is
// the weekday (graphic) over the full date (foot).
import { getConfig, setConfig } from "../api.js";
import { tile } from "../layout.js";

let clockMode = "digital";

// Analog face: all twelve numerals on a circle, tick marks, and three hands.
function analogSvg(now) {
  const cx = 50;
  const cy = 50;
  const numeralR = 38;
  let numerals = "";
  for (let n = 1; n <= 12; n++) {
    const angle = (n * 30 * Math.PI) / 180;
    const x = cx + numeralR * Math.sin(angle);
    const y = cy - numeralR * Math.cos(angle);
    numerals +=
      `<text x="${x.toFixed(2)}" y="${y.toFixed(2)}" font-size="9" ` +
      `text-anchor="middle" dominant-baseline="central" fill="currentColor">${n}</text>`;
  }
  let ticks = "";
  for (let n = 0; n < 12; n++) {
    const angle = (n * 30 * Math.PI) / 180;
    const x1 = cx + 42 * Math.sin(angle);
    const y1 = cy - 42 * Math.cos(angle);
    const x2 = cx + 46 * Math.sin(angle);
    const y2 = cy - 46 * Math.cos(angle);
    ticks += `<line x1="${x1.toFixed(2)}" y1="${y1.toFixed(2)}" x2="${x2.toFixed(
      2
    )}" y2="${y2.toFixed(2)}" stroke="currentColor" stroke-width="1" opacity="0.6"/>`;
  }
  const hours = now.getHours() % 12;
  const minutes = now.getMinutes();
  const seconds = now.getSeconds();
  const hourDeg = hours * 30 + minutes * 0.5;
  const minDeg = minutes * 6;
  const secDeg = seconds * 6;
  return (
    `<svg class="gauge" viewBox="0 0 100 100" role="img" aria-label="Analog clock">` +
    `<circle cx="${cx}" cy="${cy}" r="47" fill="none" stroke="currentColor" stroke-width="1.5" opacity="0.5"/>` +
    ticks +
    numerals +
    `<line x1="${cx}" y1="${cy}" x2="${cx}" y2="${cy - 22}" stroke="currentColor" stroke-width="3" stroke-linecap="round" transform="rotate(${hourDeg.toFixed(
      2
    )} ${cx} ${cy})"/>` +
    `<line x1="${cx}" y1="${cy}" x2="${cx}" y2="${cy - 32}" stroke="currentColor" stroke-width="2" stroke-linecap="round" transform="rotate(${minDeg.toFixed(
      2
    )} ${cx} ${cy})"/>` +
    `<line x1="${cx}" y1="${cy}" x2="${cx}" y2="${cy - 36}" stroke="#e0564f" stroke-width="1" stroke-linecap="round" transform="rotate(${secDeg.toFixed(
      2
    )} ${cx} ${cy})"/>` +
    `<circle cx="${cx}" cy="${cy}" r="2.5" fill="currentColor"/></svg>`
  );
}

function renderClock(el) {
  const now = new Date();
  const next = clockMode === "analog" ? "digital" : "analog";
  const graphic =
    clockMode === "analog"
      ? analogSvg(now)
      : `<div class="tile-big">${now.toLocaleTimeString()}</div>`;
  el.innerHTML = tile({
    title: "Clock",
    graphic,
    foot: `<button class="tile-btn" type="button" data-action="toggle">Show ${next}</button>`,
  });
  el.querySelector('[data-action="toggle"]')?.addEventListener("click", () => {
    clockMode = next;
    setConfig({ clock_mode: clockMode });
    renderClock(el);
  });
}

function renderDate(el) {
  const now = new Date();
  const weekday = now.toLocaleDateString(undefined, { weekday: "long" });
  el.innerHTML = tile({
    title: "Date",
    graphic: `<div class="tile-big">${weekday}</div>`,
    foot: `<div class="tile-sub">${now.toLocaleDateString()}</div>`,
  });
}

export async function register(registerTile) {
  try {
    const cfg = await getConfig();
    if (cfg && (cfg.clock_mode === "analog" || cfg.clock_mode === "digital")) {
      clockMode = cfg.clock_mode;
    }
  } catch {
    clockMode = "digital";
  }
  registerTile({ id: "clock", title: "Clock", intervalMs: 1000, fetch: async () => null, render: renderClock });
  registerTile({ id: "date", title: "Date", intervalMs: 1000, fetch: async () => null, render: renderDate });
}

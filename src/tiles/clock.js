// Clock + Date tiles (frontend only). Clock graphic is an analog face or a big
// digital time, with a Show analog/digital toggle in the foot (persisted). Date is
// the weekday (graphic) over the full date (foot), with a mini month calendar.
//
// Both tiles tick every second but PATCH their DOM in place instead of rebuilding
// it: rebuilding wiped keyboard focus and hover off the toggle button 60x a minute,
// and rebuilt the whole month calendar for content that changes once a day.
import { getConfig, setConfig } from "../api.js";
import { tile } from "../layout.js";

let clockMode = "digital";

// Freeze the sweeping second hand for users who ask the OS for less motion.
const reduceMotion =
  typeof window !== "undefined" &&
  window.matchMedia?.("(prefers-reduced-motion: reduce)").matches;

// Static parts of the analog face: ring, ticks, numerals. Drawn once; only the
// hands' rotation is patched on each tick.
function analogFaceSvg() {
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
  const secondHand = reduceMotion
    ? ""
    : `<line data-hand="s" x1="${cx}" y1="${cy}" x2="${cx}" y2="${cy - 36}" ` +
      `stroke="var(--bad)" stroke-width="1" stroke-linecap="round"/>`;
  return (
    `<svg class="gauge" viewBox="0 0 100 100" role="img" aria-label="Analog clock">` +
    `<circle cx="${cx}" cy="${cy}" r="47" fill="none" stroke="currentColor" stroke-width="1.5" opacity="0.5"/>` +
    ticks +
    numerals +
    `<line data-hand="h" x1="${cx}" y1="${cy}" x2="${cx}" y2="${cy - 22}" stroke="currentColor" stroke-width="3" stroke-linecap="round"/>` +
    `<line data-hand="m" x1="${cx}" y1="${cy}" x2="${cx}" y2="${cy - 32}" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>` +
    secondHand +
    `<circle cx="${cx}" cy="${cy}" r="2.5" fill="currentColor"/></svg>`
  );
}

function patchAnalog(el, now) {
  const hours = now.getHours() % 12;
  const minutes = now.getMinutes();
  const seconds = now.getSeconds();
  const rotate = (hand, deg) =>
    el
      .querySelector(`[data-hand="${hand}"]`)
      ?.setAttribute("transform", `rotate(${deg.toFixed(2)} 50 50)`);
  rotate("h", hours * 30 + minutes * 0.5);
  rotate("m", minutes * 6);
  if (!reduceMotion) rotate("s", seconds * 6);
}

// Digital time split for a calm, glanceable hierarchy: hours:minutes big, the
// seconds and AM/PM small and dimmed beside them.
function timeParts(now) {
  const parts = new Intl.DateTimeFormat(undefined, {
    hour: "numeric",
    minute: "2-digit",
    second: "2-digit",
  }).formatToParts(now);
  const get = (t) => parts.find((p) => p.type === t)?.value ?? "";
  return {
    hm: `${get("hour")}:${get("minute")}`,
    rest: `${get("second")}${get("dayPeriod") ? " " + get("dayPeriod") : ""}`,
  };
}

function patchDigital(el, now) {
  const { hm, rest } = timeParts(now);
  const hmEl = el.querySelector("[data-clock-hm]");
  const restEl = el.querySelector("[data-clock-rest]");
  if (hmEl && hmEl.textContent !== hm) hmEl.textContent = hm;
  if (restEl && restEl.textContent !== rest) restEl.textContent = rest;
}

// Build the tile skeleton for the current mode; called only when the mode changes
// (or on first render), never on the per-second tick.
function buildClock(el) {
  const next = clockMode === "analog" ? "digital" : "analog";
  const graphic =
    clockMode === "analog"
      ? analogFaceSvg()
      : `<div class="tile-big"><span data-clock-hm></span>` +
        `<span class="tile-sub" data-clock-rest style="margin-left:6px"></span></div>`;
  el.innerHTML = tile({
    title: "Clock",
    graphic,
    foot: `<button class="tile-btn" type="button" data-action="toggle">Show ${next}</button>`,
  });
  el.dataset.clockMode = clockMode;
  el.querySelector('[data-action="toggle"]')?.addEventListener("click", () => {
    clockMode = next;
    setConfig({ clock_mode: clockMode });
    renderClock(el);
    el.querySelector('[data-action="toggle"]')?.focus(); // rebuild drops focus; restore
  });
}

function renderClock(el) {
  const now = new Date();
  if (el.dataset.clockMode !== clockMode) buildClock(el);
  if (clockMode === "analog") patchAnalog(el, now);
  else patchDigital(el, now);
}

// A mini calendar for the current month with today highlighted.
function calendarHtml(now) {
  const y = now.getFullYear();
  const m = now.getMonth();
  const firstDow = new Date(y, m, 1).getDay(); // 0 = Sunday
  const daysInMonth = new Date(y, m + 1, 0).getDate();
  const today = now.getDate();
  const monthLabel = now.toLocaleDateString(undefined, {
    month: "long",
    year: "numeric",
  });

  let cells = ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"]
    .map((d) => `<span class="cal-dow">${d}</span>`)
    .join("");
  for (let i = 0; i < firstDow; i++) cells += `<span class="cal-day"></span>`;
  for (let d = 1; d <= daysInMonth; d++) {
    const cls = d === today ? "cal-day cal-today" : "cal-day";
    cells += `<span class="${cls}">${d}</span>`;
  }
  return (
    `<div class="cal"><div class="cal-title">${monthLabel}</div>` +
    `<div class="cal-grid">${cells}</div></div>`
  );
}

// The date tile only changes at midnight, so rebuild only when the day changes
// (the 1s tick just compares a cached key).
function renderDate(el) {
  const now = new Date();
  const key = now.toDateString();
  if (el.dataset.dateKey === key) return;
  el.dataset.dateKey = key;
  const weekday = now.toLocaleDateString(undefined, { weekday: "long" });
  el.innerHTML =
    `<div class="tile-title">Date</div>` +
    `<div class="date-row">` +
    `<div class="date-left"><div class="tile-big">${weekday}</div>` +
    `<div class="tile-sub">${now.toLocaleDateString()}</div></div>` +
    calendarHtml(now) +
    `</div>`;
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

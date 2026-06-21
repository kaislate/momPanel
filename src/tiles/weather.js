// Weather tile: condition icon + temperature (graphic); condition word, high/low,
// and a "change location" link (foot). Uses read_weather(zip); first run prompts.
import { readWeather, getConfig, setConfig } from "../api.js";
import { promptZip } from "../firstrun.js";
import { refreshTile } from "../tiles.js";
import { tile, mutedGraphic } from "../layout.js";

function condition(code) {
  if (code === 0) return "clear";
  if (code >= 1 && code <= 3) return "cloudy";
  if (code === 45 || code === 48) return "fog";
  if (code >= 51 && code <= 67) return "rain";
  if (code >= 71 && code <= 86) return "snow";
  if (code >= 95 && code <= 99) return "thunder";
  return "cloudy";
}

function icon(cond, size = 56) {
  const open = `<svg class="wx-icon" viewBox="0 0 24 24" width="${size}" height="${size}" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" role="img">`;
  const cloud = `<path d="M7 18a4 4 0 0 1 0-8 5 5 0 0 1 9.6 1.3A3.5 3.5 0 0 1 16 18z"/>`;
  switch (cond) {
    case "clear":
      return `${open}<circle cx="12" cy="12" r="4"/><path d="M12 3v2M12 19v2M3 12h2M19 12h2M5.6 5.6l1.4 1.4M17 17l1.4 1.4M18.4 5.6L17 7M7 17l-1.4 1.4"/></svg>`;
    case "fog":
      return `${open}${cloud}<path d="M5 21h14M7 18h12" stroke-opacity="0.6"/></svg>`;
    case "rain":
      return `${open}${cloud}<path d="M8 20l-1 2M12 20l-1 2M16 20l-1 2"/></svg>`;
    case "snow":
      return `${open}${cloud}<path d="M8 20v.01M12 21v.01M16 20v.01M10 22v.01M14 22v.01"/></svg>`;
    case "thunder":
      return `${open}${cloud}<path d="M12 18l-2 3h3l-2 3"/></svg>`;
    case "cloudy":
    default:
      return `${open}${cloud}</svg>`;
  }
}

const round = (n) => Math.round(Number(n));

// Short weekday from a "YYYY-MM-DD" string, parsed in local time to avoid the UTC
// off-by-one. The first forecast day is labeled "Today".
function dayLabel(dateStr, index) {
  if (index === 0) return "Today";
  const [y, m, d] = String(dateStr).split("-").map(Number);
  if (!y || !m || !d) return "";
  return new Date(y, m - 1, d).toLocaleDateString(undefined, { weekday: "short" });
}

// Granular plain-language description from a WMO weather code. For wet conditions it
// appends the precipitation probability, e.g. "Rain 60%".
function describe(code, prob) {
  let base;
  if (code === 0) base = "Sunny";
  else if (code === 1) base = "Mostly Sunny";
  else if (code === 2) base = "Partly Cloudy";
  else if (code === 3) base = "Cloudy";
  else if (code === 45 || code === 48) base = "Fog";
  else if (code >= 51 && code <= 55) base = "Drizzle";
  else if (code === 56 || code === 57) base = "Icy Drizzle";
  else if (code === 61) base = "Light Rain";
  else if (code === 63) base = "Rain";
  else if (code === 65) base = "Heavy Rain";
  else if (code === 66 || code === 67) base = "Icy Rain";
  else if (code >= 71 && code <= 77) base = "Snow";
  else if (code >= 80 && code <= 82) base = "Showers";
  else if (code === 85 || code === 86) base = "Snow Showers";
  else if (code >= 95) base = "Storm";
  else base = "Cloudy";
  const wet = (code >= 51 && code <= 86) || code >= 95;
  return wet && prob > 0 ? `${base} ${prob}%` : base;
}

// One row of the 5-day forecast: day name | icon | description | high/low.
function forecastDay(day, index) {
  const cond = condition(day.code);
  return (
    `<div class="wx-day">` +
    `<div class="wx-day-name">${dayLabel(day.date, index)}</div>` +
    `${icon(cond, 26)}` +
    `<div class="wx-day-desc">${describe(day.code, day.precip_prob ?? 0)}</div>` +
    `<div class="wx-day-temp">${round(day.high_f)}&deg;` +
    `<span class="wx-day-low">${round(day.low_f)}&deg;</span></div></div>`
  );
}

function escapeHtml(s) {
  return String(s).replace(
    /[&<>"']/g,
    (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" }[c])
  );
}

export function register(registerTile) {
  registerTile({
    id: "weather",
    title: "Weather",
    intervalMs: 1200000, // 20 minutes
    async fetch() {
      let { zip } = await getConfig();
      if (!zip) {
        const entered = await promptZip("");
        if (entered) {
          zip = entered;
          await setConfig({ zip });
        }
      }
      return zip ? await readWeather(zip) : { state: "unavailable" };
    },
    render(el, data) {
      if (!data || data.state !== "ok") {
        el.innerHTML = tile({
          title: "Weather",
          graphic: mutedGraphic(icon("cloudy")),
          foot:
            `<div class="tile--unavailable">Weather isn't available right now.</div>` +
            `<a href="#" class="wx-change">set location</a>`,
        });
        wireChangeLink(el);
        return;
      }
      const cond = condition(data.code);
      const days = Array.isArray(data.days) ? data.days : [];
      const todayProb = days[0]?.precip_prob ?? 0;
      // Custom layout for this double-height tile: current conditions on top, the
      // 5-day forecast list below.
      el.innerHTML =
        `<div class="tile-title">${escapeHtml(data.place ?? "Weather")}</div>` +
        `<div class="wx-current">` +
        `<div class="wx-row">${icon(cond)}` +
        `<div class="tile-big">${round(data.temp_f)}&deg;</div></div>` +
        `<div class="tile-status">${describe(data.code, todayProb)}</div>` +
        `<div class="tile-sub">H ${round(data.high_f)}&deg; &middot; L ${round(
          data.low_f
        )}&deg;</div></div>` +
        `<div class="wx-forecast">${days
          .map((d, i) => forecastDay(d, i))
          .join("")}</div>` +
        `<a href="#" class="wx-change">change location</a>`;
      wireChangeLink(el);
    },
  });
}

function wireChangeLink(el) {
  const link = el.querySelector(".wx-change");
  if (!link) return;
  link.addEventListener("click", async (e) => {
    e.preventDefault();
    const { zip } = await getConfig();
    const entered = await promptZip(zip ?? "");
    if (entered) {
      await setConfig({ zip: entered });
      refreshTile("weather");
    }
  });
}

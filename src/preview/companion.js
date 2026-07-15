// Companion mode (experimental preview) — a reimagined momPanel.
//
// The classic panel is a dashboard: ten equal tiles, always showing everything.
// Companion mode flips the philosophy to STATUS BY EXCEPTION:
//   - Time, date, and weather (what mom actually glances at) are the heroes.
//   - The technical tiles (CPU, memory, storage, Wi-Fi, internet, printers, sound)
//     collapse into one calm "All is well" card. Each concern is a quiet row; the
//     panel only speaks up — a plain-language card with the right button — when
//     something genuinely needs attention.
// Same backend, same collectors, same cadences; only the presentation is new.
import {
  getTile,
  readWeather,
  getConfig,
  setConfig,
  openSettings,
  supportsTransparency,
  desktopBackground,
} from "../api.js";
import { listen } from "../bridge.js";
import { promptZip } from "../firstrun.js";
import { register as regMemory } from "../tiles/memory.js";
import { register as regStorage } from "../tiles/storage.js";
import { register as regWifi } from "../tiles/wifi.js";
import { register as regInternet } from "../tiles/internet.js";
import { register as regPrinters } from "../tiles/printers.js";
import { register as regVolume } from "../tiles/volume.js";

// Classic tile renderers, reused as hover "peek" cards on the health rows: each
// register() hands us its {id, render}; we keep the render functions and feed them
// the same data the health model already fetched.
const classicTiles = {};
[regMemory, regStorage, regWifi, regInternet, regPrinters, regVolume].forEach((r) =>
  r((t) => {
    classicTiles[t.id] = t;
  })
);

// ---------- Weather visuals (companion's own compact icon set) ----------

function condition(code) {
  if (code === 0) return "clear";
  if (code >= 1 && code <= 3) return "cloudy";
  if (code === 45 || code === 48) return "fog";
  if (code >= 51 && code <= 67) return "rain";
  if (code >= 71 && code <= 86) return "snow";
  if (code >= 95 && code <= 99) return "thunder";
  return "cloudy";
}

function wxIcon(cond, size = 64) {
  const open = `<svg class="wx-icon" viewBox="0 0 24 24" width="${size}" height="${size}" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">`;
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
    default:
      return `${open}${cloud}</svg>`;
  }
}

function describe(code) {
  if (code === 0) return "Sunny";
  if (code === 1) return "Mostly Sunny";
  if (code === 2) return "Partly Cloudy";
  if (code === 3) return "Cloudy";
  if (code === 45 || code === 48) return "Foggy";
  if (code >= 51 && code <= 57) return "Drizzly";
  if (code >= 61 && code <= 67) return "Rainy";
  if (code >= 71 && code <= 77) return "Snowy";
  if (code >= 80 && code <= 82) return "Showers";
  if (code === 85 || code === 86) return "Snow Showers";
  if (code >= 95) return "Stormy";
  return "Cloudy";
}

const round = (n) => Math.round(Number(n));

// ---------- Time-of-day: greeting + sky ----------

function greetingFor(h) {
  if (h < 5) return "Good night"; // the wee hours aren't "evening"
  if (h < 12) return "Good morning";
  if (h < 17) return "Good afternoon";
  return "Good evening";
}

function skyFor(h) {
  if (h >= 5 && h < 8) return "dawn";
  if (h >= 8 && h < 17) return "day";
  if (h >= 17 && h < 20) return "dusk";
  return "night";
}

// ---------- The health model: each concern turns tile data into a quiet row, and
// (only when needed) an attention card with a calm sentence and the right button.
// level: "ok" | "warn" | "bad" | "off" (off = neutral gray, nothing to worry about)

const CONCERNS = [
  {
    id: "internet",
    label: "Internet",
    periodMs: 20000,
    evaluate(d) {
      if (!d || d.state !== "ok") return { level: "off", word: "—" };
      if (d.online === true) return { level: "ok", word: "Online" };
      return {
        level: "bad",
        word: "Offline",
        attention: {
          text:
            "You're offline right now. This usually fixes itself in a minute or two. " +
            "If it doesn't, try unplugging the internet box, counting to ten, and plugging it back in.",
          action: { label: "Open Wi-Fi settings", target: "wifi" },
        },
      };
    },
  },
  {
    id: "wifi",
    label: "Wi-Fi",
    periodMs: 20000,
    evaluate(d, ctx) {
      if (!d || d.state !== "ok") {
        // No Wi-Fi but the internet works → she's on a cable; that's fine, say so.
        if (ctx.internet?.online === true) return { level: "ok", word: "Using a cable" };
        return { level: "off", word: "—" };
      }
      const p = typeof d.signal_percent === "number" ? d.signal_percent : null;
      if (p === null) return { level: "ok", word: "Connected" };
      if (p >= 60) return { level: "ok", word: "Strong" };
      if (p >= 34) return { level: "ok", word: "Okay" };
      return {
        level: "warn",
        word: "Weak",
        attention: {
          text: "The Wi-Fi signal is weak here. Sitting closer to the internet box can help.",
          action: { label: "Open Wi-Fi settings", target: "wifi" },
        },
      };
    },
  },
  {
    id: "printers",
    label: "Printer",
    periodMs: 30000,
    evaluate(d) {
      if (!d || d.state === "unavailable") return { level: "off", word: "—" };
      const list = Array.isArray(d.printers) ? d.printers : [];
      if (list.length === 0) return { level: "off", word: "None" };
      // Judge by the default printer when there is one, else the first.
      const p = list.find((x) => x.name === d.default_name) || list[0];
      // Low ink only matters when the printer is otherwise fine — paper/power first.
      const lowInks =
        p.status === "ready" && Array.isArray(d.inks) ? d.inks.filter((i) => i.low) : [];
      if (lowInks.length > 0) {
        const which = lowInks.map((i) => String(i.name).toLowerCase()).join(" and ");
        return {
          level: "warn",
          word: "Ink low",
          attention: {
            text: `The printer is getting low on ${which}. It still prints — just worth topping up soon.`,
          },
        };
      }
      if (p.status === "ready") return { level: "ok", word: "Ready" };
      if (p.status === "out_of_paper")
        return {
          level: "warn",
          word: "No paper",
          attention: {
            text: "The printer is out of paper. Add paper and it will carry on by itself.",
            action: { label: "Open printer settings", target: "printers" },
          },
        };
      if (p.status === "offline")
        return {
          level: "warn",
          word: "Off",
          attention: {
            text: "The printer looks switched off or unplugged. Check its power button and cable.",
            action: { label: "Open printer settings", target: "printers" },
          },
        };
      return { level: "ok", word: "Connected" };
    },
  },
  {
    id: "volume",
    label: "Sound",
    periodMs: 5000,
    evaluate(d) {
      if (!d || d.state !== "ok") return { level: "off", word: "—" };
      if (d.muted)
        return {
          level: "warn",
          word: "Off",
          attention: {
            text: "The sound is switched off, so videos and calls will be silent.",
            action: { label: "Open sound settings", target: "sound" },
          },
        };
      return { level: "ok", word: `On · ${round(d.level_percent)}%` };
    },
  },
  {
    id: "memory",
    label: "Speed",
    periodMs: 5000,
    evaluate(d, ctx) {
      if (!d || d.state !== "ok") return { level: "off", word: "—" };
      const p = d.used_percent;
      const warnAt = ctx.memWarnPercent;
      if (p >= warnAt)
        return {
          level: "bad",
          word: "Very busy",
          attention: {
            text:
              "The computer is working very hard right now. Closing windows you're not " +
              "using — or restarting the computer — will make it happier.",
          },
        };
      if (p >= 70) return { level: "ok", word: "Working hard" };
      return { level: "ok", word: "Fine" };
    },
  },
  {
    id: "storage",
    label: "Space",
    periodMs: 60000,
    evaluate(d) {
      if (!d || d.state !== "ok") return { level: "off", word: "—" };
      const p = d.used_percent;
      if (p >= 90)
        return {
          level: "warn",
          word: "Almost full",
          attention: {
            text: "The computer is nearly full. A family member can help clear some space.",
            action: { label: "Open storage", target: "storage" },
          },
        };
      if (p >= 70) return { level: "ok", word: "Getting full" };
      return { level: "ok", word: "Plenty" };
    },
  },
];

function heartSvg() {
  return (
    `<svg class="comp-health-glyph" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">` +
    `<path d="M12 21C8 18 2.5 13.7 2.1 9.4 1.8 6.4 4 4 6.8 4c2 0 3.8 1.1 4.7 2.8h1C13.4 5.1 15.2 4 17.2 4 20 4 22.2 6.4 21.9 9.4 21.5 13.7 16 18 12 21z"/>` +
    `</svg>`
  );
}

// ---------- Rendering ----------

const state = {
  data: {}, // concern id -> latest tile payload
  due: {}, // concern id -> next fetch timestamp
  memWarnPercent: 85,
  els: {},
  lastAttentionKey: "",
  lastWeather: null, // last good payload + timestamp, so a blip never blanks the hero
};

function buildSkeleton(main) {
  main.className = "comp";
  main.innerHTML =
    `<div class="comp-hero">` +
    `<div class="comp-greeting"></div>` +
    `<div class="comp-time"><span class="comp-hm"></span><span class="comp-ampm"></span></div>` +
    `<div class="comp-date"></div>` +
    `<div class="comp-weather"></div>` +
    `</div>` +
    `<div class="comp-health">` +
    `<div class="comp-health-head">${heartSvg()}<div class="comp-health-title"></div></div>` +
    CONCERNS.map(
      (c) =>
        `<div class="comp-health-row" data-concern="${c.id}">` +
        `<span class="comp-dot"></span>` +
        `<span class="comp-row-label">${c.label}</span>` +
        `<span class="comp-row-word">—</span></div>`
    ).join("") +
    `</div>` +
    `<div class="comp-attention"></div>`;

  state.els = {
    greeting: main.querySelector(".comp-greeting"),
    hm: main.querySelector(".comp-hm"),
    ampm: main.querySelector(".comp-ampm"),
    date: main.querySelector(".comp-date"),
    weather: main.querySelector(".comp-weather"),
    healthHead: main.querySelector(".comp-health-head"),
    healthTitle: main.querySelector(".comp-health-title"),
    attention: main.querySelector(".comp-attention"),
    rows: {},
  };
  CONCERNS.forEach((c) => {
    const row = main.querySelector(`[data-concern="${c.id}"]`);
    state.els.rows[c.id] = { dot: row.querySelector(".comp-dot"), word: row.querySelector(".comp-row-word") };
  });
}

// Clock/date/sky: patch text in place every second — nothing is rebuilt, so there is
// no flicker and no focus loss (the lesson learned from the classic clock tile).
let lastDateKey = "";
function tickClock() {
  const now = new Date();
  const parts = new Intl.DateTimeFormat(undefined, {
    hour: "numeric",
    minute: "2-digit",
  }).formatToParts(now);
  const ampm = parts.find((p) => p.type === "dayPeriod")?.value ?? "";
  const hm = parts
    .filter((p) => p.type !== "dayPeriod")
    .map((p) => p.value)
    .join("")
    .trim();
  if (state.els.hm.textContent !== hm) state.els.hm.textContent = hm;
  if (state.els.ampm.textContent !== ampm) state.els.ampm.textContent = ampm;

  const h = now.getHours();
  const greeting = greetingFor(h);
  if (state.els.greeting.textContent !== greeting) state.els.greeting.textContent = greeting;
  const sky = skyFor(h);
  if (document.body.dataset.sky !== sky) document.body.dataset.sky = sky;

  const dateKey = now.toDateString();
  if (dateKey !== lastDateKey) {
    lastDateKey = dateKey;
    state.els.date.textContent = now.toLocaleDateString(undefined, {
      weekday: "long",
      month: "long",
      day: "numeric",
    });
  }
}

function renderWeather(data) {
  const el = state.els.weather;
  if (data && data.state === "ok") {
    state.lastWeather = { data, at: new Date() };
  }
  const shown = data && data.state === "ok" ? data : state.lastWeather?.data;

  if (!shown) {
    // Nothing to show yet: either no ZIP (offer setup) or the first fetch failed.
    el.innerHTML =
      `<div class="comp-wx-note">I can show the weather once I know where you are.</div>` +
      `<button class="tile-btn comp-wx-setup" type="button" data-wx-setup>Set up weather</button>`;
    el.querySelector("[data-wx-setup]").addEventListener("click", changeLocation);
    return;
  }

  const cond = condition(shown.code);
  const days = (Array.isArray(shown.days) ? shown.days : []).slice(1, 5);
  const stale = !(data && data.state === "ok");
  const staleNote = stale && state.lastWeather
    ? `as of ${state.lastWeather.at.toLocaleTimeString([], { hour: "numeric", minute: "2-digit" })} · `
    : "";

  el.innerHTML =
    `<div class="comp-wx-now">${wxIcon(cond)}` +
    `<div class="comp-wx-temp">${round(shown.temp_f)}&deg;</div>` +
    `<div class="comp-wx-words">` +
    `<div class="comp-wx-cond">${describe(shown.code)}</div>` +
    `<div class="comp-wx-hilo">High ${round(shown.high_f)}&deg; &middot; Low ${round(shown.low_f)}&deg;</div>` +
    `</div></div>` +
    `<div class="comp-wx-days">` +
    days
      .map((d) => {
        const [y, m, dd] = String(d.date).split("-").map(Number);
        const name = y ? new Date(y, m - 1, dd).toLocaleDateString(undefined, { weekday: "short" }) : "";
        return (
          `<div class="comp-wx-day"><span class="comp-wx-day-name">${name}</span>` +
          wxIcon(condition(d.code), 26) +
          `<span class="comp-wx-day-temp">${round(d.high_f)}&deg;<span class="lo">${round(d.low_f)}&deg;</span></span></div>`
        );
      })
      .join("") +
    `</div>` +
    `<div class="comp-wx-note">${staleNote}<a data-wx-change>change location</a></div>`;
  el.querySelector("[data-wx-change]").addEventListener("click", changeLocation);
}

async function changeLocation() {
  const { zip } = await getConfig();
  const entered = await promptZip(zip ?? "");
  if (entered) {
    await setConfig({ zip: entered });
    fetchWeather();
  }
}

async function fetchWeather() {
  const { zip } = await getConfig();
  const data = zip ? await readWeather(zip) : { state: "unavailable" };
  renderWeather(data);
}

function renderHealth() {
  const results = {};
  const ctx = { internet: state.data.internet, memWarnPercent: state.memWarnPercent };
  CONCERNS.forEach((c) => {
    results[c.id] = c.evaluate(state.data[c.id], ctx);
  });

  // Rows: dot color + word, patched in place.
  CONCERNS.forEach((c) => {
    const r = results[c.id];
    const els = state.els.rows[c.id];
    els.dot.className = `comp-dot ${r.level === "off" ? "" : r.level}`.trim();
    if (els.word.textContent !== r.word) els.word.textContent = r.word;
  });

  // Header: the worst level wins.
  const levels = Object.values(results).map((r) => r.level);
  const worst = levels.includes("bad") ? "bad" : levels.includes("warn") ? "warn" : "ok";
  const titles = { ok: "All is well", warn: "Worth a look", bad: "Needs attention" };
  const colors = { ok: "var(--ok)", warn: "var(--warn)", bad: "var(--bad)" };
  state.els.healthTitle.textContent = titles[worst];
  state.els.healthHead.querySelector(".comp-health-glyph").style.color = colors[worst];

  // Attention cards: rebuilt only when the set of concerns actually changes, so an
  // open card's button is never yanked out from under a click.
  const attns = CONCERNS.filter((c) => results[c.id].attention).map((c) => ({
    id: c.id,
    level: results[c.id].level,
    ...results[c.id].attention,
  }));
  const key = attns.map((a) => a.id + a.level).join("|");
  if (key === state.lastAttentionKey) return;
  state.lastAttentionKey = key;

  const box = state.els.attention;
  clearTimeout(state._attnClear);

  if (attns.length === 0) {
    // Collapse gracefully first (the row's max-height animates closed while the
    // resolved cards are still inside), then drop the DOM once it's hidden.
    box.classList.remove("has-cards");
    state._attnClear = setTimeout(() => {
      if (!box.classList.contains("has-cards")) box.innerHTML = "";
    }, 550);
    return;
  }

  box.innerHTML = attns
    .map(
      (a) =>
        `<div class="comp-card ${a.level}" data-card="${a.id}">` +
        `<div class="comp-card-text">${a.text}</div>` +
        (a.action
          ? `<button class="tile-btn" type="button" data-target="${a.action.target}">${a.action.label}</button>`
          : "") +
        `</div>`
    )
    .join("");
  box.querySelectorAll("[data-target]").forEach((btn) =>
    btn.addEventListener("click", () => openSettings(btn.dataset.target))
  );
  // Next frame so the max-height transition runs (content above glides up).
  requestAnimationFrame(() => box.classList.add("has-cards"));
}

// ---------- Peek cards: hovering a health row shows the classic tile ----------

function initPeek() {
  const peek = document.createElement("div");
  peek.className = "tile comp-peek";
  peek.hidden = true;
  document.body.appendChild(peek);

  let hideTimer = 0;
  let showTimer = 0;
  const hide = () => {
    peek.hidden = true;
    peek.innerHTML = "";
  };
  const scheduleHide = () => {
    clearTimeout(hideTimer);
    hideTimer = setTimeout(hide, 250);
  };
  // Moving onto the peek itself keeps it open (its buttons are clickable).
  peek.addEventListener("mouseenter", () => clearTimeout(hideTimer));
  peek.addEventListener("mouseleave", scheduleHide);
  document.addEventListener("keydown", (e) => {
    if (e.key === "Escape") hide();
  });

  CONCERNS.forEach((c) => {
    const row = document.querySelector(`[data-concern="${c.id}"]`);
    const tileMod = classicTiles[c.id];
    if (!row || !tileMod) return;
    row.addEventListener("mouseenter", () => {
      clearTimeout(hideTimer);
      clearTimeout(showTimer);
      // Small delay so skimming the list doesn't strobe cards.
      showTimer = setTimeout(() => {
        try {
          tileMod.render(peek, state.data[c.id]);
        } catch {
          return;
        }
        peek.hidden = false;
        // Place beside the health card, vertically near the hovered row, on-screen.
        const rowR = row.getBoundingClientRect();
        const cardR = row.closest(".comp-health").getBoundingClientRect();
        const left = Math.max(12, cardR.left - peek.offsetWidth - 14);
        const top = Math.min(
          Math.max(12, rowR.top - peek.offsetHeight / 2),
          window.innerHeight - peek.offsetHeight - 12
        );
        peek.style.left = `${left}px`;
        peek.style.top = `${top}px`;
      }, 200);
    });
    row.addEventListener("mouseleave", () => {
      clearTimeout(showTimer);
      scheduleHide();
    });
  });
}

// ---------- Data loop: one 5s scheduler, per-concern periods, paused when hidden ----

async function refreshConcern(c) {
  state.data[c.id] = await getTile(c.id);
  renderHealth();
}

function dueNow(nowMs) {
  return CONCERNS.filter((c) => (state.due[c.id] ?? 0) <= nowMs);
}

async function tickData() {
  if (document.hidden) return;
  const nowMs = Date.now();
  const due = dueNow(nowMs);
  due.forEach((c) => (state.due[c.id] = nowMs + c.periodMs));
  await Promise.all(due.map(refreshConcern));
}

export async function initCompanion() {
  const cfg = await getConfig();
  state.memWarnPercent = Math.round(cfg.mem_warn_percent || 85);

  // Companion has its own stylesheet, loaded only in this mode.
  const link = document.createElement("link");
  link.rel = "stylesheet";
  link.href = "preview/companion.css";
  document.head.appendChild(link);
  // Both roots take the class: html must go transparent too or it paints an opaque
  // backdrop behind the translucent body sky.
  document.documentElement.classList.add("companion");
  document.body.classList.add("companion");

  // User-tunable sky opacity (About → General), down to fully invisible.
  let alpha = Math.min(1, Math.max(0, Number(cfg.companion_bg_opacity ?? 1)));

  // Real window transparency ghosts and breaks input on Linux/WebKitGTK, so there
  // the window stays opaque and "clear" skies reveal a drawn copy of the desktop
  // wallpaper instead — the desktop, never other windows.
  if (!(await supportsTransparency())) {
    // Load the wallpaper even at "Solid" so the About dropdown can reveal it live.
    const wall = await desktopBackground();
    if (wall) {
      const desk = document.createElement("div");
      desk.className = "comp-desktop";
      desk.style.backgroundImage = `url(${wall})`;
      document.body.prepend(desk);
    } else if (alpha < 1) {
      alpha = 1; // nothing to reveal — keep the sky solid rather than paint white
    }
  }
  document.documentElement.style.setProperty("--comp-bg-alpha", String(alpha));

  buildSkeleton(document.getElementById("grid"));
  // Solid readability panels behind the hero and/or health card (About → General).
  document.querySelector(".comp-hero")?.classList.toggle("comp-solid", !!cfg.companion_solid_hero);
  document.querySelector(".comp-health")?.classList.toggle("comp-solid", !!cfg.companion_solid_health);
  initPeek();

  tickClock();
  setInterval(tickClock, 1000);

  tickData();
  setInterval(tickData, 5000);
  setInterval(() => {
    if (!document.hidden) fetchWeather();
  }, 1200000); // weather every 20 minutes, same as classic
  fetchWeather();

  document.addEventListener("visibilitychange", () => {
    if (!document.hidden) tickData();
  });
  // Instant network refresh on the backend's NetworkManager D-Bus push.
  await listen("net-changed", () => {
    state.due.internet = 0;
    state.due.wifi = 0;
    tickData();
  });
}

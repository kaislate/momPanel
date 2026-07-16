// Bottom-right control cluster: an eye toggle (hide/show all buttons & links for a
// clean info-only view) plus A-/A+ size steps. Both persist in config.
//
// "Make everything bigger" scales the whole panel uniformly: the root font-size grows
// (so all rem-based sizing grows with it) and the window is resized to match, keeping
// the identical layout — just bigger.
import { getConfig, setConfig } from "./api.js";
import { openInfo } from "./info.js";

const STEPS = ["normal", "big", "biggest"];
// Base window size at scale "normal" (must match tauri.conf.json). 760 tall gives
// the ten-tile grid the height its content actually needs; on screens too short for
// a step, fittedFactor() shrinks the window and font together, so nothing overlaps.
// Companion mode shows less at once, so it asks for a smaller canvas via
// setPanelBase() before the first resize.
const BASE = { w: 1100, h: 760 };
const FACTORS = { normal: 1, big: 19 / 16, biggest: 22 / 16 };

export function setPanelBase(w, h) {
  BASE.w = w;
  BASE.h = h;
}

function clampScale(s) {
  return STEPS.includes(s) ? s : "normal";
}

// Cap the scale factor so the window always fits the screen: on a small laptop,
// "biggest" (1512x935) would otherwise grow past the edges. Best-effort — if the
// monitor can't be read, the requested factor is used unchanged.
async function fittedFactor(f) {
  const winApi = window.__TAURI__?.window;
  try {
    const mon = winApi?.currentMonitor ? await winApi.currentMonitor() : null;
    if (!mon) return f;
    const sf = mon.scaleFactor || 1;
    // workArea (excludes taskbars/docks) where the runtime provides it; otherwise
    // full monitor size minus a margin so a taskbar never hides the panel's edge.
    const area = mon.workArea?.size ?? mon.size;
    const margin = mon.workArea ? 0 : 80;
    return Math.min(f, (area.width / sf - margin) / BASE.w, (area.height / sf - margin) / BASE.h);
  } catch {
    return f;
  }
}

// Grow/shrink the OS window to match the chosen size (best-effort). Deliberately
// does NOT re-center: the window position is user-chosen and remembered by the
// backend, and re-centering on every size change would fight that.
async function resizeWindow(f) {
  const tauri = window.__TAURI__;
  const winApi = tauri?.window;
  const LogicalSize = tauri?.dpi?.LogicalSize ?? winApi?.LogicalSize;
  if (!winApi || !LogicalSize) return;
  try {
    const win = winApi.getCurrentWindow();
    await win.setSize(new LogicalSize(Math.round(BASE.w * f), Math.round(BASE.h * f)));
  } catch {
    /* window resize unavailable; font scaling still applies */
  }
}

// When the screen can't fit the requested step, the font scale shrinks to the
// same capped factor so the fixed 4-column layout still fits the smaller window.
async function fitAndResize(s) {
  const requested = FACTORS[s] ?? 1;
  const f = await fittedFactor(requested);
  document.documentElement.style.fontSize =
    f < requested - 0.001 ? `${Math.round(16 * f * 10) / 10}px` : "";
  await resizeWindow(f);
}

export function applyScale(scale) {
  const s = clampScale(scale);
  const root = document.documentElement;
  STEPS.forEach((step) => root.classList.remove(`scale-${step}`));
  root.classList.add(`scale-${s}`);
  fitAndResize(s); // fire-and-forget
  return s;
}

// Hide/show all buttons and links (the "chrome") via one body class.
export function applyHidden(hidden) {
  document.body.classList.toggle("controls-hidden", !!hidden);
  return !!hidden;
}

// Set help-dot visibility with NO animation (used on boot to avoid a flash).
function applyHelpHidden(hidden) {
  document.body.classList.toggle("help-hidden", !!hidden);
}

// Hide the "?" dots with a spin-and-poof, then remove them.
function poofHelpDots() {
  const dots = document.querySelectorAll(".tile-help");
  dots.forEach((d) => {
    d.classList.remove("tile-help--appear");
    d.classList.add("tile-help--poof");
  });
  setTimeout(() => {
    document.body.classList.add("help-hidden");
    dots.forEach((d) => d.classList.remove("tile-help--poof"));
  }, 450);
}

// Show the "?" dots: pop them back in and pulse-glow a few times.
function popHelpDots() {
  document.body.classList.remove("help-hidden");
  document.querySelectorAll(".tile-help").forEach((d) => {
    d.classList.remove("tile-help--poof", "tile-help--appear");
    void d.offsetWidth; // restart the animation
    d.classList.add("tile-help--appear");
  });
}

// Read saved chrome state and apply it before tiles render (avoids a flash).
export async function initChrome() {
  let scale = "normal";
  let hidden = false;
  let helpHidden = false;
  try {
    const cfg = await getConfig();
    scale = clampScale(cfg.ui_scale);
    hidden = !!cfg.hide_controls;
    helpHidden = !!cfg.hide_help;
  } catch {
    /* defaults */
  }
  applyScale(scale);
  applyHidden(hidden);
  applyHelpHidden(helpHidden);
  return { scale, hidden, helpHidden };
}

const EYE_OPEN =
  `<svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" ` +
  `stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">` +
  `<path d="M2 12s3.5-7 10-7 10 7 10 7-3.5 7-10 7-10-7-10-7z"/><circle cx="12" cy="12" r="3"/></svg>`;
const EYE_OFF =
  `<svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" ` +
  `stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">` +
  `<path d="M2 12s3.5-7 10-7c2 0 3.8.6 5.3 1.5M22 12s-3.5 7-10 7c-2 0-3.8-.6-5.3-1.5"/>` +
  `<path d="M3 3l18 18"/></svg>`;

const GEAR =
  `<svg viewBox="0 0 24 24" width="17" height="17" fill="none" stroke="currentColor" ` +
  `stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">` +
  `<circle cx="12" cy="12" r="3.2"/>` +
  `<path d="M19.4 15a1.7 1.7 0 0 0 .34 1.87l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.7 1.7 0 0 0-1.87-.34 1.7 1.7 0 0 0-1.03 1.56V21a2 2 0 1 1-4 0v-.09a1.7 1.7 0 0 0-1.11-1.56 1.7 1.7 0 0 0-1.87.34l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.7 1.7 0 0 0 .34-1.87 1.7 1.7 0 0 0-1.56-1.03H3a2 2 0 1 1 0-4h.09a1.7 1.7 0 0 0 1.56-1.11 1.7 1.7 0 0 0-.34-1.87l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.7 1.7 0 0 0 1.87.34h.01A1.7 1.7 0 0 0 10 3.09V3a2 2 0 1 1 4 0v.09a1.7 1.7 0 0 0 1.03 1.56h.01a1.7 1.7 0 0 0 1.87-.34l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.7 1.7 0 0 0-.34 1.87v.01A1.7 1.7 0 0 0 20.91 10H21a2 2 0 1 1 0 4h-.09a1.7 1.7 0 0 0-1.56 1.03z"/></svg>`;

// Render the controls corner: a single faint gear that expands into the
// i / ? / eye / A-/A+ tray on demand, so the controls never crowd the panel.
export function mountControls({ scale, hidden, helpHidden }) {
  let cur = clampScale(scale);
  let isHidden = !!hidden;
  let helpOff = !!helpHidden;

  const wrap = document.createElement("div");
  wrap.className = "scale-control";
  wrap.innerHTML =
    `<div class="scale-tray" hidden>` +
    `<button type="button" class="scale-btn scale-btn--info" data-info aria-label="About momPanel">i</button>` +
    `<button type="button" class="scale-btn scale-btn--help" data-help-toggle aria-label="Show or hide the help dots">?</button>` +
    `<button type="button" class="scale-btn" data-eye aria-label="Hide or show buttons"></button>` +
    `<button type="button" class="scale-btn" data-step="-1" aria-label="Make text smaller">A&minus;</button>` +
    `<button type="button" class="scale-btn" data-step="1" aria-label="Make text bigger">A+</button>` +
    `</div>` +
    `<button type="button" class="scale-btn scale-btn--launcher" data-launcher ` +
    `aria-label="Panel controls" aria-expanded="false">${GEAR}</button>`;
  document.body.appendChild(wrap);

  // Expand/collapse the tray. Collapsed is the resting state so the corner stays
  // out of the way; clicking anywhere else tucks it back in.
  const tray = wrap.querySelector(".scale-tray");
  const launcher = wrap.querySelector("[data-launcher]");
  const setOpen = (open) => {
    tray.hidden = !open;
    wrap.classList.toggle("is-open", open);
    launcher.setAttribute("aria-expanded", String(open));
  };
  launcher.addEventListener("click", () => setOpen(tray.hidden));
  document.addEventListener("click", (e) => {
    if (!tray.hidden && !wrap.contains(e.target)) setOpen(false);
  });

  wrap.querySelector("[data-info]").addEventListener("click", () => openInfo());

  const helpBtn = wrap.querySelector("[data-help-toggle]");
  helpBtn.classList.toggle("is-off", helpOff);
  helpBtn.addEventListener("click", () => {
    helpOff = !helpOff;
    helpBtn.classList.toggle("is-off", helpOff);
    if (helpOff) poofHelpDots();
    else popHelpDots();
    setConfig({ hide_help: helpOff });
  });

  const eyeBtn = wrap.querySelector("[data-eye]");
  const paintEye = () => {
    eyeBtn.innerHTML = isHidden ? EYE_OFF : EYE_OPEN;
    eyeBtn.setAttribute("aria-pressed", String(isHidden));
  };
  paintEye();
  eyeBtn.addEventListener("click", () => {
    isHidden = !isHidden;
    applyHidden(isHidden);
    paintEye();
    setConfig({ hide_controls: isHidden });
  });

  const step = (delta) => {
    const i = Math.min(STEPS.length - 1, Math.max(0, STEPS.indexOf(cur) + delta));
    if (STEPS[i] === cur) return;
    cur = STEPS[i];
    applyScale(cur);
    setConfig({ ui_scale: cur });
  };
  wrap
    .querySelectorAll("[data-step]")
    .forEach((btn) => btn.addEventListener("click", () => step(Number(btn.dataset.step))));
}

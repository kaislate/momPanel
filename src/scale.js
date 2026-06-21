// Bottom-right control cluster: an eye toggle (hide/show all buttons & links for a
// clean info-only view) plus A-/A+ size steps. Both persist in config.
//
// "Make everything bigger" scales the whole panel uniformly: the root font-size grows
// (so all rem-based sizing grows with it) and the window is resized to match, keeping
// the identical layout — just bigger.
import { getConfig, setConfig } from "./api.js";
import { openInfo } from "./info.js";

const STEPS = ["normal", "big", "biggest"];
const BASE_W = 1100;
const BASE_H = 680;
const FACTORS = { normal: 1, big: 19 / 16, biggest: 22 / 16 };

function clampScale(s) {
  return STEPS.includes(s) ? s : "normal";
}

// Grow/shrink the OS window to match the chosen size (best-effort).
async function resizeWindow(scale) {
  const tauri = window.__TAURI__;
  const winApi = tauri?.window;
  const LogicalSize = tauri?.dpi?.LogicalSize ?? winApi?.LogicalSize;
  if (!winApi || !LogicalSize) return;
  try {
    const f = FACTORS[scale] ?? 1;
    const win = winApi.getCurrentWindow();
    await win.setSize(new LogicalSize(Math.round(BASE_W * f), Math.round(BASE_H * f)));
    await win.center();
  } catch {
    /* window resize unavailable; font scaling still applies */
  }
}

export function applyScale(scale) {
  const s = clampScale(scale);
  const root = document.documentElement;
  STEPS.forEach((step) => root.classList.remove(`scale-${step}`));
  root.classList.add(`scale-${s}`);
  resizeWindow(s); // fire-and-forget
  return s;
}

// Hide/show all buttons and links (the "chrome") via one body class.
export function applyHidden(hidden) {
  document.body.classList.toggle("controls-hidden", !!hidden);
  return !!hidden;
}

// Read saved chrome state and apply it before tiles render (avoids a flash).
export async function initChrome() {
  let scale = "normal";
  let hidden = false;
  try {
    const cfg = await getConfig();
    scale = clampScale(cfg.ui_scale);
    hidden = !!cfg.hide_controls;
  } catch {
    /* defaults */
  }
  applyScale(scale);
  applyHidden(hidden);
  return { scale, hidden };
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

// Render the eye + A-/A+ pill and wire the behaviors.
export function mountControls({ scale, hidden }) {
  let cur = clampScale(scale);
  let isHidden = !!hidden;

  const wrap = document.createElement("div");
  wrap.className = "scale-control";
  wrap.innerHTML =
    `<button type="button" class="scale-btn scale-btn--info" data-info aria-label="About momPanel">i</button>` +
    `<button type="button" class="scale-btn" data-eye aria-label="Hide or show buttons"></button>` +
    `<button type="button" class="scale-btn" data-step="-1" aria-label="Make text smaller">A&minus;</button>` +
    `<button type="button" class="scale-btn" data-step="1" aria-label="Make text bigger">A+</button>`;
  document.body.appendChild(wrap);

  wrap.querySelector("[data-info]").addEventListener("click", () => openInfo());

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

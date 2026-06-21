// "Make everything bigger" size control. Three steps (normal/big/biggest) drive a
// root font-size + grid column count via a class on <html>, so all rem-based text,
// padding, gauges, and buttons scale together and bigger text gets wider tiles
// instead of cramped ones. The choice persists in config (like clock mode).
import { getConfig, setConfig } from "./api.js";

const STEPS = ["normal", "big", "biggest"];

// Base window size (matches tauri.conf.json) and the per-step scale factors. The
// window grows by the same factor as the font, so the 4-column layout stays identical
// — just larger — instead of reflowing or clipping.
const BASE_W = 1100;
const BASE_H = 680;
const FACTORS = { normal: 1, big: 19 / 16, biggest: 22 / 16 };

function clampScale(s) {
  return STEPS.includes(s) ? s : "normal";
}

// Grow/shrink the OS window to match the chosen size. Best-effort: if the window API
// or permission is unavailable, the font still scales (content just stays in the
// original window).
async function resizeWindow(scale) {
  const tauri = window.__TAURI__;
  const winApi = tauri?.window;
  // LogicalSize lives in the dpi namespace in Tauri v2 (older/global builds expose it
  // on the window namespace too) — accept either.
  const LogicalSize = tauri?.dpi?.LogicalSize ?? winApi?.LogicalSize;
  if (!winApi || !LogicalSize) return;
  try {
    const f = FACTORS[scale] ?? 1;
    const w = Math.round(BASE_W * f);
    const h = Math.round(BASE_H * f);
    const win = winApi.getCurrentWindow();
    await win.setSize(new LogicalSize(w, h));
    await win.center();
  } catch {
    /* window resize not available; font scaling still applies */
  }
}

// Apply the scale: swap the single scale-* class on <html> (font) and resize the
// window to match (layout-preserving).
export function applyScale(scale) {
  const s = clampScale(scale);
  const root = document.documentElement;
  STEPS.forEach((step) => root.classList.remove(`scale-${step}`));
  root.classList.add(`scale-${s}`);
  resizeWindow(s); // fire-and-forget
  return s;
}

// Read the saved scale and apply it immediately (call this first, before tiles mount,
// to avoid a flash of the wrong size).
export async function initScale() {
  let scale = "normal";
  try {
    scale = clampScale((await getConfig()).ui_scale);
  } catch {
    scale = "normal";
  }
  applyScale(scale);
  return scale;
}

// Render the fixed A-/A+ pill and wire it to step the size and persist it.
export function mountScaleControl(initialScale) {
  let scale = clampScale(initialScale);

  const wrap = document.createElement("div");
  wrap.className = "scale-control";
  wrap.innerHTML =
    `<button type="button" class="scale-btn" data-step="-1" aria-label="Make text smaller">A&minus;</button>` +
    `<button type="button" class="scale-btn" data-step="1" aria-label="Make text bigger">A+</button>`;
  document.body.appendChild(wrap);

  const update = (delta) => {
    const i = Math.min(
      STEPS.length - 1,
      Math.max(0, STEPS.indexOf(scale) + delta)
    );
    const next = STEPS[i];
    if (next === scale) return;
    scale = next;
    applyScale(scale);
    setConfig({ ui_scale: scale }); // persist; fire-and-forget
  };

  wrap.querySelectorAll(".scale-btn").forEach((btn) => {
    btn.addEventListener("click", () =>
      update(Number(btn.dataset.step))
    );
  });
}

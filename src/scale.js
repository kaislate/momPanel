// "Make everything bigger" size control. Three steps (normal/big/biggest) drive a
// root font-size + grid column count via a class on <html>, so all rem-based text,
// padding, gauges, and buttons scale together and bigger text gets wider tiles
// instead of cramped ones. The choice persists in config (like clock mode).
import { getConfig, setConfig } from "./api.js";

const STEPS = ["normal", "big", "biggest"];

function clampScale(s) {
  return STEPS.includes(s) ? s : "normal";
}

// Apply the scale by swapping the single scale-* class on <html>.
export function applyScale(scale) {
  const s = clampScale(scale);
  const root = document.documentElement;
  STEPS.forEach((step) => root.classList.remove(`scale-${step}`));
  root.classList.add(`scale-${s}`);
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

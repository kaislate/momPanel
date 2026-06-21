// Volume tile: shows the default audio sink level on an arc gauge with a speaker
// icon (muted variant when muted) and a shortcut to the system Sound settings.
import { arcGauge } from "../gauge.js";
import { openSettings } from "../api.js";

// Speaker SVG. When muted, draw an "x" instead of the sound waves.
function speakerIcon(muted) {
  const waves = muted
    ? `<line x1="16" y1="9" x2="22" y2="15" stroke="currentColor" stroke-width="2" stroke-linecap="round" />
       <line x1="22" y1="9" x2="16" y2="15" stroke="currentColor" stroke-width="2" stroke-linecap="round" />`
    : `<path d="M16 9a4 4 0 0 1 0 6" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" />
       <path d="M18.5 6.5a8 8 0 0 1 0 11" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" />`;
  return `
  <svg viewBox="0 0 24 24" width="22" height="22" role="img" aria-label="${
    muted ? "muted" : "volume"
  }">
    <path d="M3 9v6h4l5 4V5L7 9H3z" fill="currentColor" />
    ${waves}
  </svg>`;
}

export function register(registerTile) {
  registerTile({
    id: "volume",
    title: "Volume",
    intervalMs: 5000,
    render(el, data) {
      if (!data || data.state !== "ok") {
        el.innerHTML =
          `<div class="tile-title">Volume</div>` +
          `<div class="tile--unavailable">Not available</div>`;
        return;
      }
      const { level_percent, muted } = data;
      el.innerHTML =
        `<div class="tile-title">${speakerIcon(muted)} Volume</div>` +
        arcGauge(level_percent, level_percent + "%", muted ? "muted" : "") +
        `<button class="tile-btn" type="button">Sound settings</button>`;
      const btn = el.querySelector(".tile-btn");
      if (btn) btn.addEventListener("click", () => openSettings("sound"));
    },
  });
}

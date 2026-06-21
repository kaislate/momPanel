// Volume tile: level arc (graphic) and an "open settings" button (foot).
import { arcGauge } from "../gauge.js";
import { openSettings } from "../api.js";
import { tile, mutedGraphic } from "../layout.js";

// Speaker SVG; "x" waves when muted. Used for the unavailable placeholder graphic.
function speakerIcon(muted) {
  const waves = muted
    ? `<line x1="16" y1="9" x2="22" y2="15" stroke="currentColor" stroke-width="2" stroke-linecap="round" />
       <line x1="22" y1="9" x2="16" y2="15" stroke="currentColor" stroke-width="2" stroke-linecap="round" />`
    : `<path d="M16 9a4 4 0 0 1 0 6" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" />
       <path d="M18.5 6.5a8 8 0 0 1 0 11" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" />`;
  return `<svg viewBox="0 0 24 24" aria-hidden="true">
    <path d="M3 9v6h4l5 4V5L7 9H3z" fill="currentColor" />${waves}</svg>`;
}

export function register(registerTile) {
  registerTile({
    id: "volume",
    title: "Volume",
    intervalMs: 5000,
    render(el, data) {
      if (!data || data.state !== "ok") {
        el.innerHTML = tile({
          title: "Volume",
          graphic: mutedGraphic(speakerIcon(false)),
          foot: `<div class="tile--unavailable">Sound info isn't available here.</div>`,
        });
        return;
      }
      const { level_percent, muted } = data;
      el.innerHTML = tile({
        title: "Volume",
        graphic: arcGauge(level_percent, level_percent + "%", muted ? "Muted" : ""),
        foot: `<button class="tile-btn" type="button">Open sound settings</button>`,
      });
      el.querySelector(".tile-btn")?.addEventListener("click", () =>
        openSettings("sound")
      );
    },
  });
}

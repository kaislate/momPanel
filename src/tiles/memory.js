// Memory tile: RAM usage as an arc gauge (graphic) with a plain-language line (foot).
import { arcGauge } from "../gauge.js";
import { memoryMessage } from "../copy.js";
import { tile } from "../layout.js";

// RAM stick icon (board with chips on top and pins on the bottom).
function ramIcon() {
  return (
    `<svg class="tile-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" ` +
    `stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">` +
    `<rect x="2" y="7" width="20" height="9" rx="1"/>` +
    `<path d="M6 7v3M10 7v3M14 7v3M18 7v3"/>` +
    `<path d="M5 16v2M8 16v2M11 16v2M14 16v2M17 16v2M20 16v2"/></svg>`
  );
}

export function register(registerTile) {
  registerTile({
    id: "memory",
    title: "Memory",
    intervalMs: 3000,
    render(el, data) {
      if (!data || data.state !== "ok") {
        el.innerHTML = tile({
          title: "Memory",
          foot: `<div class="tile--unavailable">Not available</div>`,
        });
        return;
      }
      const { used_percent, used_mb, total_mb } = data;
      const sub =
        Math.round(used_mb / 1024) + " / " + Math.round(total_mb / 1024) + " GB";
      el.innerHTML = tile({
        title: "Memory",
        graphic:
          `<div class="gauge-row">${ramIcon()}` +
          `<div class="gauge-fixed">${arcGauge(
            used_percent,
            Math.round(used_percent) + "%",
            sub
          )}</div></div>`,
        foot: `<div class="tile-status">${memoryMessage(used_percent)}</div>`,
      });
    },
  });
}

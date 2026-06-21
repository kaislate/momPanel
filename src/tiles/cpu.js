// CPU tile: overall processor usage as an arc gauge (graphic) with a plain-language
// line (foot). Matches the Memory/Storage layout: device icon left, gauge right.
import { arcGauge } from "../gauge.js";
import { cpuMessage } from "../copy.js";
import { tile } from "../layout.js";

// Simple processor-chip icon (square die with pins on all four sides).
function cpuIcon() {
  return (
    `<svg class="svg-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" ` +
    `stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">` +
    `<rect x="7" y="7" width="10" height="10" rx="1.5"/>` +
    `<rect x="10" y="10" width="4" height="4"/>` +
    `<path d="M10 7V4M14 7V4M10 20v-3M14 20v-3M7 10H4M7 14H4M20 10h-3M20 14h-3"/></svg>`
  );
}

export function register(registerTile) {
  registerTile({
    id: "cpu",
    title: "CPU",
    intervalMs: 3000,
    render(el, data) {
      if (!data || data.state !== "ok") {
        el.innerHTML = tile({
          title: "CPU",
          foot: `<div class="tile--unavailable">Not available</div>`,
        });
        return;
      }
      const pct = Math.round(data.used_percent);
      el.innerHTML = tile({
        title: "CPU",
        graphic:
          `<div class="gauge-row">${cpuIcon()}` +
          `<div class="gauge-fixed">${arcGauge(pct, pct + "%", "")}</div></div>`,
        foot: `<div class="tile-status">${cpuMessage(pct)}</div>`,
      });
    },
  });
}

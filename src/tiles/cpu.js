// CPU tile: overall processor usage as an arc gauge (graphic) with a plain-language
// line (foot). Matches the Memory/Storage layout: device icon left, gauge right.
import { arcGauge } from "../gauge.js";
import { cpuMessage } from "../copy.js";
import { tile } from "../layout.js";

// The computer image (background removed) stands in for the CPU/machine.
function cpuIcon() {
  return `<img class="device-icon" src="assets/cpu.png" alt="" />`;
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

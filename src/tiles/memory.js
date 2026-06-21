// Memory tile: RAM usage as an arc gauge (graphic) with a plain-language line (foot).
import { arcGauge } from "../gauge.js";
import { memoryMessage } from "../copy.js";
import { tile } from "../layout.js";

// Two RAM sticks stacked, using the user-supplied RAM icon.
function ramStack() {
  return (
    `<div class="ram-stack">` +
    `<img class="device-icon" src="assets/ram.png" alt="" />` +
    `<img class="device-icon" src="assets/ram.png" alt="" />` +
    `</div>`
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
          `<div class="gauge-row">${ramStack()}` +
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

// Memory tile: shows RAM usage as an arc gauge with used/total in GB.
import { arcGauge } from "../gauge.js";

export function register(registerTile) {
  registerTile({
    id: "memory",
    title: "Memory",
    intervalMs: 3000,
    render(el, data) {
      if (!data || data.state !== "ok") {
        el.innerHTML =
          `<div class="tile-title">Memory</div>` +
          `<div class="tile--unavailable">Not available</div>`;
        return;
      }
      const { used_percent, used_mb, total_mb } = data;
      const sub =
        Math.round(used_mb / 1024) + " / " + Math.round(total_mb / 1024) + " GB";
      el.innerHTML =
        `<div class="tile-title">Memory</div>` +
        arcGauge(used_percent, Math.round(used_percent) + "%", sub);
    },
  });
}

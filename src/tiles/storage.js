// Storage tile: shows primary-disk usage as an arc gauge plus a safe shortcut
// to the system storage settings. Data shape matches mock.js:
//   { state:"ok", used_percent, free_gb, total_gb } | { state:"unavailable" }
import { arcGauge } from "../gauge.js";
import { openSettings } from "../api.js";

export function register(registerTile) {
  registerTile({
    id: "storage",
    title: "Storage",
    intervalMs: 30000,
    render(el, data) {
      if (!data || data.state !== "ok") {
        el.innerHTML = `<div class="tile--unavailable">Not available</div>`;
        return;
      }
      const { used_percent, free_gb, total_gb } = data;
      el.innerHTML =
        arcGauge(used_percent, free_gb + " GB", "free of " + total_gb + " GB") +
        `<button class="tile-btn" type="button">Open storage</button>`;
      const btn = el.querySelector(".tile-btn");
      if (btn) btn.addEventListener("click", () => openSettings("storage"));
    },
  });
}

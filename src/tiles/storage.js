// Storage tile: shows primary-disk usage as an arc gauge plus a safe shortcut
// to the system storage settings. Data shape matches mock.js:
//   { state:"ok", used_percent, free_gb, total_gb } | { state:"unavailable" }
import { arcGauge } from "../gauge.js";
import { openSettings } from "../api.js";
import { storageMessage, SETTINGS_BTN_NOTE } from "../copy.js";

export function register(registerTile) {
  registerTile({
    id: "storage",
    title: "Storage",
    intervalMs: 30000,
    render(el, data) {
      if (!data || data.state !== "ok") {
        el.innerHTML =
          `<div class="tile-title">Storage</div>` +
          `<div class="tile--unavailable">Not available</div>`;
        return;
      }
      const { used_percent, free_gb, total_gb } = data;
      el.innerHTML =
        `<div class="tile-title">Storage</div>` +
        arcGauge(used_percent, free_gb + " GB", "free of " + total_gb + " GB") +
        `<div class="tile-status">${storageMessage(used_percent)}</div>` +
        `<button class="tile-btn" type="button">Open storage settings</button>` +
        `<div class="btn-note">${SETTINGS_BTN_NOTE}</div>`;
      const btn = el.querySelector(".tile-btn");
      if (btn) btn.addEventListener("click", () => openSettings("storage"));
    },
  });
}

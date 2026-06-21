// Storage tile: disk usage ring (graphic), fullness in plain words + an "open
// settings" button (foot). Reports the user's system/home drive (see storage.rs).
import { arcGauge } from "../gauge.js";
import { openSettings } from "../api.js";
import { storageMessage } from "../copy.js";
import { tile } from "../layout.js";

// Hard-drive icon, using the user-supplied HDD image.
function hddIcon() {
  return `<img class="device-icon" src="assets/hdd.png" alt="" />`;
}

export function register(registerTile) {
  registerTile({
    id: "storage",
    title: "Storage",
    intervalMs: 30000,
    render(el, data) {
      if (!data || data.state !== "ok") {
        el.innerHTML = tile({
          title: "Storage",
          foot: `<div class="tile--unavailable">Not available</div>`,
        });
        return;
      }
      const { used_percent, free_gb, total_gb } = data;
      const pct = Math.round(used_percent);
      el.innerHTML = tile({
        title: "Storage",
        graphic:
          `<div class="gauge-row">${hddIcon()}` +
          `<div class="gauge-fixed">${arcGauge(used_percent, pct + "% full", "")}</div></div>`,
        foot:
          `<div class="tile-status">${storageMessage(used_percent)}</div>` +
          `<div class="storage-detail">${free_gb} GB free of ${total_gb} GB</div>` +
          `<button class="tile-btn" type="button">Open storage settings</button>`,
      });
      el.querySelector(".tile-btn")?.addEventListener("click", () =>
        openSettings("storage")
      );
    },
  });
}

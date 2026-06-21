// Storage tile: disk usage ring (graphic), fullness in plain words + an "open
// settings" button (foot). Reports the user's system/home drive (see storage.rs).
import { arcGauge } from "../gauge.js";
import { openSettings } from "../api.js";
import { storageMessage } from "../copy.js";
import { tile } from "../layout.js";

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
      el.innerHTML = tile({
        title: "Storage",
        graphic: arcGauge(used_percent, free_gb + " GB", "free of " + total_gb + " GB"),
        foot:
          `<div class="tile-status">${storageMessage(used_percent)}</div>` +
          `<button class="tile-btn" type="button">Open storage settings</button>`,
      });
      el.querySelector(".tile-btn")?.addEventListener("click", () =>
        openSettings("storage")
      );
    },
  });
}

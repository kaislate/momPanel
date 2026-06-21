// Storage tile: disk usage ring (graphic), fullness in plain words + an "open
// settings" button (foot). Reports the user's system/home drive (see storage.rs).
import { arcGauge } from "../gauge.js";
import { openSettings } from "../api.js";
import { storageMessage } from "../copy.js";
import { tile } from "../layout.js";

// Hard-drive icon (case with a platter and actuator arm).
function hddIcon() {
  return (
    `<svg class="tile-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" ` +
    `stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">` +
    `<rect x="3" y="6" width="18" height="12" rx="2"/>` +
    `<circle cx="9" cy="12" r="3.4"/>` +
    `<circle cx="9" cy="12" r="0.6" fill="currentColor"/>` +
    `<path d="M11.5 14.5 16 18"/>` +
    `<circle cx="17" cy="15" r="0.8" fill="currentColor"/></svg>`
  );
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
      el.innerHTML = tile({
        title: "Storage",
        graphic:
          `<div class="gauge-row">${hddIcon()}` +
          `<div class="gauge-fixed">${arcGauge(
            used_percent,
            free_gb + " GB",
            "free of " + total_gb + " GB"
          )}</div></div>`,
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

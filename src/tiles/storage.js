// Storage tile: a fullness ring (graphic) with a two-line center — the percentage on
// top and "Full"/"Free" below. Tap the ring to switch between showing percent FULL and
// percent FREE; the detail line under the gauge follows the mode (used-of-total vs
// free-of-total). Reports the user's system/home drive (see storage.rs).
import { arcGauge } from "../gauge.js";
import { openSettings } from "../api.js";
import { storageMessage } from "../copy.js";
import { tile } from "../layout.js";

// Display mode persists for the session: "full" (percent used) or "free" (percent free).
let mode = "full";

function hddIcon() {
  return `<img class="device-icon" src="assets/hdd.png" alt="" />`;
}

function draw(el, data) {
  if (!data || data.state !== "ok") {
    el.innerHTML = tile({
      title: "Storage",
      foot: `<div class="tile--unavailable">Not available</div>`,
    });
    return;
  }

  const { used_percent, free_gb, total_gb } = data;
  const usedPct = Math.round(used_percent);
  const freePct = Math.max(0, 100 - usedPct);
  const usedGb = Math.max(0, total_gb - free_gb);
  const isFull = mode === "full";

  const displayPct = isFull ? usedPct : freePct;
  const modeLabel = isFull ? "Full" : "Free";
  const detail = isFull
    ? `${usedGb} GB used of ${total_gb} GB`
    : `${free_gb} GB free of ${total_gb} GB`;

  el.innerHTML = tile({
    title: "Storage",
    graphic:
      `<div class="gauge-row">${hddIcon()}` +
      `<div class="gauge-fixed storage-gauge" role="button" tabindex="0" ` +
      `title="Tap to switch between full and free">` +
      // ring fills by the displayed percent, but is always colored by how FULL it is
      `${arcGauge(displayPct, displayPct + "%", modeLabel, usedPct)}</div></div>`,
    foot:
      `<div class="storage-detail">${detail}</div>` +
      `<div class="tile-status">${storageMessage(used_percent)}</div>` +
      `<button class="tile-btn" type="button">Open storage settings</button>`,
  });

  el.querySelector(".tile-btn")?.addEventListener("click", () =>
    openSettings("storage")
  );

  const gauge = el.querySelector(".storage-gauge");
  const toggle = () => {
    mode = isFull ? "free" : "full";
    draw(el, data); // re-render immediately in the new mode
    el.querySelector(".storage-gauge")?.focus(); // re-render drops focus; restore it
  };
  gauge?.addEventListener("click", toggle);
  gauge?.addEventListener("keydown", (e) => {
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      toggle();
    }
  });
}

export function register(registerTile) {
  registerTile({ id: "storage", title: "Storage", intervalMs: 30000, render: draw });
}

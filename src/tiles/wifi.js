// Wi-Fi tile: signal arcs (graphic), network name + an "open settings" button (foot).
import { openSettings } from "../api.js";
import { escapeHtml } from "../escape.js";
import { tile, mutedGraphic } from "../layout.js";

// Nested arcs filled proportionally to signal strength — one gauge across rings.
function signalArcs(percent) {
  const p = Math.max(0, Math.min(100, percent));
  const color = p >= 67 ? "#5bd6a0" : p >= 34 ? "#ffb347" : "#ff5d5d";
  const radii = [18, 27, 36, 45];
  const rings = radii
    .map((r) => {
      const circ = 2 * Math.PI * r;
      const dash = (p / 100) * circ;
      return (
        `<circle cx="50" cy="50" r="${r}" fill="none" stroke="#2a3146" stroke-width="6" />` +
        `<circle cx="50" cy="50" r="${r}" fill="none" stroke="${color}" stroke-width="6" ` +
        `stroke-linecap="round" stroke-dasharray="${dash} ${circ}" ` +
        `transform="rotate(-90 50 50)" />`
      );
    })
    .join("");
  return (
    `<svg class="gauge" viewBox="0 0 100 100" role="img" aria-label="Signal ${p}%">` +
    rings +
    `<text x="50" y="54" text-anchor="middle" class="gauge-label">${p}%</text>` +
    `</svg>`
  );
}

function wifiIcon() {
  return (
    `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" ` +
    `stroke-linecap="round" aria-hidden="true">` +
    `<path d="M2 8.5a16 16 0 0 1 20 0"/><path d="M5 12a11 11 0 0 1 14 0"/>` +
    `<path d="M8.5 15.5a6 6 0 0 1 7 0"/><circle cx="12" cy="19" r="1"/></svg>`
  );
}

export function register(registerTile) {
  registerTile({
    id: "wifi",
    title: "Wi-Fi",
    intervalMs: 20000,
    render(el, data) {
      if (!data || data.state !== "ok") {
        el.innerHTML = tile({
          title: "Wi-Fi",
          graphic: mutedGraphic(wifiIcon()),
          foot: `<div class="tile--unavailable">No Wi-Fi found — you may be on a cable, that's okay.</div>`,
        });
        return;
      }
      const ssid = String(data.ssid ?? "");
      const signal = Number(data.signal_percent ?? 0);
      el.innerHTML = tile({
        title: "Wi-Fi",
        graphic: signalArcs(signal),
        foot:
          `<div class="tile-status">${escapeHtml(ssid)}</div>` +
          `<button class="tile-btn" type="button" data-wifi-settings>Open Wi-Fi settings</button>`,
      });
      el.querySelector("[data-wifi-settings]")?.addEventListener("click", () =>
        openSettings("wifi")
      );
    },
  });
}

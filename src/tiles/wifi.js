// Wi-Fi tile: shows the active connection's SSID and signal strength as a set of
// concentric arcs, plus a shortcut to the system Wi-Fi settings.
// Data shape (matches mock.js): { state:"ok", ssid:string, signal_percent:number }
//                            or { state:"unavailable" }
import { openSettings } from "../api.js";
import { escapeHtml } from "../escape.js";
import { SETTINGS_BTN_NOTE } from "../copy.js";

// Build an SVG with 4 nested arcs. Each ring fills proportionally to the overall
// signal: the whole stack acts like one gauge spread across concentric rings, so
// a weak signal lights only the inner rings faintly and a strong one fills them all.
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

export function register(registerTile) {
  registerTile({
    id: "wifi",
    title: "Wi-Fi",
    intervalMs: 20000,
    render(el, data) {
      if (!data || data.state !== "ok") {
        el.innerHTML =
          `<div class="tile-title">Wi-Fi</div>` +
          `<div class="tile-sub tile--unavailable">No Wi-Fi found — you may be on a cable, that's okay.</div>`;
        return;
      }
      const ssid = String(data.ssid ?? "");
      const signal = Number(data.signal_percent ?? 0);
      el.innerHTML =
        `<div class="tile-title">Wi-Fi</div>` +
        signalArcs(signal) +
        `<div class="tile-big">${escapeHtml(ssid)}</div>` +
        `<button class="tile-btn" type="button" data-wifi-settings>Open Wi-Fi settings</button>` +
        `<div class="btn-note">${SETTINGS_BTN_NOTE}</div>`;
      const btn = el.querySelector("[data-wifi-settings]");
      if (btn) btn.addEventListener("click", () => openSettings("wifi"));
    },
  });
}

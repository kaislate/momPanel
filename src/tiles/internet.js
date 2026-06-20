// Internet tile: shows whether the machine has working internet access.
// Data shape (matches src/mock.js): { state: "ok", online: bool } or { state: "unavailable" }.
import { getTile } from "../api.js";

function globeSvg(color) {
  return (
    `<svg viewBox="0 0 24 24" width="1em" height="1em" ` +
    `style="vertical-align:-0.125em;display:inline-block" ` +
    `fill="none" stroke="${color}" stroke-width="1.6" ` +
    `stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">` +
    `<circle cx="12" cy="12" r="9"/>` +
    `<path d="M3 12h18"/>` +
    `<path d="M12 3a14 14 0 0 1 0 18a14 14 0 0 1 0-18z"/>` +
    `</svg>`
  );
}

export function register(registerTile) {
  registerTile({
    id: "internet",
    title: "Internet",
    intervalMs: 20000,
    // Default fetch (getTile("internet")) made explicit for clarity.
    fetch: () => getTile("internet"),
    render(el, data) {
      if (!data || data.state !== "ok") {
        el.innerHTML =
          `<div class="tile-title">Internet</div>` +
          `<div class="tile--unavailable">Not available</div>`;
        return;
      }

      const online = data.online === true;
      const color = online ? "#2ecc71" : "#e74c3c";
      const label = online ? "Online" : "Offline";

      el.innerHTML =
        `<div class="tile-title">Internet</div>` +
        `<div class="tile-big" style="color:${color}">` +
        `${globeSvg(color)} ${label}` +
        `</div>`;
    },
  });
}

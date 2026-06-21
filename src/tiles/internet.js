// Internet tile: a globe + Online/Offline (graphic) and a plain-language line (foot).
import { getTile } from "../api.js";
import { internetMessage } from "../copy.js";
import { tile, mutedGraphic } from "../layout.js";

function globeSvg(color, size) {
  return (
    `<svg viewBox="0 0 24 24" width="${size}" height="${size}" ` +
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
    fetch: () => getTile("internet"),
    render(el, data) {
      if (!data || data.state !== "ok") {
        el.innerHTML = tile({
          title: "Internet",
          graphic: mutedGraphic(globeSvg("currentColor", "3rem")),
          foot: `<div class="tile--unavailable">Not available</div>`,
        });
        return;
      }
      const online = data.online === true;
      const color = online ? "#2ecc71" : "#e74c3c";
      const label = online ? "Online" : "Offline";
      el.innerHTML = tile({
        title: "Internet",
        graphic: `<div class="tile-big" style="color:${color}">${globeSvg(
          color,
          "1em"
        )} ${label}</div>`,
        foot: `<div class="tile-status">${internetMessage(online)}</div>`,
      });
    },
  });
}

// Printers tile: the printer photo as the graphic, with each printer's name + status
// (and a colored dot) listed in the foot, plus an "open settings" button. The default
// printer is starred.
import { openSettings } from "../api.js";
import { printerStatusWord } from "../copy.js";
import { escapeHtml } from "../escape.js";
import { tile, mutedGraphic } from "../layout.js";

// Theme-aware status colors so the user's palette (incl. high-contrast) applies.
function dotColor(status) {
  switch (status) {
    case "ready":
      return "var(--ok)";
    case "offline":
      return "var(--bad)";
    case "out_of_paper":
      return "var(--warn)";
    default:
      return "var(--ink-dim)";
  }
}

function printerPhoto() {
  return `<img class="printer-photo" src="assets/printer.png" alt="" />`;
}

function statusLine(p, isDefault) {
  const word = printerStatusWord(p.status);
  const star = isDefault ? ' <span aria-hidden="true">★</span>' : "";
  return (
    `<div class="printer-line">` +
    `<span class="printer-dot" style="background:${dotColor(p.status)}"></span>` +
    `<span class="printer-name">${escapeHtml(p.name)}</span>` +
    `<span class="printer-status">${word}</span>${star}</div>`
  );
}

export function register(registerTile) {
  registerTile({
    id: "printers",
    title: "Printers",
    intervalMs: 30000,
    render(el, data) {
      if (!data || data.state === "unavailable") {
        el.innerHTML = tile({
          title: "Printers",
          graphic: mutedGraphic(printerPhoto()),
          foot: `<div class="tile--unavailable">Printer info isn't available here.</div>`,
        });
        return;
      }

      const printers = Array.isArray(data.printers) ? data.printers : [];
      const def = data.default_name ?? null;

      const graphic = printers.length === 0 ? mutedGraphic(printerPhoto()) : printerPhoto();
      const lines =
        printers.length === 0
          ? `<div class="tile-sub">No printers connected</div>`
          : printers.map((p) => statusLine(p, def != null && p.name === def)).join("");

      el.innerHTML = tile({
        title: "Printers",
        graphic,
        foot:
          lines +
          `<button class="tile-btn" type="button" data-printers-settings>Open printer settings</button>`,
      });
      el.querySelector("[data-printers-settings]")?.addEventListener("click", () =>
        openSettings("printers")
      );
    },
  });
}

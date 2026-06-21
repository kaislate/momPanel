// Printers tile: one chip per printer with a colored status dot + friendly word
// (graphic), and an "open settings" button (foot). The default printer is starred.
import { openSettings } from "../api.js";
import { printerStatusWord } from "../copy.js";
import { escapeHtml } from "../escape.js";
import { tile, mutedGraphic } from "../layout.js";

function dotColor(status) {
  switch (status) {
    case "ready":
      return "#34c759";
    case "offline":
      return "#ff3b30";
    case "out_of_paper":
      return "#ff9f0a";
    default:
      return "#8e8e93";
  }
}

function printerIcon() {
  return (
    `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" ` +
    `stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">` +
    `<path d="M6 9V3h12v6"/><rect x="4" y="9" width="16" height="8" rx="2"/>` +
    `<path d="M7 17h10v4H7z"/></svg>`
  );
}

function chip(p, isDefault) {
  const color = dotColor(p.status);
  const word = printerStatusWord(p.status);
  const star = isDefault ? '<span aria-hidden="true">★</span>' : "";
  const cls = isDefault ? "printer-chip printer-chip--default" : "printer-chip";
  const dot = `<span class="printer-dot" aria-hidden="true" style="background:${color}"></span>`;
  return (
    `<span class="${cls}" title="${escapeHtml(p.name)} — ${word}">` +
    `${dot}${star}<span class="printer-name">${escapeHtml(p.name)}</span>` +
    `<span class="printer-status">${word}</span></span>`
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
          graphic: mutedGraphic(printerIcon()),
          foot: `<div class="tile--unavailable">Printer info isn't available here.</div>`,
        });
        return;
      }

      const printers = Array.isArray(data.printers) ? data.printers : [];
      const def = data.default_name ?? null;

      const graphic =
        printers.length === 0
          ? mutedGraphic(printerIcon())
          : `<div class="printer-chips">${printers
              .map((p) => chip(p, def != null && p.name === def))
              .join("")}</div>`;

      const foot =
        (printers.length === 0
          ? `<div class="tile-sub">No printers connected</div>`
          : "") +
        `<button class="tile-btn" type="button" data-printers-settings>Open printer settings</button>`;

      el.innerHTML = tile({ title: "Printers", graphic, foot });
      el.querySelector("[data-printers-settings]")?.addEventListener("click", () =>
        openSettings("printers")
      );
    },
  });
}

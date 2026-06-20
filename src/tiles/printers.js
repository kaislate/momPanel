// Printers tile: one chip per CUPS printer with a colored status dot. The default
// printer is highlighted. Empty list -> friendly "No printers connected". A
// "Printer settings" button opens the native printers settings screen.
//
// Data shape (matches src/mock.js):
//   { state: "ok", default_name: "Office_LaserJet" | null,
//     printers: [{ name, status }] }      // status: ready | offline | out_of_paper | unknown
//   { state: "unavailable" }
import { openSettings } from "../api.js";

// Map a status string to a dot color. Unknown statuses fall back to grey.
function dotColor(status) {
  switch (status) {
    case "ready":
      return "#34c759"; // green
    case "offline":
      return "#ff3b30"; // red
    case "out_of_paper":
      return "#ff9f0a"; // amber
    default:
      return "#8e8e93"; // grey
  }
}

function escapeHtml(s) {
  return String(s).replace(
    /[&<>"']/g,
    (c) =>
      ({
        "&": "&amp;",
        "<": "&lt;",
        ">": "&gt;",
        '"': "&quot;",
        "'": "&#39;",
      }[c])
  );
}

function chip(p, isDefault) {
  const color = dotColor(p.status);
  const star = isDefault ? '<span aria-hidden="true">★</span>' : "";
  const cls = isDefault ? "printer-chip printer-chip--default" : "printer-chip";
  const dot =
    `<span class="printer-dot" aria-hidden="true" ` +
    `style="background:${color}"></span>`;
  const label = `${escapeHtml(p.name)} (${escapeHtml(p.status)})`;
  return (
    `<span class="${cls}" title="${label}">` +
    `${dot}${star}<span class="printer-name">${escapeHtml(p.name)}</span></span>`
  );
}

export function register(registerTile) {
  registerTile({
    id: "printers",
    title: "Printers",
    intervalMs: 30000,
    render(el, data) {
      if (!data || data.state === "unavailable") {
        el.classList.add("tile--unavailable");
        el.innerHTML =
          `<div class="tile-title">Printers</div>` +
          `<div class="tile-big">Not available</div>`;
        return;
      }
      el.classList.remove("tile--unavailable");

      const printers = Array.isArray(data.printers) ? data.printers : [];
      const def = data.default_name ?? null;

      let body;
      if (printers.length === 0) {
        body = `<div class="tile-sub">No printers connected</div>`;
      } else {
        const chips = printers
          .map((p) => chip(p, def != null && p.name === def))
          .join("");
        body = `<div class="printer-chips">${chips}</div>`;
      }

      el.innerHTML =
        `<div class="tile-title">Printers</div>` +
        body +
        `<button class="tile-btn" type="button" data-printers-settings>` +
        `Printer settings</button>`;

      el.querySelector("[data-printers-settings]")?.addEventListener(
        "click",
        () => openSettings("printers")
      );
    },
  });
}

// Integration barrel: the single place that wires every tile module into the panel.
// Each tile module exports `register(registerTile)`; this file imports and calls them.
//
// FOUNDATION STATE: until the real tile modules land, this registers placeholder
// tiles driven by mock data so the panel runs standalone. The integration step
// replaces the placeholder loop below with real imports + register() calls.
import { mockTile } from "../mock.js";

const PLACEHOLDER_IDS = [
  "clock",
  "date",
  "memory",
  "storage",
  "wifi",
  "internet",
  "printers",
  "volume",
  "weather",
];

export async function registerAll(registerTile) {
  for (const id of PLACEHOLDER_IDS) {
    registerTile({
      id,
      title: id,
      intervalMs: 0,
      fetch: () => mockTile(id),
      render: (el, d) => {
        el.innerHTML =
          `<div class="tile-title">${id}</div>` +
          `<pre class="tile-sub" style="white-space:pre-wrap">${JSON.stringify(
            d
          )}</pre>`;
      },
    });
  }
}

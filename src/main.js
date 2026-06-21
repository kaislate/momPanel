import {
  registerTile,
  mountTiles,
  startRenderLoop,
  refreshTile,
} from "./tiles.js";
import { registerAll } from "./tiles/index.js";
import { listen } from "./bridge.js";
import { initChrome, mountControls } from "./scale.js";
import { showHelp } from "./help.js";

async function boot() {
  // Apply saved size + hide-controls state before tiles render (avoids a flash).
  const chrome = await initChrome();

  await registerAll(registerTile);
  mountTiles();
  mountControls(chrome);
  startRenderLoop();

  // Delegated: a tile's "?" dot opens its plain-language explanation.
  document.addEventListener("click", (e) => {
    const dot = e.target.closest(".tile-help");
    if (dot) showHelp(dot.dataset.help);
  });

  // Instant Wi-Fi/internet refresh when the backend pushes a network change
  // (NetworkManager D-Bus signal). Polling remains the fallback.
  await listen("net-changed", () => {
    refreshTile("wifi");
    refreshTile("internet");
  });
}

window.addEventListener("DOMContentLoaded", boot);

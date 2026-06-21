import {
  registerTile,
  mountTiles,
  startRenderLoop,
  refreshTile,
} from "./tiles.js";
import { registerAll } from "./tiles/index.js";
import { listen } from "./bridge.js";
import { initScale, mountScaleControl } from "./scale.js";

async function boot() {
  // Apply the saved text size before tiles render (avoids a flash of the wrong size).
  const scale = await initScale();

  await registerAll(registerTile);
  mountTiles();
  mountScaleControl(scale);
  startRenderLoop();

  // Instant Wi-Fi/internet refresh when the backend pushes a network change
  // (NetworkManager D-Bus signal). Polling remains the fallback.
  await listen("net-changed", () => {
    refreshTile("wifi");
    refreshTile("internet");
  });
}

window.addEventListener("DOMContentLoaded", boot);

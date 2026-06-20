import {
  registerTile,
  mountTiles,
  startRenderLoop,
  refreshTile,
} from "./tiles.js";
import { registerAll } from "./tiles/index.js";
import { listen } from "./bridge.js";

async function boot() {
  await registerAll(registerTile);
  mountTiles();
  startRenderLoop();

  // Instant Wi-Fi/internet refresh when the backend pushes a network change
  // (NetworkManager D-Bus signal). Polling remains the fallback.
  await listen("net-changed", () => {
    refreshTile("wifi");
    refreshTile("internet");
  });
}

window.addEventListener("DOMContentLoaded", boot);

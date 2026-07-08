import { applyTheme } from "./theme.js";
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
import { appVersion, getConfig, setConfig } from "./api.js";
import { showWhatsNew } from "./whatsnew.js";

// Show the "what's new" note once after the app updates to a new version. Skipped on
// a fresh install (no last-seen version yet) — only real updates get the popup.
async function checkWhatsNew() {
  const [version, cfg] = await Promise.all([appVersion(), getConfig()]);
  if (!version) return;
  const updated = cfg.last_seen_version && cfg.last_seen_version !== version;
  if (cfg.last_seen_version !== version) {
    setConfig({ last_seen_version: version }); // record it either way
  }
  if (updated) showWhatsNew(version);
}

async function boot() {
  const cfg = await getConfig();
  // Apply saved colors before anything renders (prevents a flash of the default theme).
  applyTheme(cfg.theme);

  // Apply saved size + hide-controls state before tiles render (avoids a flash).
  const chrome = await initChrome();

  // Experimental Companion mode replaces the tile grid entirely (status by
  // exception: big time/weather, one health card). Toggled in the About panel;
  // switching reloads the window, so exactly one mode ever boots.
  if (cfg.experimental_ui) {
    const { initCompanion } = await import("./preview/companion.js");
    await initCompanion();
    mountControls(chrome);
    checkWhatsNew();
    return;
  }

  await registerAll(registerTile);
  mountTiles();
  mountControls(chrome);
  startRenderLoop();

  checkWhatsNew();

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

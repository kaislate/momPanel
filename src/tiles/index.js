// Integration barrel: the single place that wires every tile module into the panel.
// Each tile module exports `register(registerTile)`; this file imports and calls them.
//
// The clock module registers BOTH the "clock" and "date" tiles.
import { register as registerClock } from "./clock.js";
import { register as registerMemory } from "./memory.js";
import { register as registerStorage } from "./storage.js";
import { register as registerWifi } from "./wifi.js";
import { register as registerInternet } from "./internet.js";
import { register as registerPrinters } from "./printers.js";
import { register as registerVolume } from "./volume.js";
import { register as registerWeather } from "./weather.js";

export async function registerAll(registerTile) {
  await registerClock(registerTile);
  registerMemory(registerTile);
  registerStorage(registerTile);
  registerWifi(registerTile);
  registerInternet(registerTile);
  registerPrinters(registerTile);
  registerVolume(registerTile);
  registerWeather(registerTile);
}

// Backend API surface. Every call returns a uniform shape and never throws to the
// caller: on any failure a tile gets `{ state: "unavailable" }` and shows a calm
// "not available" state instead of breaking the panel.
import { invoke } from "./bridge.js";

const UNAVAILABLE = { state: "unavailable" };

// Read a simple (no-argument) tile collector by name.
export async function getTile(name) {
  try {
    return await invoke("read_tile", { name });
  } catch {
    return UNAVAILABLE;
  }
}

// Weather needs the stored ZIP, so it has its own command.
export async function readWeather(zip) {
  try {
    return await invoke("read_weather", { zip });
  } catch {
    return UNAVAILABLE;
  }
}

export async function getConfig() {
  try {
    return await invoke("get_config");
  } catch {
    return {
      zip: null,
      clock_mode: "digital",
      ui_scale: "normal",
      hide_controls: false,
      auto_update: true,
      hide_help: true,
      last_seen_version: "",
    };
  }
}

export async function setConfig(cfg) {
  try {
    return await invoke("set_config", { cfg });
  } catch {
    return null;
  }
}

// Safe shortcut: opens a native settings screen. Never performs destructive actions.
export async function openSettings(target) {
  try {
    return await invoke("open_settings", { target });
  } catch {
    return null;
  }
}

export async function appVersion() {
  try {
    return await invoke("app_version");
  } catch {
    return "";
  }
}

export async function osInfo() {
  try {
    return await invoke("os_info");
  } catch {
    return "";
  }
}

export async function openGithub() {
  try {
    return await invoke("open_github");
  } catch {
    return null;
  }
}

export async function checkUpdates() {
  try {
    return await invoke("check_updates");
  } catch {
    return "Couldn't check for updates";
  }
}

export async function getAutostart() {
  try {
    return await invoke("get_autostart");
  } catch {
    return false;
  }
}

export async function setAutostart(enabled) {
  try {
    return await invoke("set_autostart", { enabled });
  } catch {
    return null;
  }
}

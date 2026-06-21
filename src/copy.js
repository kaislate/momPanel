// Plain-language "comprehension layer": one place that turns raw numbers/states into
// calm, jargon-free words for a non-technical user. Reused across tiles so wording
// stays consistent (and is easy to reword once).

// Storage fullness, keyed on percent used. Free GB shown separately by the tile.
export function storageMessage(usedPercent) {
  if (usedPercent < 70) return "Plenty of room";
  if (usedPercent < 90) return "Getting full";
  return "Almost full";
}

// Memory pressure, keyed on percent used.
export function memoryMessage(usedPercent) {
  if (usedPercent < 70) return "Running smoothly";
  if (usedPercent < 90) return "Working hard";
  return "Very busy";
}

// Friendly internet line. `online` is a boolean.
export function internetMessage(online) {
  return online
    ? "You're connected to the internet"
    : "No internet right now — this is usually temporary";
}

// Friendly, readable printer status word (replaces raw strings like "out_of_paper").
export function printerStatusWord(status) {
  switch (status) {
    case "ready":
      return "Ready";
    case "offline":
      return "Offline";
    case "out_of_paper":
      return "Out of paper";
    default:
      return "Unknown";
  }
}

// Friendly weather word for a WMO condition bucket (matches the icon).
export function weatherWord(cond) {
  switch (cond) {
    case "clear":
      return "Sunny";
    case "fog":
      return "Foggy";
    case "rain":
      return "Rainy";
    case "snow":
      return "Snowy";
    case "thunder":
      return "Stormy";
    case "cloudy":
    default:
      return "Cloudy";
  }
}

// Calm caption shown under every "open settings" button so the user knows exactly
// what will happen and that nothing on the panel changes.
export const SETTINGS_BTN_NOTE = "opens a new window — nothing here changes";

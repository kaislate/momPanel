// Plain-English, kind "what's new" notes shown inside the app after an update, keyed
// by version. (The GitHub release notes in CHANGELOG.md can be more technical.)
export const CHANGELOG = {
  "0.4.3": {
    changes: [
      "🌦️ Weather is more dependable now. If the main weather service is ever unreachable, momPanel automatically falls back to the US National Weather Service — so your forecast keeps showing instead of going blank.",
    ],
  },
  "0.4.2": {
    changes: [
      "🎨 momPanel finally wears its own face! The app icon is now the momPanel logo (Grandma with her little panel) instead of the generic placeholder — so it looks right in the taskbar and file manager.",
      "🔽 Fixed the settings drop-downs — they were showing up white-on-white; now they're dark and easy to read.",
    ],
  },
  "0.4.1": {
    changes: [
      "🔧 Fixed the About window: it's now wider and scrolls, so you can reach every setting — including the new Theme colors that were hiding below the bottom.",
      "✅ The on/off checkboxes line up neatly with their labels again.",
    ],
  },
  "0.4.0": {
    changes: [
      "🚨 momPanel now warns you *before* your computer runs low on memory — with an alert sound and a spoken heads-up, even when momPanel is minimized, so a runaway app can't quietly freeze your PC.",
      "💡 The warning names the app using the most memory, so you know exactly what to close. If you don't act, it keeps reminding you and finally pops up a dialog.",
      "🎨 New — make momPanel your own! Pick a ready-made theme (Midnight, Warm, High-contrast) or choose your own colors for the accent, background, tiles, and the usage gauges, all in the About window.",
      "🎚️ You can fine-tune the memory warning too: when it appears (70–90% full), its sound, its volume, and its color.",
    ],
  },
  "0.3.5": {
    changes: [
      "🔊 The sound level now works on Windows, not just Linux.",
      "📶 Wi-Fi now shows your network name on Windows. (Tip: turn on Location in Windows settings to also see the signal strength.)",
    ],
  },
  "0.3.4": {
    changes: [
      "📶 Wi-Fi and 🖨️ printers now work on Windows too, not just Linux.",
      "💻 The About window now shows which system you're running on.",
    ],
  },
  "0.3.3": {
    changes: [
      "✨ momPanel now shows a friendly note like this whenever it updates, so you always know what changed.",
      "📖 You can re-read the latest update note anytime — there's a new “What's New” button in the About window.",
      "🌦️ The weather now shows a 7-day forecast, with clearer day descriptions that fit properly.",
      "🔌 You can choose whether momPanel starts automatically when you log in (in the About window).",
    ],
  },
};

// Returns the entry for a version, or null if we don't have notes for it.
export function changesFor(version) {
  return CHANGELOG[version] || null;
}

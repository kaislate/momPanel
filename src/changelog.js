// Plain-English, kind "what's new" notes shown inside the app after an update, keyed
// by version. (The GitHub release notes in CHANGELOG.md can be more technical.)
export const CHANGELOG = {
  "0.4.0": {
    changes: [
      "🚨 momPanel now warns you *before* your computer runs low on memory. A banner pops up over everything — even when momPanel is minimized — so a runaway app can't quietly freeze your PC.",
      "💡 The warning tells you which app is using the most memory, so you know exactly what to close.",
      "🎚️ In the About window you can choose when the warning appears (from 70% to 90% full) and pick the banner's color.",
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

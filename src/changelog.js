// Plain-English, kind "what's new" notes shown inside the app after an update, keyed
// by version. (The GitHub release notes in CHANGELOG.md can be more technical.)
export const CHANGELOG = {
  "0.6.5": {
    changes: [
      "🖼️ On Linux, Companion mode's see-through background shows your actual wallpaper again — “Invisible” was coming up plain white because the app was being handed the wrong wallpaper (one that didn't exist) by the same library mix-up that broke the buttons. Also fixed for good measure: the clear setting now refuses to show a blank page even if the wallpaper can't be found.",
      "🧰 The same library mix-up was quietly at risk of breaking sounds, alerts, and status readings on Linux — all of those now launch with a clean slate too.",
    ],
  },
  "0.6.4": {
    changes: [
      "🔘 On Linux, the buttons (open printer settings, open storage, and friends) truly work now. We caught the culprit red-handed on the real machine: the app was accidentally lending its own bundled libraries to the settings programs it opened, which made them crash before showing anything. They now launch with a clean slate.",
      "👻 Fixed the see-through ghosts on Linux — closed windows and notification animations no longer leave trails behind. The panel background is drawn from your wallpaper instead of true see-through there (a Linux graphics bug upstream); Windows and Mac keep the real thing.",
    ],
  },
  "0.6.3": {
    changes: [
      "📐 Tiles finally have elbow room: the window is a little taller, the gauges size themselves to fit, and nothing squishes or overlaps anymore — the Storage tile especially is back to looking tidy.",
      "🗂️ The About window behaves in small windows too: settings columns rearrange themselves instead of crushing together, and drop-downs stay inside their column.",
      "🫥 See-through Companion mode is truly see-through now — the faint leftover tint is gone on both Linux and Windows, and “Very clear” really is very clear.",
      "🪟 On Windows, the odd old-fashioned title bar that peeked through a see-through panel (or flashed while moving it) is gone for good.",
      "📌 Fixed a sneaky bug where minimizing momPanel could make it reopen invisible — parked off the edge of the screen — the next time it started.",
      "🛡️ New in Companion mode: turn on a solid panel behind the clock &amp; weather, or behind the “All is well” card, so a busy wallpaper can't make them hard to read. Both live in the About window.",
      "⚙️ In Companion mode the little settings gear now sits right under the “All is well” card instead of floating off in the corner by itself.",
      "📏 New option: make the “All is well” side the same height as the clock-and-weather side, for a tidier, more even look (About window).",
      "📜 Popups like this one now scroll with a slim rounded bar instead of a boxy scrollbar with arrows.",
      "🕰️ Curious what changed before? This popup now has “Older updates” and “Newer updates” links at the bottom, so you can flip through every past update note.",
    ],
  },
  "0.6.2": {
    changes: [
      "🫥 Companion mode's see-through background now works properly on Linux — “Invisible” really is invisible, showing whatever is behind the panel instead of a black box.",
      "🔴 Fixed the stuck red notification dot: memory warnings now update one single notification instead of piling up, and it clears itself when memory recovers or you dismiss it.",
      "🖥️ Rebuilt how the panel draws on Linux (a workaround for 2023-era graphics bugs was itself causing today's glitches) — this may also cure unresponsive buttons; if any button still does nothing, momPanel now keeps a small log so we can find out exactly why.",
    ],
  },
  "0.6.1": {
    changes: [
      "🖱️ Fixed: on Linux, the buttons (open printer settings, open storage, and friends) work again — a see-through window feature was quietly breaking clicks there.",
      "🖼️ Companion mode's see-through background now shows your actual desktop on Linux — cleanly, with no ghost images — and there's a new “Invisible” setting that shows just the desktop behind your panel.",
      "🔍 New in Companion mode: rest your mouse on any line in the “All is well” card (Wi-Fi, Printer, Sound…) to peek at its full tile — gauges, details, and buttons included.",
      "🩸 The Printers tile now shows your ink levels as little colored bars, and warns you when one is running low — Companion mode mentions it too.",
    ],
  },
  "0.6.0": {
    changes: [
      "🍎 momPanel now runs on Macs too — alongside Linux and Windows. The settings buttons open the right screens on every system.",
      "🧹 A tidier corner: the little control buttons now tuck away behind one faint ⚙ dot. Tap it when you need them; they stay out of your way when you don't.",
      "🗂️ The About window is all on one page now — everything visible at a glance, nothing hiding below the bottom.",
      "🌙 Companion mode fits in a smaller, cozier window — and greets you properly in the wee hours.",
      "🫧 New in Companion mode: make the background see-through! Pick how clear in the About window, from solid to barely-there.",
      "🌊 Notes now glide in and out at the bottom of Companion mode instead of popping — everything above slides gently to make room.",
    ],
  },
  "0.5.1": {
    changes: [
      "🖨️ The Printers tile now tells the truth: it says “Offline” when the printer is switched off, instead of always claiming “Ready.”",
    ],
  },
  "0.5.0": {
    changes: [
      "🧪 New to try: Companion mode! A calmer momPanel — a big clock, the weather, and one “All is well” card that only speaks up when something needs attention. Turn it on in the About window, under Preview.",
      "📍 momPanel now remembers where you put the window.",
      "🔇 When the sound is muted, the Volume tile now says “Sound is off” loud and clear.",
      "🌦️ Weather is sturdier: an internet hiccup keeps your last forecast on screen, and setting your location is one friendly button.",
      "🎨 Your theme colors now reach everything — the internet globe, Wi-Fi arcs, and printer dots (including High contrast).",
      "🖨️ Printers and Wi-Fi now work in any language, and momPanel won't accidentally open twice.",
    ],
  },
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

// Every version with notes, newest first (the CHANGELOG object's insertion order).
export function versionsNewestFirst() {
  return Object.keys(CHANGELOG);
}

// The entries to step to from `version` in the What's New history: `older` is the
// next entry further back, `newer` the next one forward; null at either end. An
// unknown version (e.g. a dev build with no notes yet) still offers the newest
// recorded entry as `older`, so the history stays reachable.
export function neighbors(version) {
  const list = versionsNewestFirst();
  const i = list.indexOf(version);
  if (i === -1) return { older: list[0] ?? null, newer: null };
  return { older: list[i + 1] ?? null, newer: i > 0 ? list[i - 1] : null };
}

// One structure for every tile so the whole panel reads consistently:
//   [ Title ]      — top, always
//   [ Graphic ]    — middle, the main visual (gauge / icon / big value); fills space
//   [ Foot ]       — bottom, optional supporting text and/or a button (case by case)
//
// Tiles build their `graphic` and `foot` HTML and pass them here; they never lay out
// the title/structure themselves, so all tiles stay visually aligned.
export function tile({ title, graphic = "", foot = "" }) {
  return (
    `<div class="tile-title">${title}</div>` +
    `<div class="tile-graphic">${graphic}</div>` +
    (foot ? `<div class="tile-foot">${foot}</div>` : "")
  );
}

// A muted placeholder graphic for "unavailable" states, so those tiles keep the same
// Title / Graphic / Foot shape instead of collapsing to centered text.
export function mutedGraphic(iconSvg) {
  return `<div class="graphic-muted">${iconSvg}</div>`;
}

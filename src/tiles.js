// Tile registry + render loop. Tiles are OS-agnostic view modules; each one is:
//   { id, title, intervalMs, fetch?, render(el, data), onMount?(el) }
// - intervalMs: 0 = no polling (event-driven or static); >0 = poll on that interval
// - fetch():   optional data source override; defaults to getTile(id)
// - render():  draw `data` into the tile's element
// The loop pauses polling whenever the document is hidden (resource strategy) and
// re-fetches everything when the panel becomes visible again.
import { getTile } from "./api.js";

const tiles = [];

export function registerTile(tile) {
  tiles.push({ intervalMs: 0, ...tile });
}

export function mountTiles() {
  const grid = document.getElementById("grid");
  tiles.forEach((t, i) => {
    const el = document.createElement("section");
    el.className = "tile tile--enter";
    el.dataset.id = t.id;
    el.style.animationDelay = `${i * 70}ms`; // staggered reveal
    el.setAttribute("aria-label", t.title);

    // A "?" help dot, persistent because the tile renders into a separate inner
    // content div (so re-renders don't wipe the dot). Clicks are delegated in main.js.
    const help = document.createElement("button");
    help.className = "tile-help";
    help.type = "button";
    help.dataset.help = t.id;
    help.setAttribute("aria-label", "What is this?");
    help.textContent = "?";
    el.appendChild(help);

    const content = document.createElement("div");
    content.className = "tile-content";
    el.appendChild(content);

    grid.appendChild(el);
    t._el = content; // tiles render into here
    t.onMount?.(content);
  });
}

function runner(t) {
  return async () => {
    const data = t.fetch ? await t.fetch() : await getTile(t.id);
    try {
      t.render(t._el, data);
    } catch (e) {
      console.error(`tile ${t.id} render failed`, e);
    }
  };
}

export function startRenderLoop() {
  for (const t of tiles) {
    t._run = runner(t);
    t._run();
    if (t.intervalMs > 0) {
      setInterval(() => {
        if (!document.hidden) t._run();
      }, t.intervalMs);
    }
  }
  document.addEventListener("visibilitychange", () => {
    if (!document.hidden) refreshAll();
  });
}

export function refreshAll() {
  for (const t of tiles) t._run?.();
}

export function refreshTile(id) {
  tiles.find((t) => t.id === id)?._run?.();
}

// Banner logic for the always-on-top high-memory warning window.
// The Rust watcher pushes a `mem-warn` event every ~2s while usage is over the
// threshold; we render the live percentage, the biggest memory user, and apply the
// user's chosen banner color (auto-contrasting the text for readability).
const tauri = window.__TAURI__ || {};
const event = tauri.event;
const core = tauri.core;

const banner = document.getElementById("banner");
const headline = document.getElementById("headline");
const sub = document.getElementById("sub");
const dismiss = document.getElementById("dismiss");

// Pick black or white text depending on how light the background color is.
function contrastText(hex) {
  const m = /^#?([0-9a-f]{6})$/i.exec(hex || "");
  if (!m) return "#ffffff";
  const n = parseInt(m[1], 16);
  const r = (n >> 16) & 255;
  const g = (n >> 8) & 255;
  const b = n & 255;
  // Relative luminance (sRGB, perceptual weights).
  const lum = (0.299 * r + 0.587 * g + 0.114 * b) / 255;
  return lum > 0.6 ? "#1a1a1a" : "#ffffff";
}

function applyColor(hex) {
  if (!/^#?[0-9a-f]{6}$/i.test(hex || "")) return;
  const color = hex[0] === "#" ? hex : "#" + hex;
  banner.style.background = color;
  banner.style.color = contrastText(color);
}

function render(p) {
  if (typeof p.percent === "number") {
    headline.textContent = `Memory almost full — ${p.percent}% used`;
  }
  if (p.proc && p.proc_mb) {
    const gb = (p.proc_mb / 1024).toFixed(1);
    sub.textContent = `${p.proc} is using ${gb} GB. Close some tabs or apps to free memory.`;
  } else {
    sub.textContent = "Your computer may become slow or unresponsive.";
  }
  applyColor(p.color);
}

if (event) {
  event.listen("mem-warn", (e) => render(e.payload || {}));
}

dismiss.addEventListener("click", () => {
  if (core) core.invoke("dismiss_mem_warn");
});

// Escalation modal: applies the user's warning color (from mem_warn_color, pushed on
// show), and wires the Open/Dismiss buttons.
const core = window.__TAURI__ && window.__TAURI__.core;
const event = window.__TAURI__ && window.__TAURI__.event;
const modal = document.getElementById("modal");

function contrastText(hex) {
  const m = /^#?([0-9a-f]{6})$/i.exec(hex || "");
  if (!m) return "#ffffff";
  const n = parseInt(m[1], 16);
  const r = (n >> 16) & 255, g = (n >> 8) & 255, b = n & 255;
  return (0.299 * r + 0.587 * g + 0.114 * b) / 255 > 0.6 ? "#1a1a1a" : "#ffffff";
}
function applyColor(hex) {
  if (!/^#?[0-9a-f]{6}$/i.test(hex || "")) return;
  const c = hex[0] === "#" ? hex : "#" + hex;
  modal.style.background = c;
  modal.style.color = contrastText(c);
}
if (event) event.listen("modal-color", (e) => applyColor(e.payload));

document.getElementById("open").addEventListener("click", () => {
  if (core) core.invoke("open_main_window");
});
document.getElementById("dismiss").addEventListener("click", () => {
  if (core) core.invoke("dismiss_mem_warn");
});

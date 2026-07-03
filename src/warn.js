// Escalation modal: "Open momPanel" and "Dismiss" buttons. Details are already
// delivered by the critical notification + speech; this is the visual last resort.
const core = window.__TAURI__ && window.__TAURI__.core;

document.getElementById("open").addEventListener("click", () => {
  if (core) core.invoke("open_main_window");
});
document.getElementById("dismiss").addEventListener("click", () => {
  if (core) core.invoke("dismiss_mem_warn");
});

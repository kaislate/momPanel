// Thin wrappers over the global Tauri API (withGlobalTauri: true in tauri.conf.json).
// Keeping all `window.__TAURI__` access in one place means tile code never touches
// the global directly and the app degrades gracefully if a command fails.

const core = window.__TAURI__?.core;
const event = window.__TAURI__?.event;

export async function invoke(cmd, args) {
  if (!core) throw new Error("Tauri core unavailable");
  return core.invoke(cmd, args);
}

export async function listen(name, handler) {
  if (!event) return () => {};
  return event.listen(name, handler);
}

// "About momPanel" overlay: logo + name, version, Visit GitHub, Check for updates,
// and an auto-update toggle (on by default). Rendered into #modal-root.
import {
  appVersion,
  openGithub,
  checkUpdates,
  getConfig,
  setConfig,
  getAutostart,
  setAutostart,
} from "./api.js";
import { logoSvg } from "./logo.js";
import { showWhatsNew } from "./whatsnew.js";

export async function openInfo() {
  const root = document.getElementById("modal-root");
  if (!root) return;

  const [version, cfg, autostart] = await Promise.all([
    appVersion(),
    getConfig(),
    getAutostart(),
  ]);

  root.innerHTML =
    `<div class="modal-backdrop"><div class="modal-card info-card">` +
    `<div class="info-logo">${logoSvg(96)}</div>` +
    `<div class="info-name">momPanel</div>` +
    `<div class="info-version">Version ${version || "—"}</div>` +
    `<div class="info-status" aria-live="polite"></div>` +
    `<button class="tile-btn info-btn" data-action="updates">Check for updates</button>` +
    `<button class="tile-btn info-btn" data-action="whatsnew">What's New</button>` +
    `<button class="tile-btn info-btn" data-action="github">Visit GitHub</button>` +
    `<label class="info-auto"><input type="checkbox" data-startup ${
      autostart ? "checked" : ""
    } /><span>Start automatically when I log in</span></label>` +
    `<label class="info-auto"><input type="checkbox" data-auto ${
      cfg.auto_update ? "checked" : ""
    } /><span>Update automatically</span></label>` +
    `<button class="tile-btn info-close" data-action="close">Close</button>` +
    `</div></div>`;

  const close = () => {
    root.innerHTML = "";
    document.removeEventListener("keydown", onKey);
  };
  const onKey = (e) => {
    if (e.key === "Escape") close();
  };
  document.addEventListener("keydown", onKey);

  const backdrop = root.querySelector(".modal-backdrop");
  backdrop.addEventListener("click", (e) => {
    if (e.target === backdrop) close();
  });
  root.querySelector('[data-action="close"]').addEventListener("click", close);
  root
    .querySelector('[data-action="github"]')
    .addEventListener("click", () => openGithub());
  root
    .querySelector('[data-action="whatsnew"]')
    .addEventListener("click", () => showWhatsNew(version));

  const status = root.querySelector(".info-status");
  root
    .querySelector('[data-action="updates"]')
    .addEventListener("click", async (e) => {
      const btn = e.currentTarget;
      btn.disabled = true;
      status.textContent = "Checking…";
      status.textContent = await checkUpdates();
      btn.disabled = false;
    });

  root.querySelector("[data-auto]").addEventListener("change", (e) => {
    setConfig({ auto_update: e.target.checked });
  });

  root.querySelector("[data-startup]").addEventListener("change", (e) => {
    setAutostart(e.target.checked);
  });
}

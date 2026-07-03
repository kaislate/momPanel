// "About momPanel" overlay: logo + name, version, Visit GitHub, Check for updates,
// and an auto-update toggle (on by default). Rendered into #modal-root.
import {
  appVersion,
  osInfo,
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

  const [version, os, cfg, autostart] = await Promise.all([
    appVersion(),
    osInfo(),
    getConfig(),
    getAutostart(),
  ]);

  // Threshold choices: 70–90% in 5% steps.
  const curPct = Math.round(cfg.mem_warn_percent || 85);
  const pctOptions = [70, 75, 80, 85, 90]
    .map((v) => `<option value="${v}" ${curPct === v ? "selected" : ""}>${v}%</option>`)
    .join("");
  const warnColor = cfg.mem_warn_color || "#D97706";

  root.innerHTML =
    `<div class="modal-backdrop"><div class="modal-card info-card">` +
    `<div class="info-logo">${logoSvg(96)}</div>` +
    `<div class="info-name">momPanel</div>` +
    `<div class="info-version">Version ${version || "—"}</div>` +
    `<div class="info-os">Running on ${os || "this computer"}</div>` +
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
    `<label class="info-auto"><input type="checkbox" data-memwarn ${
      cfg.mem_warn_enabled ? "checked" : ""
    } /><span>Warn me about high memory</span></label>` +
    `<label class="info-row"><span>Warn when memory reaches</span>` +
    `<select data-mempct>${pctOptions}</select></label>` +
    `<label class="info-row"><span>Warning color</span>` +
    `<input type="color" data-memcolor value="${warnColor}" /></label>` +
    `<label class="info-auto"><input type="checkbox" data-memsound ${
      cfg.mem_warn_sound_enabled ? "checked" : ""
    } /><span>Play an alert sound</span></label>` +
    `<label class="info-auto"><input type="checkbox" data-memspeech ${
      cfg.mem_warn_speech_enabled ? "checked" : ""
    } /><span>Speak the warning aloud</span></label>` +
    `<label class="info-auto"><input type="checkbox" data-mempulse ${
      cfg.mem_warn_pulse_enabled ? "checked" : ""
    } /><span>Repeat until resolved</span></label>` +
    `<label class="info-auto"><input type="checkbox" data-memescalate ${
      cfg.mem_warn_escalate_enabled ? "checked" : ""
    } /><span>Pop a dialog if ignored</span></label>` +
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

  root.querySelector("[data-memwarn]").addEventListener("change", (e) => {
    setConfig({ mem_warn_enabled: e.target.checked });
  });

  root.querySelector("[data-mempct]").addEventListener("change", (e) => {
    setConfig({ mem_warn_percent: Number(e.target.value) });
  });

  root.querySelector("[data-memcolor]").addEventListener("change", (e) => {
    setConfig({ mem_warn_color: e.target.value });
  });
  root.querySelector("[data-memsound]").addEventListener("change", (e) => {
    setConfig({ mem_warn_sound_enabled: e.target.checked });
  });
  root.querySelector("[data-memspeech]").addEventListener("change", (e) => {
    setConfig({ mem_warn_speech_enabled: e.target.checked });
  });
  root.querySelector("[data-mempulse]").addEventListener("change", (e) => {
    setConfig({ mem_warn_pulse_enabled: e.target.checked });
  });
  root.querySelector("[data-memescalate]").addEventListener("change", (e) => {
    setConfig({ mem_warn_escalate_enabled: e.target.checked });
  });
}

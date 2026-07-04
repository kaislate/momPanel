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
import { applyTheme, defaultTheme, PRESETS } from "./theme.js";

export async function openInfo() {
  const root = document.getElementById("modal-root");
  if (!root) return;

  const [version, os, cfg, autostart] = await Promise.all([
    appVersion(),
    osInfo(),
    getConfig(),
    getAutostart(),
  ]);

  const th = cfg.theme || defaultTheme();
  const presetOpts = [
    ["midnight", "Midnight"],
    ["warm", "Warm"],
    ["high-contrast", "High contrast"],
    ["custom", "Custom"],
  ]
    .map(([v, label]) => `<option value="${v}" ${th.preset === v ? "selected" : ""}>${label}</option>`)
    .join("");
  const swatch = (slot, label) =>
    `<label class="info-row"><span>${label}</span>` +
    `<input type="color" data-theme="${slot}" value="${th[slot]}" /></label>`;

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
    `<label class="info-row"><span>Alert sound</span>` +
    `<select data-memsoundchoice>` +
    [
      ["suspend-error", "Alarm (default)"],
      ["dialog-error", "Error"],
      ["dialog-warning", "Warning"],
      ["bell", "Bell"],
      ["alarm-clock-elapsed", "Alarm clock"],
    ]
      .map(([v, l]) => `<option value="${v}" ${cfg.mem_warn_sound === v ? "selected" : ""}>${l}</option>`)
      .join("") +
    `</select></label>` +
    `<label class="info-row"><span>Minimum alert volume</span>` +
    `<select data-memfloor>` +
    [50, 60, 70, 80, 90, 100]
      .map((v) => `<option value="${v / 100}" ${Math.round((cfg.mem_warn_volume_floor || 0.6) * 100) === v ? "selected" : ""}>${v}%</option>`)
      .join("") +
    `</select></label>` +
    `<div class="info-section">Theme</div>` +
    `<label class="info-row"><span>Preset</span><select data-preset>${presetOpts}</select></label>` +
    swatch("accent", "Accent") +
    swatch("bg", "Background") +
    swatch("tile", "Tiles") +
    swatch("gauge_ok", "Gauge · healthy") +
    swatch("gauge_warn", "Gauge · getting full") +
    swatch("gauge_bad", "Gauge · critical") +
    `<button class="tile-btn info-btn" data-action="theme-reset">Reset to default</button>` +
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
  root.querySelector("[data-memsoundchoice]").addEventListener("change", (e) => {
    setConfig({ mem_warn_sound: e.target.value });
  });
  root.querySelector("[data-memfloor]").addEventListener("change", (e) => {
    setConfig({ mem_warn_volume_floor: Number(e.target.value) });
  });

  // Live theme editing: apply immediately + persist. Editing a swatch marks it Custom.
  const presetSel = root.querySelector("[data-preset]");
  const readSwatches = () => {
    const t = { preset: presetSel.value };
    root.querySelectorAll("[data-theme]").forEach((el) => (t[el.dataset.theme] = el.value));
    return t;
  };

  presetSel.addEventListener("change", (e) => {
    const name = e.target.value;
    const base = PRESETS[name];
    if (base) {
      root.querySelectorAll("[data-theme]").forEach((el) => (el.value = base[el.dataset.theme]));
      const t = { preset: name, ...base };
      applyTheme(t);
      setConfig({ theme: t });
    }
  });

  root.querySelectorAll("[data-theme]").forEach((el) =>
    el.addEventListener("input", () => {
      presetSel.value = "custom";
      const t = { ...readSwatches(), preset: "custom" };
      applyTheme(t);
      setConfig({ theme: t });
    })
  );

  root.querySelector('[data-action="theme-reset"]').addEventListener("click", () => {
    const t = defaultTheme();
    presetSel.value = t.preset;
    root.querySelectorAll("[data-theme]").forEach((el) => (el.value = t[el.dataset.theme]));
    applyTheme(t);
    setConfig({ theme: t });
  });
}

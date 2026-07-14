// "About momPanel" overlay, laid out as one wide pane so every setting is visible
// at once (no scrolling to discover options): an identity header with the action
// buttons, then three columns — General, Memory alerts, Appearance.
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
import { closeActiveModal, setActiveModal } from "./modal.js";
import { escapeHtml } from "./escape.js";

export async function openInfo() {
  const root = document.getElementById("modal-root");
  if (!root) return;

  const [version, os, cfg, autostart] = await Promise.all([
    appVersion(),
    osInfo(),
    getConfig(),
    getAutostart(),
  ]);

  // Merge over the defaults so a partial stored theme can't yield "undefined"
  // swatch values (which render as black).
  const th = { ...defaultTheme(), ...(cfg.theme || {}) };
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

  const check = (attr, on, label) =>
    `<label class="info-auto"><input type="checkbox" ${attr} ${on ? "checked" : ""} /><span>${label}</span></label>`;

  // Close any overlay already open (this open* is async, so one could be) before we
  // overwrite #modal-root, so its keydown listener doesn't leak.
  closeActiveModal();

  root.innerHTML =
    `<div class="modal-backdrop"><div class="modal-card info-card">` +
    `<button class="info-x" data-action="close" aria-label="Close">&times;</button>` +
    `<div class="info-head">` +
    `<div class="info-logo">${logoSvg(60)}</div>` +
    `<div class="info-id">` +
    `<div class="info-name">momPanel</div>` +
    `<div class="info-version">Version ${escapeHtml(version || "—")} &middot; ${escapeHtml(os || "this computer")}</div>` +
    `<div class="info-status" aria-live="polite"></div>` +
    `</div>` +
    `<div class="info-actions">` +
    `<button class="tile-btn info-btn--slim" data-action="updates">Check for updates</button>` +
    `<button class="tile-btn info-btn--slim" data-action="whatsnew">What's New</button>` +
    `<button class="tile-btn info-btn--slim" data-action="github">GitHub</button>` +
    `</div>` +
    `</div>` +
    `<div class="info-cols">` +
    // --- Column 1: General ---
    `<section class="info-col"><h3 class="info-section">General</h3>` +
    check("data-startup", autostart, "Start automatically when I log in") +
    check("data-auto", cfg.auto_update, "Update automatically") +
    check(
      "data-experimental",
      cfg.experimental_ui,
      "Try the new look — Companion mode (experimental). momPanel refreshes when you switch."
    ) +
    `<label class="info-row"><span>Companion background</span>` +
    `<select data-compbg>` +
    [
      [1, "Solid"],
      [0.85, "Slightly clear"],
      [0.7, "Half clear"],
      [0.55, "Mostly clear"],
      [0.4, "Very clear"],
      [0, "Invisible — just the desktop"],
    ]
      .map(([v, l]) => {
        const cur = Number(cfg.companion_bg_opacity ?? 1);
        return `<option value="${v}" ${Math.abs(cur - v) < 0.01 ? "selected" : ""}>${l}</option>`;
      })
      .join("") +
    `</select></label>` +
    `</section>` +
    // --- Column 2: Memory alerts ---
    `<section class="info-col"><h3 class="info-section">Memory alerts</h3>` +
    check("data-memwarn", cfg.mem_warn_enabled, "Warn me about high memory") +
    `<label class="info-row"><span>Warn at</span><select data-mempct>${pctOptions}</select></label>` +
    `<label class="info-row"><span>Warning color</span>` +
    `<input type="color" data-memcolor value="${warnColor}" /></label>` +
    check("data-memsound", cfg.mem_warn_sound_enabled, "Play an alert sound") +
    check("data-memspeech", cfg.mem_warn_speech_enabled, "Speak the warning aloud") +
    check("data-mempulse", cfg.mem_warn_pulse_enabled, "Repeat until resolved") +
    check("data-memescalate", cfg.mem_warn_escalate_enabled, "Pop a dialog if ignored") +
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
    `<label class="info-row"><span>Min. alert volume</span>` +
    `<select data-memfloor>` +
    [50, 60, 70, 80, 90, 100]
      .map((v) => `<option value="${v / 100}" ${Math.round((cfg.mem_warn_volume_floor || 0.6) * 100) === v ? "selected" : ""}>${v}%</option>`)
      .join("") +
    `</select></label>` +
    `</section>` +
    // --- Column 3: Appearance ---
    `<section class="info-col"><h3 class="info-section">Appearance</h3>` +
    `<label class="info-row"><span>Preset</span><select data-preset>${presetOpts}</select></label>` +
    swatch("accent", "Accent") +
    swatch("bg", "Background") +
    swatch("tile", "Tiles") +
    swatch("gauge_ok", "Gauge · healthy") +
    swatch("gauge_warn", "Gauge · getting full") +
    swatch("gauge_bad", "Gauge · critical") +
    `<button class="tile-btn info-btn--slim info-reset" data-action="theme-reset">Reset to default</button>` +
    `</section>` +
    `</div></div></div>`;

  const close = () => {
    root.innerHTML = "";
    document.removeEventListener("keydown", onKey);
  };
  const onKey = (e) => {
    if (e.key === "Escape") close();
  };
  document.addEventListener("keydown", onKey);
  setActiveModal(close);

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

  // Companion mode swap: persist first, then reload so exactly one mode boots.
  root.querySelector("[data-experimental]").addEventListener("change", async (e) => {
    await setConfig({ experimental_ui: e.target.checked });
    location.reload();
  });

  // Companion sky opacity: persist and apply live (the CSS var is only consumed by
  // companion mode, so this is a harmless no-op in the classic grid).
  root.querySelector("[data-compbg]").addEventListener("change", (e) => {
    const v = Number(e.target.value);
    setConfig({ companion_bg_opacity: v });
    document.documentElement.style.setProperty("--comp-bg-alpha", String(v));
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

import assert from "node:assert/strict";
import { PRESETS, defaultTheme, contrastText } from "../src/theme.js";

// Every preset defines every slot.
const slots = ["accent", "bg", "tile", "gauge_ok", "gauge_warn", "gauge_bad"];
for (const [name, p] of Object.entries(PRESETS)) {
  for (const s of slots) {
    assert.ok(/^#[0-9a-fA-F]{6}$/.test(p[s]), `${name}.${s} must be #RRGGBB, got ${p[s]}`);
  }
}

// Default theme is midnight and carries all slots.
const d = defaultTheme();
assert.equal(d.preset, "midnight");
assert.equal(d.accent, PRESETS.midnight.accent);

// Contrast: dark bg -> light text; light bg -> dark text.
assert.equal(contrastText("#0e1119"), "#eef2ff");
assert.equal(contrastText("#f5f5f5"), "#1a1a1a");

console.log("theme.test.mjs OK");

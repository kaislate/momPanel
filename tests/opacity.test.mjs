import assert from "node:assert/strict";
import { OPACITY_STEPS, nearestStep } from "../src/opacity.js";

// Steps run from Solid to Invisible, respaced so "Very clear" is actually
// barely-there (the old 0.4 floor read as a heavy tint).
assert.equal(OPACITY_STEPS[0][0], 1);
assert.equal(OPACITY_STEPS[OPACITY_STEPS.length - 1][0], 0);
const values = OPACITY_STEPS.map(([v]) => v);
assert.ok(values.every((v, i) => i === 0 || v < values[i - 1]), "strictly descending");
assert.ok(!values.includes(0.4), "old heavy 'Very clear' value is gone");
assert.ok(values.some((v) => v > 0 && v <= 0.15), "has a barely-there step");

// Exact matches map to themselves.
for (const v of values) assert.equal(nearestStep(v), v);

// Legacy stored values (old step scale) snap to the closest new step instead of
// leaving the select with nothing selected.
assert.equal(nearestStep(0.85), 1); // old "Slightly clear" -> Solid (closest)
assert.equal(nearestStep(0.55), 0.45); // old "Mostly clear" -> Half clear
assert.equal(nearestStep(0.4), 0.45); // old "Very clear" -> Half clear
assert.equal(nearestStep(0.05), 0.1);

// Out-of-range and junk input degrade safely to Solid.
assert.equal(nearestStep(7), 1);
assert.equal(nearestStep(-1), 0);
assert.equal(nearestStep(NaN), 1);
assert.equal(nearestStep(undefined), 1);

console.log("opacity.test.mjs OK");

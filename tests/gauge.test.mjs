import assert from "node:assert/strict";
import { pickGauge } from "../src/gauge.js";

// Below 70 -> ok; 70..89 -> warn; 90+ -> bad.
assert.equal(pickGauge(10, "ok", "warn", "bad"), "ok");
assert.equal(pickGauge(75, "ok", "warn", "bad"), "warn");
assert.equal(pickGauge(95, "ok", "warn", "bad"), "bad");
assert.equal(pickGauge(70, "ok", "warn", "bad"), "warn");
assert.equal(pickGauge(90, "ok", "warn", "bad"), "bad");
console.log("gauge.test.mjs OK");

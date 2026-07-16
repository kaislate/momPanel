import assert from "node:assert/strict";
import { CHANGELOG, changesFor, versionsNewestFirst, neighbors } from "../src/changelog.js";

// The list mirrors the CHANGELOG object's order, which is newest-first.
const list = versionsNewestFirst();
assert.deepEqual(list, Object.keys(CHANGELOG));
assert.ok(list.length >= 2, "history has multiple entries");
assert.equal(changesFor(list[0]).changes.length > 0, true);

// Newest entry: can only step back.
assert.deepEqual(neighbors(list[0]), { older: list[1], newer: null });

// A middle entry steps both ways.
assert.deepEqual(neighbors(list[1]), { older: list[2], newer: list[0] });

// The oldest entry: can only step forward.
const last = list[list.length - 1];
assert.deepEqual(neighbors(last), { older: null, newer: list[list.length - 2] });

// An unknown version (e.g. a dev build with no notes yet) still offers the newest
// recorded entry as "older" so the history stays reachable.
assert.deepEqual(neighbors("99.9.9"), { older: list[0], newer: null });

console.log("changelog.test.mjs OK");

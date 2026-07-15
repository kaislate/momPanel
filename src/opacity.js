// Companion sky opacity steps (About → General → Companion background).
// Respaced in v0.6.3: the old scale bottomed out at 0.4 before jumping to 0, so
// "Very clear" still read as a heavy tint over the desktop. The new scale thins
// out gradually and ends at truly invisible.
export const OPACITY_STEPS = [
  [1, "Solid"],
  [0.7, "Slightly clear"],
  [0.45, "Half clear"],
  [0.25, "Mostly clear"],
  [0.1, "Very clear"],
  [0, "Invisible"],
];

// Snap a stored opacity to the closest step, so a value saved under the old scale
// (e.g. 0.85) still selects a sensible option instead of leaving the select blank.
// Junk input (NaN, undefined) degrades to Solid — the safe, fully-visible default.
export function nearestStep(value, steps = OPACITY_STEPS) {
  const v = Number(value);
  if (!Number.isFinite(v)) return 1;
  let best = steps[0][0];
  for (const [s] of steps) {
    if (Math.abs(s - v) < Math.abs(best - v)) best = s;
  }
  return best;
}

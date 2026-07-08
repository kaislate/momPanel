// Shared lifecycle guard for full-screen overlays (About, tile help, What's New).
// Each renders into the single #modal-root and attaches a document keydown listener
// torn down only by its own close(). Reopening one — or opening another over it —
// without closing first would stack those listeners, so we track a single "close the
// modal that's open now" fn: call closeActiveModal() just before writing new markup,
// then setActiveModal(close) to register the freshly opened one.

let activeClose = null;

// Tear down whatever modal is open, if any. Safe to call when none is (no-op).
export function closeActiveModal() {
  const fn = activeClose;
  activeClose = null;
  fn?.();
}

// Register the just-opened modal's teardown as the current one.
export function setActiveModal(close) {
  activeClose = close;
}

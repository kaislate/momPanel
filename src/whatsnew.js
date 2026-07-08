// "What's new" popup: a warm welcome to the new version with a plain-English list of
// changes. Shown automatically after an update, and re-openable from the About panel.
import { changesFor } from "./changelog.js";
import { closeActiveModal, setActiveModal } from "./modal.js";

export function showWhatsNew(version) {
  const root = document.getElementById("modal-root");
  if (!root) return;

  const entry = changesFor(version);
  const items = entry?.changes ?? [];
  const body = items.length
    ? `<ul class="whatsnew-list">${items.map((c) => `<li>${c}</li>`).join("")}</ul>`
    : `<div class="tile-sub">Thanks for keeping momPanel up to date! 💛</div>`;

  // What's New opens from the About panel, so a modal is already up — close it first so
  // its keydown listener doesn't leak when we overwrite #modal-root.
  closeActiveModal();

  root.innerHTML =
    `<div class="modal-backdrop"><div class="modal-card whatsnew-card">` +
    `<div class="whatsnew-emoji">🎉</div>` +
    `<div class="info-name">Welcome to momPanel ${version}</div>` +
    `<div class="info-version">Here's what's new</div>` +
    body +
    `<button class="tile-btn info-close" data-action="close">Got it</button>` +
    `</div></div>`;

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
}

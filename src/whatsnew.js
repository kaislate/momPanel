// "What's new" popup: a warm welcome to the new version with a plain-English list of
// changes. Shown automatically after an update, and re-openable from the About panel.
// Past versions' notes are never overwritten (see changelog.js), so the popup lets
// you page back through them with Older/Newer links at the bottom.
import { changesFor, neighbors } from "./changelog.js";
import { closeActiveModal, setActiveModal } from "./modal.js";

export function showWhatsNew(version) {
  const root = document.getElementById("modal-root");
  if (!root) return;

  // What's New opens from the About panel, so a modal is already up — close it first so
  // its keydown listener doesn't leak when we overwrite #modal-root.
  closeActiveModal();

  const close = () => {
    root.innerHTML = "";
    document.removeEventListener("keydown", onKey);
  };
  const onKey = (e) => {
    if (e.key === "Escape") close();
  };
  document.addEventListener("keydown", onKey);
  setActiveModal(close);

  // Render one version's notes; the Older/Newer links repaint in place.
  const paint = (v) => {
    const entry = changesFor(v);
    const items = entry?.changes ?? [];
    const body = items.length
      ? `<ul class="whatsnew-list">${items.map((c) => `<li>${c}</li>`).join("")}</ul>`
      : `<div class="tile-sub">Thanks for keeping momPanel up to date! 💛</div>`;

    const isCurrent = v === version;
    const { older, newer } = neighbors(v);
    const nav =
      older || newer
        ? `<div class="whatsnew-nav">` +
          (older ? `<a data-nav="${older}">&lsaquo; Older updates</a>` : `<span></span>`) +
          (newer ? `<a data-nav="${newer}">Newer updates &rsaquo;</a>` : `<span></span>`) +
          `</div>`
        : "";

    root.innerHTML =
      `<div class="modal-backdrop"><div class="modal-card whatsnew-card">` +
      `<div class="whatsnew-emoji">${isCurrent ? "🎉" : "🕰️"}</div>` +
      `<div class="info-name">${isCurrent ? `Welcome to momPanel ${v}` : `momPanel ${v}`}</div>` +
      `<div class="info-version">${isCurrent ? "Here's what's new" : "What was new in this update"}</div>` +
      body +
      `<button class="tile-btn info-close" data-action="close">Got it</button>` +
      nav +
      `</div></div>`;

    const backdrop = root.querySelector(".modal-backdrop");
    backdrop.addEventListener("click", (e) => {
      if (e.target === backdrop) close();
    });
    root.querySelector('[data-action="close"]').addEventListener("click", close);
    root
      .querySelectorAll("[data-nav]")
      .forEach((a) => a.addEventListener("click", () => paint(a.dataset.nav)));
  };

  paint(version);
}

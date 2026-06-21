// Plain-language "what is this?" explanations for each tile, shown in a small popup
// when the tile's "?" dot is clicked. Written for a non-technical reader.
export const HELP = {
  clock: {
    title: "Clock",
    text: "The current time. Tap the button to switch between a clock face and numbers.",
  },
  date: {
    title: "Date",
    text: "Today's day and date, with a calendar of the whole month. Today is highlighted.",
  },
  cpu: {
    title: "CPU",
    text: "How hard your computer's 'brain' is working right now. A low number is normal and good — it only climbs while it's busy doing something.",
  },
  memory: {
    title: "Memory",
    text: "Memory (also called RAM) is your computer's short-term workspace for whatever it's doing right now. It's normal for it to fill up. To free some up, close programs you aren't using, close extra tabs in Opera, or restart the computer.",
  },
  storage: {
    title: "Storage",
    text: "This shows how full your storage is — the space where your files, photos, and programs are kept for good. Tap the circle to switch between how full it is and how much room is still free.",
  },
  wifi: {
    title: "Wi-Fi",
    text: "Whether you're connected to a wireless (Wi-Fi) network, and how strong the signal is. If you use a cable instead, this can say 'not found' — that's okay.",
  },
  internet: {
    title: "Internet",
    text: "Whether your computer can reach the internet right now. 'Online' means web pages and email will work.",
  },
  printers: {
    title: "Printers",
    text: "The printers your computer knows about and whether they're ready to print.",
  },
  volume: {
    title: "Volume",
    text: "How loud your computer's sound is set, and whether it's muted.",
  },
  weather: {
    title: "Weather",
    text: "The weather right now and for the next few days, for the ZIP code you entered.",
  },
};

// Show a small explanation popup for a tile id.
export function showHelp(id) {
  const root = document.getElementById("modal-root");
  const item = HELP[id];
  if (!root || !item) return;

  root.innerHTML =
    `<div class="modal-backdrop"><div class="modal-card help-card">` +
    `<div class="help-title">${item.title}</div>` +
    `<div class="help-text">${item.text}</div>` +
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
  const backdrop = root.querySelector(".modal-backdrop");
  backdrop.addEventListener("click", (e) => {
    if (e.target === backdrop) close();
  });
  root.querySelector('[data-action="close"]').addEventListener("click", close);
}

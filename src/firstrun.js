// First-run / change-location modal. Collects a validated 5-digit US ZIP code.
//
// promptZip(currentZip) renders a .modal-backdrop/.modal-card into #modal-root and
// resolves to the entered ZIP string (validated against /^\d{5}$/) or null if the
// user cancels. Only one modal is shown at a time; opening again replaces it.

const ZIP_RE = /^\d{5}$/;

export function promptZip(currentZip = "") {
  return new Promise((resolve) => {
    const root = document.getElementById("modal-root");
    if (!root) {
      resolve(null);
      return;
    }
    // Replace any existing modal.
    root.innerHTML = "";

    const backdrop = document.createElement("div");
    backdrop.className = "modal-backdrop";

    const card = document.createElement("div");
    card.className = "modal-card";

    const title = document.createElement("div");
    title.className = "tile-title";
    title.textContent = "Set your location";

    const hint = document.createElement("div");
    hint.className = "tile-sub";
    hint.textContent = "Enter a 5-digit US ZIP code for the weather.";

    const input = document.createElement("input");
    input.type = "text";
    input.inputMode = "numeric";
    input.maxLength = 5;
    input.placeholder = "90210";
    input.value = currentZip ? String(currentZip) : "";
    input.setAttribute("aria-label", "ZIP code");

    const err = document.createElement("div");
    err.className = "tile-sub";
    err.style.color = "#ff5d5d";
    err.style.minHeight = "1.2em";
    err.textContent = "";

    const okBtn = document.createElement("button");
    okBtn.className = "tile-btn";
    okBtn.textContent = "OK";

    const cancelBtn = document.createElement("button");
    cancelBtn.className = "tile-btn";
    cancelBtn.textContent = "Cancel";
    cancelBtn.style.marginLeft = "10px";
    cancelBtn.style.background = "#2c3450";

    const row = document.createElement("div");
    row.appendChild(okBtn);
    row.appendChild(cancelBtn);

    card.appendChild(title);
    card.appendChild(hint);
    card.appendChild(input);
    card.appendChild(err);
    card.appendChild(row);
    backdrop.appendChild(card);
    root.appendChild(backdrop);

    let settled = false;
    function close(result) {
      if (settled) return;
      settled = true;
      root.innerHTML = "";
      document.removeEventListener("keydown", onKey);
      resolve(result);
    }

    function submit() {
      const zip = input.value.trim();
      if (ZIP_RE.test(zip)) {
        close(zip);
      } else {
        err.textContent = "Please enter exactly 5 digits.";
        input.focus();
        input.select();
      }
    }

    function onKey(e) {
      if (e.key === "Enter") {
        e.preventDefault();
        submit();
      } else if (e.key === "Escape") {
        e.preventDefault();
        close(null);
      }
    }

    okBtn.addEventListener("click", submit);
    cancelBtn.addEventListener("click", () => close(null));
    backdrop.addEventListener("click", (e) => {
      if (e.target === backdrop) close(null);
    });
    document.addEventListener("keydown", onKey);

    // Focus after the element is in the DOM.
    requestAnimationFrame(() => {
      input.focus();
      input.select();
    });
  });
}

// Convenience alias for the "change location" flow; identical behaviour.
export function openLocationModal(currentZip = "") {
  return promptZip(currentZip);
}

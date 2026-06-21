// Shared HTML escaper for any system-derived string rendered via innerHTML
// (Wi-Fi SSIDs, printer names, weather place names). System data is untrusted —
// a crafted Wi-Fi SSID could otherwise inject markup into the webview.
export function escapeHtml(s) {
  return String(s).replace(
    /[&<>"']/g,
    (c) =>
      ({
        "&": "&amp;",
        "<": "&lt;",
        ">": "&gt;",
        '"': "&quot;",
        "'": "&#39;",
      }[c])
  );
}

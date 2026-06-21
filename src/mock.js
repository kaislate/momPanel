// Representative mock data so the frontend renders standalone (no backend / on the
// Windows dev box). Each tile agent should keep its real data shape matching these.
const FIXTURES = {
  cpu: { state: "ok", used_percent: 27.0 },
  memory: { state: "ok", used_percent: 42.0, total_mb: 16384, used_mb: 6881 },
  storage: { state: "ok", used_percent: 63.0, free_gb: 180, total_gb: 488 },
  wifi: { state: "ok", ssid: "HomeWifi", signal_percent: 78 },
  internet: { state: "ok", online: true },
  volume: { state: "ok", level_percent: 55, muted: false },
  printers: {
    state: "ok",
    default_name: "Office_LaserJet",
    printers: [
      { name: "Office_LaserJet", status: "ready" },
      { name: "Photo", status: "out_of_paper" },
    ],
  },
  weather: {
    state: "ok",
    temp_f: 71,
    code: 1,
    high_f: 75,
    low_f: 57,
    place: "Beverly Hills",
    days: [
      { date: "2026-06-20", code: 1, high_f: 75, low_f: 57, precip_prob: 5 },
      { date: "2026-06-21", code: 61, high_f: 72, low_f: 55, precip_prob: 60 },
      { date: "2026-06-22", code: 3, high_f: 70, low_f: 54, precip_prob: 20 },
      { date: "2026-06-23", code: 0, high_f: 79, low_f: 59, precip_prob: 0 },
      { date: "2026-06-24", code: 95, high_f: 73, low_f: 61, precip_prob: 70 },
    ],
  },
};

export async function mockTile(name) {
  return FIXTURES[name] ?? { state: "unavailable" };
}

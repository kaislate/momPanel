// Representative mock data so the frontend renders standalone (no backend / on the
// Windows dev box). Each tile agent should keep its real data shape matching these.
const FIXTURES = {
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
    temp_c: 21.5,
    code: 1,
    high_c: 24,
    low_c: 14,
    place: "Beverly Hills",
  },
};

export async function mockTile(name) {
  return FIXTURES[name] ?? { state: "unavailable" };
}

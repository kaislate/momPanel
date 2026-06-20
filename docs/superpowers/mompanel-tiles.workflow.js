export const meta = {
  name: 'mompanel-tiles',
  description: 'Build momPanel collectors + tiles in parallel, then integrate and verify',
  phases: [
    { title: 'Tiles', detail: 'one agent per tile: isolated Rust collector + JS tile + unit tests' },
    { title: 'Integrate', detail: 'wire snippets into shared files, add shortcuts backend, cargo test + build' },
  ],
}

// ---- Shared contract every tile agent must follow -------------------------
const CONTRACT = `
PROJECT: momPanel — a Tauri 2 desktop info panel. Repo root:
  C:/Documents/NEw project/Project 11/momPanel
Rust backend: src-tauri/  (crate lib name "app_lib", commands in src/lib.rs)
Web frontend: src/        (vanilla JS, NO bundler, withGlobalTauri=true)

TARGET: Linux/Zorin (GNOME) is the REAL implementation. Windows is test-only:
on Windows (and on any failure) a collector MUST return its Unavailable variant —
NEVER panic, NEVER block. Shell out with std::process::Command; map every Err to
Unavailable.

RUST COLLECTOR CONVENTION (each collector is its OWN file, e.g. src-tauri/src/collectors/memory.rs):
  use serde::Serialize;
  #[derive(Serialize)]
  #[serde(tag = "state", rename_all = "lowercase")]
  pub enum <Name>Data { Ok { /* fields */ }, Unavailable }
  pub fn read() -> <Name>Data { ... }            // pure dispatch; Linux real, else Unavailable
  // Put any PARSING logic in a free function (e.g. parse_nmcli(&str)) and unit-test it
  // with #[cfg(test)] mod tests so it runs on Windows without the real tool.
Serialized shape MUST be {"state":"ok", ...fields} or {"state":"unavailable"}.
Do NOT add dependencies — sysinfo="0.33", dirs, reqwest(blocking,json), serde, serde_json
are already in Cargo.toml. sysinfo 0.33 API: System::new(), sys.refresh_memory(),
sys.total_memory()/used_memory() return BYTES; Disks::new_with_refreshed_list().

FRONTEND TILE CONTRACT (each tile is its own file, e.g. src/tiles/memory.js):
  export function register(registerTile) {
    registerTile({
      id: "<id>", title: "<Title>", intervalMs: <number>,   // 0 = no polling
      // fetch optional; default fetch calls getTile(id). Override only when needed.
      render(el, data) { /* set el.innerHTML; handle data.state==="unavailable" calmly */ },
    });
  }
Helpers you may import (already exist):
  ./tiles.js   -> registerTile is passed in; also refreshTile(id) exported
  ./api.js     -> getTile(name), readWeather(zip), getConfig(), setConfig(cfg), openSettings(target)
  ./gauge.js   -> arcGauge(percent, label, sub) returns an SVG string; gaugeColor(p)
CSS classes available: .tile-title .tile-big .tile-sub .tile--unavailable .tile-btn .gauge
The data shapes match src/mock.js — keep them identical.

UNAVAILABLE RENDER: show a calm centered "Not available" using class tile--unavailable.

STRICT FILE OWNERSHIP: create ONLY your own new files. DO NOT edit any shared file
(src-tauri/src/lib.rs, src-tauri/src/collectors/mod.rs, src-tauri/Cargo.toml,
src/main.js, src/tiles/index.js, src/tiles.js). DO NOT run cargo or npm — your module
is not wired into the crate yet, so it cannot compile in isolation; the integration
step compiles everything. Return the wiring snippets instead.
`

const AGENT_SCHEMA = {
  type: 'object',
  additionalProperties: false,
  required: ['id', 'created_files', 'rust_mod_line', 'read_tile_arm',
             'extra_command_name', 'extra_command_fn', 'js_import', 'js_register_call', 'notes'],
  properties: {
    id: { type: 'string' },
    created_files: { type: 'array', items: { type: 'string' } },
    rust_mod_line: { type: ['string', 'null'], description: 'e.g. "pub mod memory;" or null if frontend-only' },
    read_tile_arm: { type: ['string', 'null'], description: 'match arm for read_tile, or null' },
    extra_command_name: { type: ['string', 'null'], description: 'e.g. "read_weather" or null' },
    extra_command_fn: { type: ['string', 'null'], description: 'full #[tauri::command] fn source to paste into lib.rs, or null' },
    js_import: { type: 'string', description: 'import line for src/tiles/index.js' },
    js_register_call: { type: 'string', description: 'call line for registerAll(), e.g. "registerMemory(registerTile);"' },
    notes: { type: 'string', description: 'anything the integrator must know (Linux-only verification, edge cases)' },
  },
}

// ---- Per-tile specs --------------------------------------------------------
const SPECS = [
  {
    id: 'memory',
    task: `Build the MEMORY tile.
Rust src-tauri/src/collectors/memory.rs: MemoryData::Ok{used_percent:f32,total_mb:u64,used_mb:u64}.
read(): sysinfo System::new()+refresh_memory(); total/used are BYTES; total_mb=total/1048576.
Free fn percent(used:u64,total:u64)->f32 (0.0 if total==0). Unit-test percent (50%, and 0/0=0).
If total==0 -> Unavailable.
Frontend src/tiles/memory.js: id "memory", intervalMs 3000, render with arcGauge(used_percent,
Math.round(used_percent)+"%", Math.round(used_mb/1024)+" / "+Math.round(total_mb/1024)+" GB")
under a .tile-title "Memory".`,
  },
  {
    id: 'storage',
    task: `Build the STORAGE tile.
Rust src-tauri/src/collectors/storage.rs: StorageData::Ok{used_percent:f32,free_gb:u64,total_gb:u64}.
read(): sysinfo Disks::new_with_refreshed_list(); pick the disk with the largest total_space()
(or the one mounted at "/" on Linux). total_gb=total/1073741824, free from available_space().
Free fn percent(used:u64,total:u64)->f32 tested. Empty disk list -> Unavailable.
Frontend src/tiles/storage.js: id "storage", intervalMs 30000, arcGauge(used_percent, free_gb+" GB",
"free of "+total_gb+" GB") + a .tile-btn "Open storage" calling openSettings("storage").`,
  },
  {
    id: 'wifi',
    task: `Build the WIFI tile.
Rust src-tauri/src/collectors/wifi.rs: WifiData::Ok{ssid:String,signal_percent:u8}.
read(): run "nmcli -t -f ACTIVE,SSID,SIGNAL dev wifi"; free fn parse_nmcli(&str)->Option<(String,u8)>
returns the row whose first colon-field is "yes" -> (ssid, signal). Unit-test: active row picked,
none when no active row. No nmcli / parse fail -> Unavailable.
Frontend src/tiles/wifi.js: id "wifi", intervalMs 20000. Render concentric signal arcs (an SVG with
3-4 nested arcs filled proportional to signal_percent) + ssid in .tile-big and a .tile-btn
"Wi-Fi settings" calling openSettings("wifi").`,
  },
  {
    id: 'internet',
    task: `Build the INTERNET tile.
Rust src-tauri/src/collectors/internet.rs: InternetData::Ok{online:bool}.
read(): Linux try "nmcli networking connectivity" == "full" -> online. Fallback (all OSes):
TcpStream::connect_timeout to 1.1.1.1:53 with a 1500ms timeout -> online=true. Any error path
still returns Ok{online:false} (NOT Unavailable — offline is a valid state). Provide a free fn
classify_connectivity(&str)->bool ("full"->true else false) and unit-test it.
Frontend src/tiles/internet.js: id "internet", intervalMs 20000. Big green globe + "Online" when
online, red globe + "Offline" otherwise (use inline SVG; color via .tile-big style).`,
  },
  {
    id: 'volume',
    task: `Build the VOLUME tile.
Rust src-tauri/src/collectors/volume.rs: VolumeData::Ok{level_percent:u8,muted:bool}.
read(): run "wpctl get-volume @DEFAULT_AUDIO_SINK@" -> "Volume: 0.65 [MUTED]". Free fn
parse_wpctl(&str)->Option<(u8,bool)>: level=round(frac*100), muted if "[MUTED]" present.
Unit-test muted+level and unmuted. No wpctl -> Unavailable.
Frontend src/tiles/volume.js: id "volume", intervalMs 5000. Speaker icon (muted variant when muted)
+ arcGauge(level_percent, level_percent+"%", muted?"muted":"") + .tile-btn "Sound settings"
calling openSettings("sound").`,
  },
  {
    id: 'printers',
    task: `Build the PRINTERS tile.
Rust src-tauri/src/collectors/printers.rs:
  #[derive(Serialize)] struct PrinterInfo{name:String,status:String}
  PrintersData::Ok{printers:Vec<PrinterInfo>,default_name:Option<String>} | Unavailable.
read(): run "lpstat -p -d". Free fn parse_lpstat(&str)->PrintersData::Ok-data: lines starting
"printer <name> is idle"/"is processing" -> status "ready"; "disabled"/"offline" -> "offline";
else "unknown". Line "system default destination: <name>" -> default_name. Empty list is still
Ok (friendly "No printers"). No lpstat -> Unavailable. Unit-test: 2 printers + default parsed,
first status "ready". (Return type note: have parse_lpstat return the concrete struct data so it
is testable; read() maps missing tool to Unavailable.)
Frontend src/tiles/printers.js: id "printers", intervalMs 30000. One printer chip per entry with a
colored status dot (ready=green, offline/out_of_paper=red/amber, unknown=grey); highlight the
default. Empty -> friendly "No printers connected". + .tile-btn "Printer settings"
calling openSettings("printers").`,
  },
  {
    id: 'weather',
    task: `Build the WEATHER tile (uses a dedicated command read_weather(zip), NOT read_tile).
Rust src-tauri/src/collectors/weather.rs:
  WeatherData::Ok{temp_c:f32,code:u8,high_c:f32,low_c:f32,place:String} | Unavailable.
  Free fn condition(code:u8)->&'static str mapping WMO codes: 0->"clear", 1..=3->"cloudy",
  45|48->"fog", 51..=67->"rain", 71..=86->"snow", 95..=99->"thunder", else "cloudy".
  Unit-test condition(0)=="clear", 61=="rain", 71=="snow", 95=="thunder".
  pub fn read(zip:&str)->WeatherData using reqwest::blocking with short timeouts:
   1) GET https://api.zippopotam.us/us/<zip> -> places[0].latitude/longitude + "place name"+", "+state abbr.
   2) GET https://api.open-meteo.com/v1/forecast?latitude=..&longitude=..&current=temperature_2m,weather_code
      &daily=temperature_2m_max,temperature_2m_min&timezone=auto
   Build WeatherData; ANY network/parse error -> Unavailable.
ALSO provide the command source to return via extra_command_fn / extra_command_name:
  #[tauri::command] fn read_weather(zip: String) -> serde_json::Value {
      serde_json::to_value(crate::collectors::weather::read(&zip))
          .unwrap_or_else(|_| serde_json::json!({"state":"unavailable"})) }
Frontend:
  src/tiles/weather.js: id "weather", intervalMs 1200000 (20 min). On first render, ensure a ZIP
   exists: read getConfig(); if no zip, open the first-run modal (see firstrun.js) to collect a
   5-digit ZIP, setConfig({zip}). fetch(): const {zip}=await getConfig(); return zip? readWeather(zip):
   {state:"unavailable"}. Render: condition icon (inline SVG keyed by condition string derived from
   code — replicate the same WMO mapping in JS), temp_c rounded + "°", high/low, place, and a small
   "change location" link that reopens the modal then refreshTile("weather").
  src/firstrun.js: export async function promptZip(currentZip) -> resolves to a validated 5-digit ZIP
   string (or null if cancelled). Render a .modal-backdrop/.modal-card into #modal-root with an input,
   OK/Cancel, validating /^\\d{5}$/. Export also openLocationModal() helper if convenient.
Note for integrator: register read_weather in the invoke_handler list.`,
  },
  {
    id: 'clock',
    task: `Build the CLOCK + DATE tiles (frontend ONLY — no Rust collector, all fields null).
src/tiles/clock.js: export function register(registerTile) that registers TWO tiles:
  - id "clock", intervalMs 1000, fetch: async()=>null. Mode (analog|digital) loaded from
    getConfig().clock_mode (default "digital"); keep it in a module-level let. render(el):
      * digital: big HH:MM:SS (toLocaleTimeString) in .tile-big + .tile-title "Clock".
      * analog: inline SVG 100x100 clock face showing ALL TWELVE NUMERALS (1..12) positioned with
        <text> on a circle radius ~38 at angle n*30deg (remember SVG y grows downward; 12 at top),
        tick marks, and hour/minute/second hands rotated hour*30+min*0.5 / min*6 / sec*6 degrees.
      * include a small .tile-btn "toggle" that flips mode, calls setConfig({clock_mode:newMode}),
        and re-renders immediately.
  - id "date", intervalMs 1000, fetch: async()=>null. render(el): weekday (toLocaleDateString
    {weekday:"long"}) in .tile-title and the date (toLocaleDateString) in .tile-big, calendar-styled.
Persisted mode must survive restart (it reads getConfig on register). rust_mod_line/read_tile_arm/
extra_command_* are null. js_import imports {register as registerClock}; js_register_call calls
"await registerClock(registerTile);".`,
  },
]

// ---- Phase 1: fan out one agent per tile ----------------------------------
phase('Tiles')
const results = await parallel(
  SPECS.map((spec) => () =>
    agent(`${CONTRACT}\n\nYOUR TASK (${spec.id}):\n${spec.task}\n\nWhen done, return the wiring snippets per the schema. created_files = absolute or repo-relative paths you created.`,
      { label: `tile:${spec.id}`, phase: 'Tiles', schema: AGENT_SCHEMA })
  )
)
const ok = results.filter(Boolean)
log(`Tiles complete: ${ok.length}/${SPECS.length} agents returned`)

// ---- Phase 2: single integration agent ------------------------------------
phase('Integrate')
const integrationPrompt = `
You are the INTEGRATION step for momPanel (Tauri 2). Repo root:
  C:/Documents/NEw project/Project 11/momPanel
Eight tile agents each created their own files and returned wiring snippets (below as JSON).
Your job: wire them into the shared files, add the safe shortcuts backend, then COMPILE and TEST.

SNIPPETS (JSON array):
${JSON.stringify(ok, null, 2)}

DO THIS:
1) src-tauri/src/collectors/mod.rs: add each non-null rust_mod_line (one "pub mod X;" per collector).
2) src-tauri/src/lib.rs:
   - Insert each non-null read_tile_arm into the read_tile match (before the "_ => unavail()," arm).
   - Paste each non-null extra_command_fn (e.g. read_weather) into the file.
   - Add every extra_command_name AND open_settings to the generate_handler! list, keeping
     read_tile, get_config, set_config.
3) Create the shortcuts backend src-tauri/src/shortcuts.rs and "mod shortcuts;" in lib.rs:
     pub fn linux_cmd(t:&str)->Option<(&'static str,Vec<&'static str>)> mapping
       "wifi"->("gnome-control-center",vec!["wifi"]), "printers"->(...,vec!["printers"]),
       "sound"->(...,vec!["sound"]), "storage"->("gnome-disks",vec![]), else None.
     #[tauri::command] pub fn open_settings(target:String)->Result<(),String>: on Linux spawn via
       linux_cmd; on Windows spawn "cmd /C start ms-settings:<x>" best-effort (network/printers/sound),
       unknown target -> Err. Never run anything destructive.
     Unit-test linux_cmd known + unknown targets.
4) src/tiles/index.js: replace the placeholder loop in registerAll() with the real imports
   (each js_import) and calls (each js_register_call). Keep registerAll async (clock uses await).
5) Compile + test from src-tauri:  cargo test    then    cargo build
   Fix every error until BOTH pass. Do NOT change any tile's data shape; fix wiring/types only.
   (Use PowerShell; the repo path has spaces — quote it. Do not run "tauri dev"/"tauri build".)

Return a concise report: what you wired, the final 'cargo test' summary line (e.g. "test result:
ok. N passed"), 'cargo build' result, and any tile you could NOT make compile (with the reason).
`
const report = await agent(integrationPrompt, { label: 'integrate', phase: 'Integrate', effort: 'high' })

return { tiles: ok.map((r) => r.id), report }

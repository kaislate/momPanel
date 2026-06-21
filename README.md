<div align="center">

<img src="assets/logo.svg" width="140" alt="momPanel logo" />

# 👵 momPanel

**A calm, graphics‑first desktop info panel for people who find Settings intimidating.** 💛

[![Latest release](https://img.shields.io/github/v/release/kaislate/momPanel?label=release)](https://github.com/kaislate/momPanel/releases/latest)
[![Linux](https://img.shields.io/badge/Linux-Zorin%20OS-1793D1?logo=linux&logoColor=white)](#-install)
[![Windows](https://img.shields.io/badge/Windows-testing-0078D6?logo=windows&logoColor=white)](#-install)
[![Built with Tauri](https://img.shields.io/badge/built%20with-Tauri-24C8DB?logo=tauri&logoColor=white)](https://tauri.app)

</div>

---

momPanel shows the things that actually matter — at a glance, in plain language, with big friendly graphics instead of buried menus. It was built for a non‑technical user (hi, Mom 👋) who just wants to know *"is my computer okay?"* without hunting through control panels.

## ✨ Highlights

- 🖼️ **Graphics‑first tiles** — gauges, icons, and big readable numbers instead of walls of text.
- 🪟 **Frameless & fixed** — can't be accidentally moved, resized, or closed.
- 🗣️ **Plain language** — "Plenty of room", "You're connected to the internet", "Out of paper".
- ❓ **Per‑tile help** — a "?" on each tile pops a simple explanation of what it means.
- 🔠 **Make everything bigger** — one A− / A+ control scales the whole panel (and the window) for easy reading.
- 🧘 **Calm mode** — honors your system "reduce motion" setting.
- 🖱️ **Safe shortcuts** — friendly buttons that *open the right settings screen*; momPanel never changes settings itself.
- 🔄 **Auto‑updates** — quietly updates itself from GitHub Releases (signed).
- 🔒 **Private** — your weather ZIP is stored **only on your machine**, never bundled or shared.

## 🧩 The tiles

| Tile | What it shows |
|---|---|
| 🕒 **Clock** | Time, as a numbers display or an analog face (your choice) |
| 📅 **Date** | Today + a full month calendar with today highlighted |
| 🧠 **CPU** | How hard the processor is working right now |
| 🧩 **Memory** | How much short‑term memory (RAM) is in use |
| 💾 **Storage** | How full your drive is — tap to flip between % full and % free |
| 📶 **Wi‑Fi** | Network name and signal strength |
| 🌐 **Internet** | A clear Online / Offline indicator |
| 🖨️ **Printers** | Your printers and whether they're ready |
| 🔊 **Volume** | Current sound level / muted |
| ⛅ **Weather** | Now + a 5‑day forecast (°F) for your ZIP |

> 🐧 Wi‑Fi, Printers, and Volume read live system info on **Linux** (the primary target). On Windows they show a friendly "not available here" — Windows is for development/testing.

## 🖼️ Screenshots

<div align="center">

<!-- Drop screenshots here, e.g.: -->
<!-- <img src="assets/screenshot-dashboard.png" width="720" alt="momPanel dashboard" /> -->

_Screenshots coming soon._ 📸

</div>

## 🚀 Install

### 🐧 Linux / Zorin OS — primary target

1. Download **`momPanel_x.y.z_amd64.AppImage`** from the [latest release](https://github.com/kaislate/momPanel/releases/latest).
2. Make it executable and run it:
   ```bash
   chmod +x momPanel_*_amd64.AppImage
   ./momPanel_*_amd64.AppImage
   ```
   *(`.deb` and `.rpm` packages are also provided. The **AppImage** is the self‑updating one.)*

momPanel registers itself to **start automatically on login** and keeps itself **up to date**.

### 🪟 Windows — for testing

Download and run the **`_x64-setup.exe`** (or the `.msi`) from the [latest release](https://github.com/kaislate/momPanel/releases/latest).

## 🛠️ Build from source

**Prerequisites:** [Node.js](https://nodejs.org), the [Rust toolchain](https://rustup.rs), and the [Tauri system dependencies](https://tauri.app/start/prerequisites/) for your OS.

```bash
git clone https://github.com/kaislate/momPanel.git
cd momPanel
npm install

npm run tauri dev      # 🔧 run in development
npm run tauri build    # 📦 build installers for your platform
```

## 🔄 Updates

On launch, momPanel checks **GitHub Releases** for a newer **signed** version and installs it silently. You can also open the **ℹ️ About** panel to **Check for updates** manually or turn auto‑update off. See [`docs/RELEASING.md`](docs/RELEASING.md) for how releases are cut.

## 🔒 Privacy

- 📍 Your weather **ZIP code is stored only in your local config** (`~/.config/momPanel/` on Linux, `%APPDATA%\momPanel\` on Windows). It's never committed, bundled, or shared — it's sent only to the weather service to fetch your forecast.
- 🧾 No accounts, no tracking, no telemetry.

## 🏗️ How it's built

- **[Tauri](https://tauri.app)** — a tiny Rust shell around a system webview (low RAM for an always‑on panel).
- **Rust backend** — small, independent *collector* modules (one per concern) that return a uniform data shape; the frontend never sees OS differences.
- **Vanilla HTML/CSS/JS frontend** — no framework, no bundler.

```
src/            🎨 frontend (tiles, layout, styles)
src-tauri/      🦀 Rust backend (collectors, config, commands)
docs/           📚 design specs, plans, and review notes
```

## 📜 License

A personal project, shared openly. 💛

---

<div align="center">
Made with care for Mom. 👵💙
</div>

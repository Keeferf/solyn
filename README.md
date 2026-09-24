<div align="center">
  <img src="src-tauri/icons/icon.png" width="112" alt="Solyn" />
  <h1>Solyn</h1>
  <p>A private, fully offline AI chat app for your desktop. Browse models on HuggingFace, run them locally with Ollama. No cloud, no accounts, no data leaving your machine.</p>

  <p>
    <a href="https://github.com/Keeferf/Solyn/actions/workflows/ci.yml"><img src="https://github.com/Keeferf/Solyn/actions/workflows/ci.yml/badge.svg" alt="CI" /></a>
    <img src="https://img.shields.io/badge/Tauri-2.x-24C8D8?style=flat-square&logo=tauri&logoColor=white" alt="Tauri" />
    <img src="https://img.shields.io/badge/React-19-61DAFB?style=flat-square&logo=react&logoColor=white" alt="React" />
    <img src="https://img.shields.io/badge/Rust-stable-CE422B?style=flat-square&logo=rust&logoColor=white" alt="Rust" />
    <img src="https://img.shields.io/badge/platform-Windows%20%7C%20Linux-20a39e?style=flat-square" alt="Platforms" />
  </p>

  <a href="https://github.com/Keeferf/Solyn/releases/latest">
    <img src="https://img.shields.io/badge/⬇ Download-Latest%20Release-20a39e?style=for-the-badge" alt="Download" />
  </a>
</div>

---

## Table of Contents

- [Overview](#overview)
- [Features](#features)
- [Installation](#installation)
- [Usage](#usage)
- [How It Works](#how-it-works)
- [Tech Stack](#tech-stack)
- [Development](#development)
- [Project Structure](#project-structure)
- [Privacy & Security](#privacy--security)
- [Troubleshooting](#troubleshooting)
- [FAQ](#faq)
- [Contributing](#contributing)
- [License](#license)
- [Acknowledgments](#acknowledgments)

---

## Overview

Solyn is a native desktop AI chat application built with Tauri and React. It browses and downloads open-weight GGUF models directly from HuggingFace, imports them into [Ollama](https://ollama.ai), and runs them entirely on your machine. After a model is downloaded, no internet connection is required — chat sessions, models, and settings all live on your disk.

---

## Features

- **100% local inference** — models run through Ollama on your hardware; nothing is sent to a cloud API
- **Model browser** — search HuggingFace for GGUF models, or filter by most downloaded / most liked / recent
- **Chunked, resumable downloads** — fetch models with live progress, cancellation, and a size check when the download completes
- **Automatic Ollama import** — generates a Modelfile and registers the model with Ollama for you
- **Chat sessions** — create, rename, switch between, and delete conversations, persisted locally in SQLite
- **Agent and Chat modes** — toggle the assistant's behavior from the chat toolbar
- **Streaming responses** — tokens stream into the chat window in real time
- **Rich rendering** — Markdown with tables (GFM), syntax highlighting (Shiki), and math (KaTeX)
- **File attachments** — attach files to a message from the composer
- **Themes & layout** — light/dark themes and a collapsible sidebar
- **System-aware** — detects platform and hardware resources to guide model choices
- **Fully offline** — once a model is downloaded, no network access is needed

<!-- Screenshots welcome — drop PNGs into docs/ and uncomment this block.

## Screenshots

<p align="center">
  <img src="docs/screenshot-chat.png" width="45%" alt="Chat view" />
  <img src="docs/screenshot-models.png" width="45%" alt="Model browser" />
</p>

-->

---

## Installation

One-line installers for Windows and Linux. Each script downloads the latest release from GitHub and installs it for you — no Node.js, Rust, or build tools required, because these are prebuilt binaries. Solyn then installs Ollama on first launch.

### Windows

Open **PowerShell** and run:

```powershell
irm https://raw.githubusercontent.com/Keeferf/Solyn/main/install.ps1 | iex
```

- Requires **Windows 10 or 11 (x64)**.
- Installs the `.msi` package (falling back to the NSIS `.exe` if no MSI is available).
- **Uninstall:** Settings → Apps → Installed apps → **Solyn** → Uninstall.

### Linux

Open a terminal and run:

```bash
curl -fsSL https://raw.githubusercontent.com/Keeferf/Solyn/main/install.sh | sh
```

- Requires a **64-bit** system with WebKitGTK 4.1 (installed automatically with the `.deb`).
- On Debian/Ubuntu it installs the `.deb` via `dpkg`; otherwise it installs the AppImage to `~/.local/bin/solyn`.
- **Uninstall:** `sudo apt remove solyn` for the `.deb`, or `rm ~/.local/bin/solyn` for the AppImage.

> Prefer to inspect before running? Both scripts are short and live at the root of this repo: [`install.sh`](install.sh) · [`install.ps1`](install.ps1).

---

## Usage

1. **Launch Solyn.** On first run it offers to install Ollama — accept, or install Ollama yourself beforehand.
2. **Get a model.** Open **Models**, search HuggingFace for a GGUF model, choose a quantization, and download it. Solyn imports it into Ollama when the download finishes.
3. **Chat.** Pick the model in the composer and start typing. Responses stream in as they are generated.
4. **Manage history.** The **History** view lists your sessions — reopen, rename, or delete them there.

Everything runs locally. Once a model is downloaded, you can go fully offline.

---

## How It Works

1. Browse or search HuggingFace for a GGUF model and pick a quantization.
2. Solyn downloads the file in parallel chunks into the local model cache.
3. On completion it generates an Ollama Modelfile and imports the model into Ollama.
4. Chat requests stream from Ollama through the Rust backend to the React UI, which renders Markdown, code, and math as they arrive.

---

## Tech Stack

| Layer           | Technology                                                                                                |
| --------------- | --------------------------------------------------------------------------------------------------------- |
| Desktop shell   | [Tauri 2](https://tauri.app) (tray icon, native title bar)                                                |
| Frontend        | [React 19](https://react.dev) + [Vite](https://vitejs.dev)                                                |
| Language        | [TypeScript](https://www.typescriptlang.org/) (strict) + Rust (stable)                                    |
| Styling         | [Tailwind CSS v4](https://tailwindcss.com)                                                                |
| State           | [Zustand](https://zustand-demo.pmnd.rs/)                                                                  |
| Persistence     | [SQLite](https://sqlite.org) via [`rusqlite`](https://github.com/rusqlite/rusqlite) (bundled)             |
| Inference       | [Ollama](https://ollama.ai) (local subprocess)                                                            |
| Model downloads | [reqwest](https://github.com/seanmonstar/reqwest) (streaming, chunked transfer)                           |
| Markdown        | [`react-markdown`](https://github.com/remarkjs/react-markdown) + remark-gfm / remark-math + KaTeX + Shiki |
| Tauri plugins   | [`tauri-plugin-shell`](https://github.com/tauri-apps/plugins-workspace), [`tauri-plugin-opener`](https://github.com/tauri-apps/plugins-workspace) |

---

## Development

### Prerequisites

Install these for both platforms:

- **Node.js** 22+ and **pnpm** 12+ (`corepack enable` then `corepack prepare pnpm@12 --activate`)
- **Rust** stable via [rustup](https://rustup.rs)

Then, per platform:

**Linux** — WebKitGTK and the Tauri system libraries:

```bash
sudo apt-get update
sudo apt-get install -y \
  libwebkit2gtk-4.1-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  libxdo-dev \
  libssl-dev \
  build-essential \
  patchelf
```

(Debian/Ubuntu names — adapt the equivalents for Fedora/Arch.)

**Windows** — [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) ("Desktop development with C++") and the [WebView2 runtime](https://developer.microsoft.com/microsoft-edge/webview2/) (preinstalled on Windows 11).

### Setup

```bash
git clone https://github.com/Keeferf/solyn.git
cd solyn
pnpm install
```

### Run in development

```bash
pnpm tauri dev
```

This starts the Vite dev server on port 1420 and launches the Tauri window with hot reload.

### Build a release

```bash
pnpm tauri build
```

Installers and bundles are written to `src-tauri/target/release/bundle/`.

### Frontend-only scripts

| Command        | Description                                    |
| -------------- | ---------------------------------------------- |
| `pnpm dev`     | Vite dev server (frontend only, port 1420)     |
| `pnpm build`   | Type-check and build the frontend into `dist/` |
| `pnpm preview` | Preview the built frontend                     |

---

## Project Structure

```
solyn/
├── src/                    # React frontend
│   ├── components/
│   │   ├── chat/           # Chat UI, composer, Markdown/code rendering
│   │   ├── history/        # Session list and previews
│   │   ├── models/         # HuggingFace browser and model management
│   │   ├── sidebar/        # Navigation, theme switcher
│   │   └── ui/             # Title bar, modals
│   ├── contexts/           # Ollama status/provider
│   ├── stores/             # Zustand stores (chat, theme)
│   └── styles/             # Global, markdown, and animation CSS
├── src-tauri/              # Rust backend
│   ├── src/
│   │   ├── api/            # Tauri commands & queries (ollama, models, chat, platform)
│   │   ├── core/           # Ollama client, HuggingFace downloader, platform detection, Ollama installer
│   │   ├── data/           # Chat database, model types, download state
│   │   ├── events/         # Progress broadcasting, Ollama status monitor
│   │   └── helpers/        # Platform detection, terminal output cleanup
│   ├── migrations/         # SQLite schema
│   └── tauri.conf.json     # Tauri configuration
├── install.sh              # Linux installer
└── install.ps1             # Windows installer
```

---

## Privacy & Security

Solyn makes no telemetry calls and requires no account. All data stays local:

- **Chat history** — SQLite database in the app data directory (`com.solyn.desktop`), e.g. `~/.local/share/com.solyn.desktop/` on Linux or `%APPDATA%\com.solyn.desktop\` on Windows.
- **Models** — stored in Ollama's own model directory (`~/.ollama` on Linux, `%USERPROFILE%\.ollama` on Windows).
- **Network** — the only outbound traffic is to the HuggingFace API/model host (browsing and downloading) and to Ollama over localhost.

---

## Troubleshooting

| Symptom                                                        | Fix                                                                             |
| -------------------------------------------------------------- | ------------------------------------------------------------------------------- |
| App won't launch on Linux (missing `webkit2gtk` / `appindicator`) | Install the [Linux prerequisites](#prerequisites), or use the `.deb`, which pulls them in. |
| Build fails on Linux with missing dev headers                   | Install the [Linux prerequisites](#prerequisites) above.                        |
| `cargo build` fails on Windows with linker errors               | Install the C++ Build Tools and reopen your terminal.                           |
| Blank window on Windows                                         | Install the WebView2 runtime.                                                   |
| "Ollama not detected"                                           | Let Solyn install it from the prompt, or install Ollama manually and relaunch.  |
| Port 1420 already in use                                        | Vite runs with `strictPort`; stop the other process or free the port.           |
| First Tauri build is very slow                                  | Expected — the Rust dependency graph compiles once, then caches.                |

---

## FAQ

**Do I need Ollama installed first?**
No. Solyn installs it for you on first launch. If you already have it, Solyn uses your existing installation.

**Is any of my data sent anywhere?**
No. See [Privacy & Security](#privacy--security).

**Which models should I pick?**
Any GGUF model on HuggingFace works. Smaller quantizations (e.g. `Q4`) use less memory; larger ones are more accurate. Solyn shows the quantization for each file.

**How much disk space do models take?**
It varies by size and quantization — from a couple of GB for small models to tens of GB for large ones.

**Does it run on macOS?**
Not currently. Solyn targets Windows and Linux.

---

## Contributing

Pull requests are welcome. Before opening one:

```bash
pnpm install
pnpm tauri dev     # manual smoke test
cargo fmt          # in src-tauri/
cargo clippy       # in src-tauri/
```

CI runs the frontend build plus `cargo build` on `ubuntu-latest` and `windows-latest`. For bug reports and feature requests, [open an issue](https://github.com/Keeferf/Solyn/issues).

---

## License

_Not yet specified — add a `LICENSE` file and update this section._

---

## Acknowledgments

Built on [Tauri](https://tauri.app), [Ollama](https://ollama.ai), and [HuggingFace](https://huggingface.co). UI rendering by [React](https://react.dev), [Tailwind CSS](https://tailwindcss.com), [Shiki](https://shiki.style), and [KaTeX](https://katex.org).

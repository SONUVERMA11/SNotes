# S Notes

> **Linux-native handwriting & annotation app** — modeled after GoodNotes 6

<p align="center">
  <strong>Created by <a href="https://github.com/SONUVERMA11">Sonu Verma</a></strong><br>
  <em>Pen · Brush · Pencil · Marker · Highlighter · Shapes · PDF annotation · Infinite canvas</em>
</p>

<p align="center">
  <a href="https://github.com/SONUVERMA11/SNotes"><img src="https://img.shields.io/badge/GitHub-SONUVERMA11%2FSNotes-blue?logo=github" alt="GitHub"></a>
  <a href="https://github.com/SONUVERMA11/SNotes/actions"><img src="https://img.shields.io/github/actions/workflow/status/SONUVERMA11/SNotes/ci.yml?label=CI" alt="CI"></a>
  <img src="https://img.shields.io/badge/language-Rust-orange?logo=rust" alt="Rust">
  <img src="https://img.shields.io/badge/license-GPL--3.0-green" alt="License">
  <img src="https://img.shields.io/badge/tests-100%2B%20passing-brightgreen" alt="Tests">
</p>

---

## ✨ Features

### 🖊️ Ink Engine
- **Bézier cubic spline** fitting from raw input for smooth strokes
- **Variable stroke width** based on pressure + velocity
- **Predictive ink** — 2-frame quadratic extrapolation for low latency
- **6 tool types**: Pen, Brush, Pencil, Marker, Highlighter, Eraser
- **Eraser modes**: whole-stroke delete & pixel-level split
- **Stroke geometry generator** — variable-width outline meshes with round caps
- **Text annotations** with font control, alignment, and SVG export

### 🎨 Canvas System
- **Infinite canvas** with pan/zoom (mouse, stylus, touch)
- **Multi-layer support** — add, reorder, lock, hide, opacity
- **7 page templates**: Blank, Lined, Grid, Dotted, Isometric, Music Staff, Cornell
- **Grid snapping** and alignment guides
- **Lasso selection** — move, scale, rotate, copy/paste strokes
- **Shape recognition** — circles, rectangles, triangles, arrows, lines
- **HSV color picker** with multiple palettes (default, pastel, monochrome)

### ✏️ Input Engine
- **libinput** integration for universal tablet/stylus support
- **Wacom, Huion, XP-Pen, Gaomon** tablet detection
- **Pressure normalization** (gamma curves, thresholds)
- **Palm rejection** with configurable exclusion zones
- **Barrel button** configurable actions (eraser, undo, color picker, etc.)
- **Hover detection** with cursor preview

### 📦 Document Model
- **Library → Notebooks → Sections → Pages** hierarchy
- **SQLite metadata** database with full CRUD operations
- **LZ4 compression** for high-frequency stroke data
- **Auto-save** with dirty tracking, backup rotation, and crash recovery
- **Full-text search** across notebooks, tags, text annotations, and OCR text

### 📄 Import & Export
- **PDF import** with page-by-page annotation overlay
- **PDF export** — strokes as embedded vector paths
- **PNG export** — rasterized at configurable DPI
- **SVG export** — smooth Bézier path data
- **Native `.snotes`** format

### ☁️ Sync
- **WebDAV** sync with configurable conflict strategies
- **Nextcloud** integration (WebDAV + OCS API)
- **D-Bus IPC** between GTK app and sync daemon
- **Conflict resolution** — 5 strategies (KeepLocal, KeepRemote, KeepNewest, KeepBoth, AskUser)
- **OCR pipeline** — Tesseract 5 handwriting → searchable text

### 🎨 UI (GTK4 + libadwaita)
- **Dark / Light / Sepia / Custom** themes with CSS generation
- **Toolbar** with tool selection, color picker, width slider
- **Sidebar** with notebook navigation and search
- **Preferences window** (Appearance, Canvas, Input, Sync)
- **Keyboard shortcuts** — fully configurable with defaults
- **Rulers & Protractor** tools with edge snapping

### 🔌 Plugin API
- **WASM sandboxed plugins** via Wasmtime
- **Capability-based** security (ReadStrokes, WriteStrokes, ToolbarExtension, etc.)
- **Plugin manifest** system with discovery
- **Event system** — StrokeAdded, PageChanged, ToolChanged, etc.
- **Canvas overlay API** for custom rendering

---

## 🏗️ Architecture

```
SNotes/
├── crates/
│   ├── snotes-core/          # Core engine (ink, canvas, storage, export, search)
│   ├── snotes-gtk/           # GTK4/libadwaita frontend
│   ├── snotes-sync/          # Sync daemon (WebDAV, Nextcloud, OCR, D-Bus)
│   ├── snotes-cli/           # CLI tool for batch export
│   └── snotes-plugin/        # WASM plugin sandbox
├── packaging/                # .deb, .rpm, AUR packaging scripts
├── flatpak/                  # Flatpak manifest
├── data/                     # Desktop entry, AppStream metadata
├── schemas/                  # FlatBuffers schema
└── .github/workflows/        # CI/CD pipeline
```

---

## 🚀 Getting Started

### Prerequisites

```bash
# Ubuntu/Debian
sudo apt install \
  libgtk-4-dev \
  libadwaita-1-dev \
  libinput-dev \
  libsqlite3-dev \
  pkg-config \
  clang

# Fedora
sudo dnf install \
  gtk4-devel \
  libadwaita-devel \
  libinput-devel \
  sqlite-devel \
  clang

# Optional: OCR support
sudo apt install tesseract-ocr tesseract-ocr-eng
```

### Build

```bash
# Clone
git clone https://github.com/SONUVERMA11/SNotes.git
cd SNotes

# Build (debug)
cargo build --workspace

# Build (release)
cargo build --release -p snotes-gtk -p snotes-cli -p snotes-sync

# Run tests
cargo test --workspace --exclude snotes-gtk

# Run the app
cargo run -p snotes-gtk

# Run the sync daemon
cargo run -p snotes-sync

# CLI export
cargo run -p snotes-cli -- export --format pdf --input notebook.snotes --output ./out/
```

### Install (.deb)

```bash
cd packaging
./build-deb.sh 0.1.0
sudo dpkg -i snotes_0.1.0_amd64.deb
```

### Flatpak

```bash
flatpak-builder --force-clean build-dir flatpak/org.snotes.App.json
flatpak-builder --run build-dir flatpak/org.snotes.App.json snotes-gtk
```

---

## ⌨️ Default Keyboard Shortcuts

| Action | Shortcut |
|--------|----------|
| Undo | `Ctrl+Z` |
| Redo | `Ctrl+Shift+Z` |
| Pen tool | `P` |
| Brush tool | `B` |
| Eraser tool | `E` |
| Highlighter | `H` |
| Select tool | `S` |
| Shape tool | `R` |
| Text tool | `T` |
| Zoom in | `Ctrl+=` |
| Zoom out | `Ctrl+-` |
| Fit page | `Ctrl+0` |
| Toggle grid | `Ctrl+G` |
| Toggle sidebar | `F9` |
| Fullscreen | `F11` |
| Export | `Ctrl+Shift+E` |
| Preferences | `Ctrl+,` |

---

## 🔌 Plugin Development

Create a plugin directory with:

```
my-plugin/
├── manifest.json
└── plugin.wasm
```

**manifest.json:**
```json
{
  "name": "my-plugin",
  "version": "1.0.0",
  "author": "You",
  "description": "My custom plugin",
  "capabilities": ["ReadStrokes", "ToolbarExtension"],
  "module": "plugin.wasm"
}
```

Plugins are compiled to WASM and run in a sandboxed Wasmtime environment. See `crates/snotes-plugin/src/api.rs` for the full API surface.

---

## ⚡ Performance

| Operation | Throughput |
|-----------|-----------|
| Bézier spline fitting (100pts) | 1.4M ops/sec |
| Stroke geometry generation | 40K ops/sec |
| Hit testing (1000 strokes) | 468K ops/sec |
| Predictive ink | 9.6M pts/sec |
| LZ4 compression (100KB) | 112µs |

---

## 📊 Project Stats

| Metric | Value |
|--------|-------|
| Language | Rust |
| Source files | 50+ |
| Lines of code | ~9,000 |
| Test cases | 100+ |
| Crates | 5 |

---

## 👤 Author

**Sonu Verma**
- GitHub: [@SONUVERMA11](https://github.com/SONUVERMA11)
- Project: [SNotes](https://github.com/SONUVERMA11/SNotes)

---

## 📜 License

This project is licensed under the [GPL-3.0-or-later](LICENSE) license.

Copyright © 2026 [Sonu Verma](https://github.com/SONUVERMA11)


---
Made with ❤️ by [Sonu Verma](https://github.com/SONUVERMA11)


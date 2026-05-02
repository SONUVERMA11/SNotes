# S Notes — User Guide

> **Version 0.1.0** · By [Sonu Verma](https://github.com/SONUVERMA11)

---

## Table of Contents

1. [Installation](#installation)
2. [First Launch](#first-launch)
3. [The Interface](#the-interface)
4. [Drawing & Writing](#drawing--writing)
5. [Tools](#tools)
6. [Pages & Templates](#pages--templates)
7. [Notebooks & Organization](#notebooks--organization)
8. [Layers](#layers)
9. [Selection & Editing](#selection--editing)
10. [Shapes & Recognition](#shapes--recognition)
11. [Text Annotations](#text-annotations)
12. [Color Picker](#color-picker)
13. [PDF Import & Annotation](#pdf-import--annotation)
14. [Export](#export)
15. [Cloud Sync (Nextcloud)](#cloud-sync-nextcloud)
16. [OCR — Handwriting Recognition](#ocr--handwriting-recognition)
17. [Search](#search)
18. [Auto-Save & Recovery](#auto-save--recovery)
19. [Keyboard Shortcuts](#keyboard-shortcuts)
20. [Stylus & Tablet Setup](#stylus--tablet-setup)
21. [Themes & Appearance](#themes--appearance)
22. [Plugins](#plugins)
23. [CLI Tool](#cli-tool)
24. [Troubleshooting](#troubleshooting)
25. [Building from Source](#building-from-source)

---

## Installation

### Flatpak (Recommended)

```bash
# From Flathub (once published)
flatpak install flathub org.snotes.App

# Or build locally
flatpak-builder --force-clean build-dir flatpak/org.snotes.App.json
flatpak-builder --install --user build-dir flatpak/org.snotes.App.json
```

### Debian / Ubuntu (.deb)

```bash
cd packaging
./build-deb.sh 0.1.0
sudo apt install ./snotes_0.1.0_amd64.deb
```

### Fedora / RHEL (.rpm)

```bash
rpmbuild -ba packaging/snotes.spec
sudo dnf install ~/rpmbuild/RPMS/x86_64/snotes-0.1.0-1.*.rpm
```

### Arch Linux (AUR)

```bash
cd packaging
makepkg -si
```

### From Source

```bash
git clone https://github.com/SONUVERMA11/SNotes.git
cd SNotes
cargo build --release -p snotes-gtk -p snotes-cli -p snotes-sync
sudo install -m 755 target/release/snotes-gtk /usr/local/bin/
```

---

## First Launch

When you first open S Notes, you'll see the main window with:

- A **sidebar** on the left showing your Library
- A **toolbar** at the top with drawing tools
- A **blank canvas** in the center, ready for writing

Your first notebook called "My Notes" is created automatically with a blank page.

**Quick start:**
1. Pick the **Pen tool** from the toolbar (or press `P`)
2. Choose a color and stroke width
3. Start writing on the canvas with your mouse or stylus!

---

## The Interface

### Toolbar (Top)
| Icon | Tool | Shortcut |
|------|------|----------|
| ✒️ | Pen | `P` |
| 🖌️ | Brush | `B` |
| ✏️ | Pencil | `D` |
| 🖍️ | Marker | `M` |
| 🔆 | Highlighter | `H` |
| ⬜ | Eraser | `E` |
| ◻️ | Select | `S` |
| ⬡ | Shape | `R` |
| 🔤 | Text | `T` |
| 📏 | Ruler | `U` |

### Sidebar (Left)
- **Library**: All your notebooks organized hierarchically
- **Sections**: Tabs within each notebook
- **Pages**: Thumbnails of each page in a section
- **Search**: Full-text search bar at the top

### Status Bar (Bottom)
- Current page number
- Zoom level
- Auto-save indicator
- Sync status

---

## Drawing & Writing

### Using a Stylus
S Notes is designed for stylus input. It supports:
- **Pressure sensitivity**: Press harder for thicker lines
- **Tilt detection**: Tilt your pen for shading effects (Brush tool)
- **Barrel buttons**: Configurable — defaults to eraser (button 1) and undo (button 2)
- **Hover detection**: See the cursor before touching down
- **Palm rejection**: Rest your hand on the screen without false marks

### Using a Mouse
Mouse input works with simulated pressure (0.5 constant). You can:
- **Left click + drag**: Draw
- **Middle click + drag**: Pan the canvas
- **Scroll wheel**: Zoom in/out
- **Right click**: Context menu

### Predictive Ink
S Notes uses 2-frame quadratic prediction to reduce perceived latency. The predicted stroke appears slightly ahead of your actual pen position and is corrected when the real input arrives. This makes writing feel more responsive.

---

## Tools

### Pen (P)
The default writing tool. Produces smooth, variable-width strokes that respond to pressure and velocity. Best for everyday note-taking.

### Brush (B)
A wider, softer tool that responds dramatically to pressure. Great for artistic work and calligraphy. Tilt affects the stroke angle.

### Pencil (D)
A textured tool with subtle grain. Pressure affects opacity more than width. Ideal for sketching.

### Marker (M)
Constant-width strokes regardless of pressure. Perfect for diagrams and technical drawing.

### Highlighter (H)
Semi-transparent, wide strokes. Uses pastel colors by default. Rendered behind existing ink (multiply blend). Great for emphasizing text in imported PDFs.

### Eraser (E)
Two eraser modes:
- **Whole-stroke** (default): Touching any part of a stroke deletes the entire stroke
- **Pixel-level**: Splits strokes where the eraser touches, allowing partial deletion

Toggle mode in the tool options panel.

---

## Pages & Templates

### Available Templates
| Template | Description |
|----------|-------------|
| **Blank** | Clean white page — infinite canvas |
| **Lined** | Horizontal ruled lines (college rule) |
| **Grid** | Square grid (5mm) for math/diagrams |
| **Dotted** | Dot grid — less visual clutter than full grid |
| **Isometric** | 30° isometric grid for 3D sketching |
| **Music Staff** | Standard 5-line music staves |
| **Cornell** | Cornell note-taking layout with cue/summary areas |

### Page Operations
- **Add page**: Click `+` at the bottom of the page list
- **Delete page**: Right-click → Delete
- **Reorder pages**: Drag and drop in the sidebar
- **Change template**: Right-click → Change Template
- **Resize page**: Settings → Canvas → Page size

---

## Notebooks & Organization

### Hierarchy
```
Library
  └── Notebook (has a cover color)
        └── Section (like tabs)
              └── Pages (where you write)
```

### Creating Notebooks
1. Click **+** in the sidebar
2. Enter a name (e.g., "Physics 101")
3. Choose a cover color
4. A default section with a blank page is created

### Sections
Sections act as chapter dividers within a notebook:
- Right-click a notebook → **Add Section**
- Name your section (e.g., "Mechanics", "Thermodynamics")
- Drag sections to reorder

---

## Layers

Each page supports multiple layers, similar to image editors:

| Action | How |
|--------|-----|
| **Add layer** | Layers panel → `+` button |
| **Rename** | Double-click the layer name |
| **Reorder** | Drag and drop in the layers panel |
| **Hide/Show** | Click the eye icon |
| **Lock** | Click the lock icon (prevents edits) |
| **Opacity** | Slider next to each layer |

**Tips:**
- Use separate layers for handwriting vs. diagrams
- Lock your reference images on a bottom layer
- Lower opacity for tracing

---

## Selection & Editing

### Lasso Select (S)
1. Switch to Select tool (`S`)
2. Draw a lasso around the strokes you want
3. A selection box appears with handles

### Selection Actions
| Action | How |
|--------|-----|
| **Move** | Drag the selection |
| **Scale** | Drag corner handles |
| **Rotate** | Drag the rotation handle (top) |
| **Copy** | `Ctrl+C` |
| **Paste** | `Ctrl+V` (pastes with offset) |
| **Cut** | `Ctrl+X` |
| **Delete** | `Delete` or `Backspace` |
| **Duplicate** | `Ctrl+D` (instant copy with offset) |

---

## Shapes & Recognition

### Drawing Shapes
1. Switch to Shape tool (`R`)
2. Draw a rough shape on the canvas
3. S Notes recognizes it and snaps to a perfect version

### Recognized Shapes
| Shape | How to Draw |
|-------|-------------|
| **Circle** | Draw a rough circle |
| **Rectangle** | Draw a rough rectangle |
| **Triangle** | Draw a rough triangle |
| **Arrow** | Draw a line with a flick at the end |
| **Straight Line** | Draw a mostly-straight line |

### Grid Snapping
Enable grid snapping (`Ctrl+G`) to align shapes to the grid. Shapes will snap to the nearest grid intersection.

---

## Text Annotations

### Adding Text
1. Switch to Text tool (`T`)
2. Click on the canvas where you want text
3. A text box appears — start typing
4. Click outside to finish

### Text Formatting
| Property | Options |
|----------|---------|
| **Font** | Sans, Serif, Monospace, and system fonts |
| **Size** | 8px – 72px |
| **Weight** | Light, Regular, Medium, SemiBold, Bold, ExtraBold |
| **Style** | Normal, Italic, Oblique |
| **Alignment** | Left, Center, Right, Justify |
| **Color** | Any color from the color picker |
| **Opacity** | 0% – 100% |

### Editing Text
- Double-click an existing text box to edit
- `Ctrl+A` to select all text
- Standard text editing keys (arrows, Home/End, etc.)

---

## Color Picker

### Quick Colors
The toolbar shows your most recent colors. Click any to select it.

### Full Color Picker
Click the color swatch in the toolbar to open the full picker:

1. **Color Wheel**: Drag around the ring to set hue (0°–360°)
2. **SV Square**: Drag within the square to set saturation and brightness
3. **Alpha Slider**: Set transparency (0%–100%)
4. **Hex Input**: Enter exact colors like `#FF5733`

### Palettes
| Palette | Colors |
|---------|--------|
| **Default** | 16 curated colors (black, gray, red, orange, yellow, green, blue, etc.) |
| **Pastel** | 8 semi-transparent pastel colors for highlighting |
| **Monochrome** | 11-step grayscale ramp |

### Favorites
Star any color to add it to your favorites palette for quick access.

---

## PDF Import & Annotation

### Importing a PDF
1. **File → Import PDF** (or `Ctrl+I`)
2. Select a PDF file
3. Each PDF page becomes a page in your notebook with the PDF as a background
4. Draw on top of the PDF pages to annotate

### Annotation Tips
- Use the **Highlighter** tool to emphasize text
- Use **Text annotations** for typed comments
- Each annotation goes on a separate layer above the PDF
- Export back to PDF to share annotated documents

---

## Export

### Export Formats
| Format | Description |
|--------|-------------|
| **PDF** | Vector paths — strokes rendered as Bézier curves in the PDF |
| **PNG** | Rasterized at configurable DPI (default: 300) |
| **SVG** | Scalable vector graphics with smooth paths |
| **.snotes** | Native format — preserves layers, history, metadata |

### How to Export
1. **File → Export** (or `Ctrl+Shift+E`)
2. Choose format and destination
3. Select pages (current, section, or entire notebook)
4. Click Export

### CLI Export (Batch)
```bash
snotes-cli export --format pdf --input notebook.snotes --output ./out/
snotes-cli export --format png --dpi 600 --input notebook.snotes --output ./out/
```

---

## Cloud Sync (Nextcloud)

### Setup
1. Open **Preferences → Sync**
2. Enter your Nextcloud server URL (e.g., `https://cloud.example.com`)
3. Enter your username and password
4. Choose a remote folder (default: `/SNotes/`)
5. Click **Test Connection**
6. Enable sync

### Sync Behavior
- **Auto-sync** runs every 5 minutes (configurable)
- **Manual sync**: Click the sync button in the status bar
- Files are synced as `.snotes` bundles via WebDAV

### Conflict Resolution
When the same notebook is modified on multiple devices:

| Strategy | Behavior |
|----------|----------|
| **Keep Local** | Your local version wins |
| **Keep Remote** | The server version wins |
| **Keep Newest** | Whichever was modified most recently wins |
| **Keep Both** | Both versions are kept with a conflict suffix |
| **Ask User** | A dialog asks you which to keep |

Set your default strategy in **Preferences → Sync → Conflict Resolution**.

---

## OCR — Handwriting Recognition

### Requirements
```bash
# Install Tesseract
sudo apt install tesseract-ocr tesseract-ocr-eng

# Additional languages
sudo apt install tesseract-ocr-deu tesseract-ocr-fra
```

### Using OCR
1. Write or draw on a page
2. **Tools → Recognize Text** (or `Ctrl+Shift+O`)
3. S Notes rasterizes your strokes and runs Tesseract
4. Recognized text becomes searchable — find handwritten notes via Search!

### OCR Settings
- **Language**: English (default), German, French, etc.
- **Page segmentation**: Auto, single block, single line
- **Handwriting mode**: Optimized for handwritten text (slower but more accurate)

---

## Search

### Quick Search
Press `Ctrl+F` to open the search bar. Start typing to search across:
- Notebook titles
- Section titles
- Text annotations
- OCR-recognized handwriting

### Search Filters
- **Titles only**: Search notebook/section names
- **Text only**: Search typed annotations
- **OCR only**: Search recognized handwriting
- **Specific notebook**: Filter results to one notebook

Results are ranked by relevance with snippets showing where the match was found.

---

## Auto-Save & Recovery

### Auto-Save
S Notes automatically saves your work:
- **Every 30 seconds** (configurable in Preferences)
- **On app switch** (when you alt-tab away)
- A subtle indicator in the status bar shows save status

### Backup & Recovery
- S Notes keeps up to **10 backup versions** of each notebook
- Backups are stored in `~/.local/share/snotes/backups/`
- If the app crashes, you'll see a **recovery prompt** on next launch
- Choose to restore from the backup or start fresh

### Changing Auto-Save Settings
**Preferences → General → Auto-Save:**
- Enable/disable auto-save
- Set the interval (5s – 300s)
- Set max backup count
- Change backup directory

---

## Keyboard Shortcuts

### Essential
| Action | Shortcut |
|--------|----------|
| Undo | `Ctrl+Z` |
| Redo | `Ctrl+Shift+Z` |
| Save | `Ctrl+S` |
| New page | `Ctrl+N` |
| Delete page | `Ctrl+Shift+Delete` |

### Tools
| Action | Shortcut |
|--------|----------|
| Pen | `P` |
| Brush | `B` |
| Pencil | `D` |
| Marker | `M` |
| Highlighter | `H` |
| Eraser | `E` |
| Select | `S` |
| Shape | `R` |
| Text | `T` |
| Ruler | `U` |

### Navigation
| Action | Shortcut |
|--------|----------|
| Zoom in | `Ctrl+=` |
| Zoom out | `Ctrl+-` |
| Fit page | `Ctrl+0` |
| Next page | `Page Down` |
| Previous page | `Page Up` |
| Toggle sidebar | `F9` |
| Fullscreen | `F11` |

### Editing
| Action | Shortcut |
|--------|----------|
| Copy | `Ctrl+C` |
| Paste | `Ctrl+V` |
| Cut | `Ctrl+X` |
| Select all | `Ctrl+A` |
| Duplicate | `Ctrl+D` |
| Delete | `Delete` |
| Toggle grid | `Ctrl+G` |
| Toggle snap | `Ctrl+Shift+G` |

### Other
| Action | Shortcut |
|--------|----------|
| Export | `Ctrl+Shift+E` |
| Import PDF | `Ctrl+I` |
| Preferences | `Ctrl+,` |
| Color picker | `C` |
| OCR | `Ctrl+Shift+O` |
| Search | `Ctrl+F` |

**All shortcuts are customizable** in **Preferences → Shortcuts**.

---

## Stylus & Tablet Setup

### Supported Tablets
S Notes supports all tablets recognized by the Linux kernel via libinput:
- **Wacom** (Intuos, Cintiq, Bamboo, One)
- **Huion** (Kamvas, Inspiroy)
- **XP-Pen** (Artist, Deco, Star)
- **Gaomon** (PD, M, S series)
- **Samsung S Pen** (via built-in digitizer)

### Pressure Calibration
**Preferences → Input → Pressure:**
- **Gamma curve**: Adjusts pressure response (1.0 = linear, <1.0 = softer, >1.0 = harder)
- **Min threshold**: Ignores very light touches
- **Max threshold**: Caps maximum pressure
- **Test area**: Draw to preview your pressure curve

### Barrel Buttons
**Preferences → Input → Buttons:**
| Button | Default Action | Options |
|--------|---------------|---------|
| Button 1 | Eraser | Eraser, Undo, Redo, Color Picker, Right Click, None |
| Button 2 | Undo | Same options |

### Palm Rejection
**Preferences → Input → Palm Rejection:**
- **Enable/disable**: Toggle palm rejection
- **Zone size**: How large an area to ignore (configurable per edge)
- **Position**: Bottom-left, bottom-right, or custom rectangles

---

## Themes & Appearance

### Built-in Themes
| Theme | Description |
|-------|-------------|
| **Dark** | Dark background, light ink — easy on the eyes |
| **Light** | Classic white background |
| **Sepia** | Warm, paper-like tone for comfortable reading |
| **Custom** | Define your own colors |

### Changing Theme
**Preferences → Appearance → Theme** or cycle with `Ctrl+T`.

### Custom Theme
1. Go to **Preferences → Appearance → Custom**
2. Set colors for: Background, Foreground, Accent, Sidebar, Toolbar
3. The CSS is generated automatically from your choices

---

## Plugins

### Installing Plugins
1. Download a `.wasm` plugin bundle
2. Place it in `~/.local/share/snotes/plugins/my-plugin/`
3. Ensure there's a `manifest.json` and `plugin.wasm`
4. Restart S Notes — the plugin loads automatically

### Plugin Capabilities
Plugins must declare what they can access:

| Capability | What it allows |
|------------|---------------|
| `ReadStrokes` | Read stroke data from the canvas |
| `WriteStrokes` | Create/modify strokes |
| `ToolbarExtension` | Add custom buttons to the toolbar |
| `PageAccess` | Navigate and create pages |
| `CanvasOverlay` | Draw custom overlays on the canvas |
| `Settings` | Read/write plugin-specific settings |

### Creating Plugins
See the [Plugin Development Guide](https://github.com/SONUVERMA11/SNotes#-plugin-development) in the README.

---

## CLI Tool

The `snotes-cli` tool enables batch operations from the terminal:

```bash
# Export notebook to PDF
snotes-cli export --format pdf --input my-notebook.snotes --output ./out/

# Export as high-res PNG
snotes-cli export --format png --dpi 600 --input notes.snotes --output ./images/

# Export as SVG
snotes-cli export --format svg --input notes.snotes --output ./svg/

# Show help
snotes-cli --help
```

---

## Troubleshooting

### "GTK4 not found" during build
```bash
# Install GTK4 development libraries
sudo apt install libgtk-4-dev libadwaita-1-dev
```

### Stylus not detected
1. Check `libinput list-devices` — your tablet should appear
2. If not listed, install the appropriate driver:
   ```bash
   # Huion/XP-Pen/Gaomon
   sudo apt install digimend-dkms
   ```
3. Try unplugging and re-plugging the tablet
4. Check permissions: `ls -la /dev/input/event*`

### High latency while drawing
- Enable predictive ink: **Preferences → Canvas → Predictive Ink**
- Lower the canvas resolution: **Preferences → Canvas → Render Quality**
- Ensure you're using the GPU backend (Skia, not Cairo)

### Sync not working
- Test connection: **Preferences → Sync → Test Connection**
- Check your Nextcloud URL includes `/remote.php/dav/files/USERNAME/`
- Ensure the sync daemon is running: `snotes-sync`

### Crash recovery
If S Notes crashes, on next launch it will:
1. Detect the stale lock file
2. Offer to restore from the latest backup
3. Recover your unsaved strokes

---

## Building from Source

### Prerequisites
```bash
# Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# System dependencies (Ubuntu/Debian)
sudo apt install \
  libgtk-4-dev \
  libadwaita-1-dev \
  libinput-dev \
  libsqlite3-dev \
  pkg-config \
  clang
```

### Build Commands
```bash
git clone https://github.com/SONUVERMA11/SNotes.git
cd SNotes

# Debug build
cargo build --workspace

# Release build (optimized)
cargo build --release -p snotes-gtk -p snotes-cli -p snotes-sync

# Run tests (104+ test cases)
cargo test --workspace --exclude snotes-gtk

# Run benchmarks
cargo test -p snotes-core --test benchmarks -- --nocapture
```

### Project Structure
```
SNotes/                         # Workspace root
├── Cargo.toml                  # Workspace manifest
├── crates/
│   ├── snotes-core/            # Core engine library
│   │   ├── src/
│   │   │   ├── ink/            # Stroke, Bézier, geometry, eraser, text, color
│   │   │   ├── canvas/         # Viewport, layers, templates
│   │   │   ├── document/       # Library/Notebook/Section/Page hierarchy
│   │   │   ├── input/          # libinput, pressure, palm rejection
│   │   │   ├── storage/        # SQLite, LZ4, auto-save, search
│   │   │   ├── export/         # PDF/PNG/SVG import & export
│   │   │   ├── tools/          # Selection, rulers, shapes
│   │   │   ├── shapes/         # Shape recognition
│   │   │   └── history/        # Undo/redo
│   │   └── tests/              # Integration tests & benchmarks
│   ├── snotes-gtk/             # GTK4/libadwaita frontend
│   ├── snotes-sync/            # Sync daemon (WebDAV, Nextcloud, OCR)
│   ├── snotes-cli/             # CLI export tool
│   └── snotes-plugin/          # WASM plugin sandbox
├── packaging/                  # .deb, .rpm, AUR scripts
├── flatpak/                    # Flatpak manifest
├── data/                       # Desktop entry, AppStream metadata
└── schemas/                    # FlatBuffers schema
```

---

## About

**S Notes** is created by **[Sonu Verma](https://github.com/SONUVERMA11)**.

- 🌐 GitHub: [github.com/SONUVERMA11/SNotes](https://github.com/SONUVERMA11/SNotes)
- 📜 License: [GPL-3.0-or-later](LICENSE)
- 🛠️ Built with: Rust, GTK4, libadwaita, libinput, SQLite, Wasmtime

---

*Thank you for using S Notes! If you find a bug or have a feature request, please open an issue on [GitHub](https://github.com/SONUVERMA11/SNotes/issues).*

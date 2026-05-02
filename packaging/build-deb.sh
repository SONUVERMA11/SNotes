#!/usr/bin/env bash
# Build .deb package for S Notes
# Usage: ./build-deb.sh [version]

set -euo pipefail

VERSION="${1:-0.1.0}"
ARCH="amd64"
PKG_NAME="snotes"
PKG_DIR="${PKG_NAME}_${VERSION}_${ARCH}"

echo "=== Building S Notes ${VERSION} .deb package ==="

# 1. Build release binaries
echo "  [1/5] Building release binaries..."
cargo build --release -p snotes-gtk -p snotes-cli -p snotes-sync

# 2. Create package directory structure
echo "  [2/5] Creating package structure..."
rm -rf "${PKG_DIR}"
mkdir -p "${PKG_DIR}/DEBIAN"
mkdir -p "${PKG_DIR}/usr/bin"
mkdir -p "${PKG_DIR}/usr/share/applications"
mkdir -p "${PKG_DIR}/usr/share/metainfo"
mkdir -p "${PKG_DIR}/usr/share/doc/${PKG_NAME}"

# 3. Copy binaries
echo "  [3/5] Installing binaries..."
install -m 755 target/release/snotes-gtk "${PKG_DIR}/usr/bin/"
install -m 755 target/release/snotes-cli "${PKG_DIR}/usr/bin/"
install -m 755 target/release/snotes-sync "${PKG_DIR}/usr/bin/"

# 4. Copy data files
install -m 644 data/org.snotes.App.desktop "${PKG_DIR}/usr/share/applications/"
install -m 644 data/org.snotes.App.metainfo.xml "${PKG_DIR}/usr/share/metainfo/"
install -m 644 README.md "${PKG_DIR}/usr/share/doc/${PKG_NAME}/"

# 5. Create control file
echo "  [4/5] Creating control file..."
INSTALLED_SIZE=$(du -sk "${PKG_DIR}" | cut -f1)

cat > "${PKG_DIR}/DEBIAN/control" << EOF
Package: ${PKG_NAME}
Version: ${VERSION}
Section: office
Priority: optional
Architecture: ${ARCH}
Installed-Size: ${INSTALLED_SIZE}
Depends: libgtk-4-1 (>= 4.12), libadwaita-1-0 (>= 1.4), libinput10, libsqlite3-0
Recommends: tesseract-ocr
Suggests: tesseract-ocr-eng
Maintainer: Sonu Verma <https://github.com/SONUVERMA11>
Homepage: https://github.com/SONUVERMA11/SNotes
Description: Linux-native handwriting & annotation app
 S Notes is a powerful handwriting and annotation application for Linux,
 designed for students, professionals, and creatives who use stylus tablets.
 .
 Features include Bézier-based ink rendering with pressure sensitivity,
 PDF import and annotation, multiple tool types (pen, brush, pencil,
 marker, highlighter), shape recognition, multi-layer support with
 page templates, and cloud sync via WebDAV/Nextcloud.
EOF

# 6. Build the .deb
echo "  [5/5] Building .deb package..."
dpkg-deb --build "${PKG_DIR}"

echo ""
echo "=== Done! Package: ${PKG_DIR}.deb ==="
echo "Install with: sudo dpkg -i ${PKG_DIR}.deb"
echo "Or:           sudo apt install ./${PKG_DIR}.deb"

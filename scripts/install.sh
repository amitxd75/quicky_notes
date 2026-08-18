#!/usr/bin/env bash
set -e

# Quicky Notes - Linux Desktop Installation Script
# Builds release binary, installs CLI launcher to ~/.local/bin,
# configures app icons across standard resolutions, and registers desktop MIME associations.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

echo "🚀 Building Quicky Notes release binary..."
cd "${ROOT_DIR}"
cargo build --release

BIN_SRC="${ROOT_DIR}/target/release/quicky_notes"
if [ ! -f "${BIN_SRC}" ]; then
    echo "❌ Error: Binary not found at ${BIN_SRC}"
    exit 1
fi

PREFIX_BIN="${HOME}/.local/bin"
DESKTOP_DIR="${HOME}/.local/share/applications"
ICON_BASE="${HOME}/.local/share/icons"
PIXMAP_DIR="${HOME}/.local/share/pixmaps"

mkdir -p "${PREFIX_BIN}" "${DESKTOP_DIR}" "${PIXMAP_DIR}" \
    "${ICON_BASE}/hicolor/512x512/apps" \
    "${ICON_BASE}/hicolor/256x256/apps" \
    "${ICON_BASE}/hicolor/128x128/apps" \
    "${ICON_BASE}/hicolor/scalable/apps"

echo "📦 Installing binary to ${PREFIX_BIN}/quicky..."
cp -f "${BIN_SRC}" "${PREFIX_BIN}/quicky"
chmod +x "${PREFIX_BIN}/quicky"

# Create quicky_notes alias symlink
ln -sf "${PREFIX_BIN}/quicky" "${PREFIX_BIN}/quicky_notes"

# Install application icons across standard directories
ICON_SRC="${ROOT_DIR}/assets/icon.png"
if [ -f "${ICON_SRC}" ]; then
    echo "🎨 Installing application icons..."
    cp -f "${ICON_SRC}" "${ICON_BASE}/hicolor/512x512/apps/quicky.png"
    cp -f "${ICON_SRC}" "${ICON_BASE}/hicolor/256x256/apps/quicky.png"
    cp -f "${ICON_SRC}" "${ICON_BASE}/hicolor/128x128/apps/quicky.png"
    cp -f "${ICON_SRC}" "${ICON_BASE}/hicolor/scalable/apps/quicky.png"
    cp -f "${ICON_SRC}" "${ICON_BASE}/quicky.png"
    cp -f "${ICON_SRC}" "${PIXMAP_DIR}/quicky.png"
fi

# Install .desktop launcher entry with absolute executable paths
DESKTOP_FILE="${DESKTOP_DIR}/quicky.desktop"
echo "📄 Installing desktop launcher at ${DESKTOP_FILE}..."

if [ -f "${ROOT_DIR}/assets/quicky.desktop" ]; then
    sed "s|^Exec=quicky|Exec=${PREFIX_BIN}/quicky|" "${ROOT_DIR}/assets/quicky.desktop" > "${DESKTOP_FILE}"
    sed -i "s|^TryExec=quicky|TryExec=${PREFIX_BIN}/quicky|" "${DESKTOP_FILE}"
    echo "Path=${HOME}" >> "${DESKTOP_FILE}"
else
    cat > "${DESKTOP_FILE}" << EOF
[Desktop Entry]
Type=Application
Name=Quicky Notes
GenericName=Note Taking & Scratchpad
Comment=Floating glassmorphism note widget and code scratchpad for Linux
TryExec=${PREFIX_BIN}/quicky
Exec=${PREFIX_BIN}/quicky %F
Icon=quicky
Terminal=false
StartupNotify=true
StartupWMClass=quicky_notes
Categories=Utility;TextEditor;
MimeType=text/plain;text/markdown;application/x-quickynote;inode/directory;
Keywords=notes;markdown;editor;quicky;scratchpad;
Path=${HOME}
EOF
fi

chmod +x "${DESKTOP_FILE}"

# Update desktop and MIME databases if tools are present
if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "${DESKTOP_DIR}" >/dev/null 2>&1 || true
fi

if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -f -t "${ICON_BASE}/hicolor" >/dev/null 2>&1 || true
fi

echo ""
echo "✨ Quicky Notes successfully installed!"
echo "   • Binary:  ${PREFIX_BIN}/quicky (and quicky_notes)"
echo "   • Desktop: ${DESKTOP_FILE}"
echo "   • Open with: Supported for text, markdown, and folders in file managers"
echo ""

# Check if ~/.local/bin is in PATH
if [[ ":$PATH:" != *":${HOME}/.local/bin:"* ]]; then
    echo "💡 Note: Add ~/.local/bin to your PATH in ~/.bashrc or ~/.zshrc:"
    echo "   export PATH=\"\$HOME/.local/bin:\$PATH\""
fi

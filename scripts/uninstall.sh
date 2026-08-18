#!/usr/bin/env bash
set -e

# Quicky Notes - Linux Desktop Uninstallation Script
# Removes local binaries, desktop entries, and icon assets installed by install.sh.

PREFIX_BIN="${HOME}/.local/bin"
DESKTOP_DIR="${HOME}/.local/share/applications"
ICON_BASE="${HOME}/.local/share/icons"
PIXMAP_DIR="${HOME}/.local/share/pixmaps"

echo "🗑️ Removing Quicky Notes binaries..."
rm -f "${PREFIX_BIN}/quicky"
rm -f "${PREFIX_BIN}/quicky_notes"

echo "🗑️ Removing desktop launcher entry..."
rm -f "${DESKTOP_DIR}/quicky.desktop"

echo "🗑️ Removing application icons..."
rm -f "${ICON_BASE}/hicolor/512x512/apps/quicky.png"
rm -f "${ICON_BASE}/hicolor/256x256/apps/quicky.png"
rm -f "${ICON_BASE}/hicolor/128x128/apps/quicky.png"
rm -f "${ICON_BASE}/hicolor/scalable/apps/quicky.png"
rm -f "${ICON_BASE}/quicky.png"
rm -f "${PIXMAP_DIR}/quicky.png"

# Update desktop & icon databases if available
if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "${DESKTOP_DIR}" >/dev/null 2>&1 || true
fi

if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -f -t "${ICON_BASE}/hicolor" >/dev/null 2>&1 || true
fi

echo ""
echo "✨ Quicky Notes successfully uninstalled from user system."
echo "💡 Note: Your notes, settings, and plugins are safely preserved in:"
echo "   ${HOME}/.config/quicky_notes/"
echo "   (To completely remove user data, run: rm -rf ~/.config/quicky_notes)"
echo ""

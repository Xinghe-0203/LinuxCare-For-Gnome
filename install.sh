#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

echo "Installing LinuxCare..."

# Install binaries
sudo install -Dm755 "$SCRIPT_DIR/linuxcare" /usr/local/bin/linuxcare
sudo install -Dm755 "$SCRIPT_DIR/linuxcare-cli" /usr/local/bin/linuxcare-cli

# Install desktop file
sudo install -Dm644 "$SCRIPT_DIR/data/linuxcare.desktop" /usr/share/applications/linuxcare.desktop

# Install icon
sudo install -Dm644 "$SCRIPT_DIR/icons/scalable/apps/linuxcare.svg" /usr/share/icons/hicolor/scalable/apps/linuxcare.svg
sudo gtk-update-icon-cache -f -t /usr/share/icons/hicolor 2>/dev/null || true
sudo update-desktop-database /usr/share/applications 2>/dev/null || true

echo "LinuxCare installed successfully!"
echo "  Binary:      /usr/local/bin/linuxcare"
echo "  CLI:         /usr/local/bin/linuxcare-cli"
echo "  Desktop:     /usr/share/applications/linuxcare.desktop"
echo "  Icon:        /usr/share/icons/hicolor/scalable/apps/linuxcare.svg"

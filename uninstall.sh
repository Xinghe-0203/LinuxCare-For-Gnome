#!/bin/bash
set -e

echo "Uninstalling LinuxCare..."

sudo rm -f /usr/local/bin/linuxcare
sudo rm -f /usr/local/bin/linuxcare-cli
sudo rm -f /usr/share/applications/linuxcare.desktop
sudo rm -f /usr/share/icons/hicolor/scalable/apps/linuxcare.svg
sudo gtk-update-icon-cache -f -t /usr/share/icons/hicolor 2>/dev/null || true
sudo update-desktop-database /usr/share/applications 2>/dev/null || true

echo "LinuxCare uninstalled successfully!"

#!/usr/bin/env bash
set -e

echo "🗑️ Uninstalling LogiOptions-Linux..."

PREFIX="${HOME}/.local"
BIN_DIR="${PREFIX}/bin"
APP_DIR="${PREFIX}/share/applications"
ICON_DIR="${PREFIX}/share/icons/hicolor/scalable/apps"
SYSTEMD_USER_DIR="${HOME}/.config/systemd/user"

# 1. Stop and disable systemd service
if command -v systemctl > /dev/null 2>&1; then
    echo "Stopping background daemon..."
    systemctl --user stop logioptions-daemon.service 2>/dev/null || true
    systemctl --user disable logioptions-daemon.service 2>/dev/null || true
fi

# 2. Kill any running daemon and GUI
pkill -x "logid-service" 2>/dev/null || true
pkill -f "window_launcher.py" 2>/dev/null || true

# 3. Remove files
echo "Removing binaries, desktop entry, autostart, and icons..."
rm -f "${BIN_DIR}/logid-service"
rm -f "${BIN_DIR}/logi-options"
rm -f "${APP_DIR}/logioptions.desktop"
rm -f "${HOME}/.config/autostart/logioptions.desktop"
rm -f "${ICON_DIR}/logioptions.svg"
rm -f "${SYSTEMD_USER_DIR}/logioptions-daemon.service"

if command -v systemctl > /dev/null 2>&1; then
    systemctl --user daemon-reload
fi

echo "✅ LogiOptions-Linux has been uninstalled."

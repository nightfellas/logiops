#!/usr/bin/env bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

echo "🐧 Installing LogiOptions-Linux..."

# 1. Build project in release mode
echo "📦 Compiling Rust binaries (Release mode)..."
cd "${REPO_ROOT}"
if [ -f "$HOME/.cargo/env" ]; then
    . "$HOME/.cargo/env"
fi
cargo build --release

# 2. Build Frontend UI
if [ -d "${REPO_ROOT}/crates/logi-gui/ui" ]; then
    echo "🎨 Building Frontend UI assets..."
    cd "${REPO_ROOT}/crates/logi-gui/ui"
    if command -v npm > /dev/null 2>&1; then
        npm install --silent
        npm run build
    fi
fi

cd "${REPO_ROOT}"

# 3. Determine installation targets (User or System)
PREFIX="${HOME}/.local"
BIN_DIR="${PREFIX}/bin"
APP_DIR="${PREFIX}/share/applications"
ICON_DIR="${PREFIX}/share/icons/hicolor/scalable/apps"
SHARE_DIR="${PREFIX}/share/logioptions"
SYSTEMD_USER_DIR="${HOME}/.config/systemd/user"

mkdir -p "${BIN_DIR}" "${APP_DIR}" "${ICON_DIR}" "${SHARE_DIR}" "${SYSTEMD_USER_DIR}"

# 4. Copy Binaries & GUI Assets
echo "🚀 Installing binaries to ${BIN_DIR}..."
install -m 755 "${REPO_ROOT}/target/release/logid-service" "${BIN_DIR}/logid-service"
install -m 755 "${REPO_ROOT}/target/release/logi-options" "${BIN_DIR}/logi-options"

echo "🎨 Installing GUI window runtime assets to ${SHARE_DIR}..."
install -m 755 "${REPO_ROOT}/crates/logi-gui/src-tauri/window_launcher.py" "${SHARE_DIR}/window_launcher.py"
rm -rf "${SHARE_DIR}/ui"
mkdir -p "${SHARE_DIR}/ui"
cp -r "${REPO_ROOT}/crates/logi-gui/ui/dist" "${SHARE_DIR}/ui/"

# 5. Copy Desktop Entry & Icon
echo "🖼️ Installing Desktop Shortcut and Icon..."
install -m 644 "${REPO_ROOT}/assets/icons/logioptions.svg" "${ICON_DIR}/logioptions.svg"

# Replace binary path in desktop file for user installation
sed "s|Exec=logi-options|Exec=${BIN_DIR}/logi-options|g" "${REPO_ROOT}/packaging/appimage/logioptions.desktop" > "${APP_DIR}/logioptions.desktop"
chmod 644 "${APP_DIR}/logioptions.desktop"

# Configure desktop autostart (Top Bar Battery Tray Widget on login)
AUTOSTART_DIR="${HOME}/.config/autostart"
mkdir -p "${AUTOSTART_DIR}"
cat <<EOF > "${AUTOSTART_DIR}/logioptions.desktop"
[Desktop Entry]
Name=LogiOptions Tray
Comment=Logitech MX Battery and Gesture Companion
Exec=${BIN_DIR}/logi-options --tray
Icon=logioptions
Terminal=false
Type=Application
Categories=Settings;HardwareSettings;Utility;
X-GNOME-Autostart-enabled=true
EOF
chmod 644 "${AUTOSTART_DIR}/logioptions.desktop"
echo "✅ Desktop autostart configured in ~/.config/autostart/logioptions.desktop."

# 6. Configure systemd user daemon
echo "⚙️ Setting up systemd background service..."
sed "s|/usr/local/bin/logid-service|${BIN_DIR}/logid-service|g" "${REPO_ROOT}/packaging/systemd/logioptions-daemon.service" > "${SYSTEMD_USER_DIR}/logioptions-daemon.service"

if command -v systemctl > /dev/null 2>&1; then
    systemctl --user daemon-reload
    systemctl --user enable --now logioptions-daemon.service || true
    echo "✅ Background daemon enabled via systemctl --user."
fi

# 7. Check udev rules
if [ ! -f "/etc/udev/rules.d/99-logitech-options.rules" ]; then
    echo "⚠️ udev rules not found in /etc/udev/rules.d/."
    if command -v sudo > /dev/null 2>&1; then
        echo "Installing udev rules (requires sudo)..."
        sudo cp "${REPO_ROOT}/packaging/udev/99-logitech-options.rules" "/etc/udev/rules.d/"
        sudo udevadm control --reload-rules && sudo udevadm trigger --action=change
        echo "✅ udev rules installed."
    else
        echo "Please run manually: sudo cp packaging/udev/99-logitech-options.rules /etc/udev/rules.d/"
    fi
else
    echo "✅ udev rules are already installed."
fi

# 8. Check if ~/.local/bin is in PATH
if [[ ":$PATH:" != *":${BIN_DIR}:"* ]]; then
    echo "⚠️  ${BIN_DIR} is not in your current PATH."
    if [ -f "${HOME}/.bashrc" ] && ! grep -q 'PATH.*\.local/bin' "${HOME}/.bashrc"; then
        echo 'export PATH="$HOME/.local/bin:$PATH"' >> "${HOME}/.bashrc"
        echo "Added ~/.local/bin to ~/.bashrc."
    fi
    echo ""
    echo "To update your current terminal session, run:"
    echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
    echo "  source ~/.bashrc"
fi

echo ""
echo "🎉 LogiOptions-Linux installation complete!"
echo "You can launch the app from your application launcher or run:"
echo "  ~/.local/bin/logi-options  (or 'logi-options' after reloading shell)"

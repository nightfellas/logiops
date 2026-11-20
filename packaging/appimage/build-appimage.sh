#!/usr/bin/env bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
APPDIR="${REPO_ROOT}/build/AppDir"

echo "📦 Packaging LogiOptions AppImage..."

cd "${REPO_ROOT}"
if [ -f "$HOME/.cargo/env" ]; then
    . "$HOME/.cargo/env"
fi

# 1. Build Rust Release Binaries
echo "Compiling release binaries..."
cargo build --release

# 2. Build Frontend
if [ -d "${REPO_ROOT}/crates/logi-gui/ui" ]; then
    echo "Building UI..."
    cd "${REPO_ROOT}/crates/logi-gui/ui"
    npm run build
fi

cd "${REPO_ROOT}"

# 3. Create AppDir Structure
rm -rf "${APPDIR}"
mkdir -p "${APPDIR}/usr/bin"
mkdir -p "${APPDIR}/usr/share/applications"
mkdir -p "${APPDIR}/usr/share/icons/hicolor/scalable/apps"

# 4. Copy Binaries
cp "${REPO_ROOT}/target/release/logid-service" "${APPDIR}/usr/bin/"
cp "${REPO_ROOT}/target/release/logi-options" "${APPDIR}/usr/bin/"

# 5. Copy Desktop entry, Icon, and AppRun
cp "${REPO_ROOT}/packaging/appimage/AppRun" "${APPDIR}/"
cp "${REPO_ROOT}/packaging/appimage/logioptions.desktop" "${APPDIR}/"
cp "${REPO_ROOT}/packaging/appimage/logioptions.desktop" "${APPDIR}/usr/share/applications/"
cp "${REPO_ROOT}/assets/icons/logioptions.svg" "${APPDIR}/logioptions.svg"
cp "${REPO_ROOT}/assets/icons/logioptions.svg" "${APPDIR}/usr/share/icons/hicolor/scalable/apps/"
cp "${REPO_ROOT}/assets/icons/logioptions.svg" "${APPDIR}/.DirIcon"

chmod +x "${APPDIR}/AppRun"

echo "✅ AppDir staged at: ${APPDIR}"

# 6. Check for appimagetool to create .AppImage
if command -v appimagetool > /dev/null 2>&1; then
    echo "Creating standalone .AppImage..."
    appimagetool "${APPDIR}" "${REPO_ROOT}/LogiOptions-x86_64.AppImage"
    echo "🎉 Generated: ${REPO_ROOT}/LogiOptions-x86_64.AppImage"
else
    echo "ℹ️  appimagetool not found on PATH. AppDir structure is ready."
    echo "To produce the final .AppImage, run:"
    echo "  ARCH=x86_64 appimagetool ${APPDIR} LogiOptions-x86_64.AppImage"
fi

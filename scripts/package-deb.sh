#!/bin/bash
set -e

VERSION="${1:?Usage: $0 <version>}"
DIST_DIR="${2:-dist}"
BINARY_DIR="${3:-target/release}"
DEB_ARCH="${4:-amd64}"

PKG_NAME="linuxcare_${VERSION}_${DEB_ARCH}"
PKG_DIR="${DIST_DIR}/${PKG_NAME}"

rm -rf "${PKG_DIR}"
mkdir -p "${PKG_DIR}/usr/local/bin"
mkdir -p "${PKG_DIR}/usr/share/applications"
mkdir -p "${PKG_DIR}/usr/share/icons/hicolor/scalable/apps"
mkdir -p "${PKG_DIR}/usr/share/doc/linuxcare"
mkdir -p "${PKG_DIR}/DEBIAN"

cp "${BINARY_DIR}/linuxcare" "${PKG_DIR}/usr/local/bin/"
cp "${BINARY_DIR}/linuxcare-cli" "${PKG_DIR}/usr/local/bin/"
cp data/linuxcare.desktop "${PKG_DIR}/usr/share/applications/"
cp icons/scalable/apps/linuxcare.svg "${PKG_DIR}/usr/share/icons/hicolor/scalable/apps/"
cp README.md "${PKG_DIR}/usr/share/doc/linuxcare/"
cp LICENSE "${PKG_DIR}/usr/share/doc/linuxcare/"

chmod 755 "${PKG_DIR}/usr/local/bin/linuxcare" "${PKG_DIR}/usr/local/bin/linuxcare-cli"

SIZE=$(du -sk "${PKG_DIR}" | cut -f1)

cat > "${PKG_DIR}/DEBIAN/control" << DEBCTRL
Package: linuxcare
Version: ${VERSION}
Section: utils
Priority: optional
Architecture: ${DEB_ARCH}
Installed-Size: ${SIZE}
Depends: libgtk-4-1 (>= 4.6), libadwaita-1-0 (>= 1.2)
Maintainer: LinuxCare Team
Description: A comprehensive Linux system management tool for GNOME desktop
 LinuxCare provides an intuitive interface for managing desktop
 environments, system settings, networking, software packages,
 disk usage, system cleanup, and much more.
DEBCTRL

cat > "${PKG_DIR}/DEBIAN/postinst" << 'DEBPOST'
#!/bin/sh
set -e
if [ -x /usr/bin/gtk-update-icon-cache ]; then
	gtk-update-icon-cache -f -t /usr/share/icons/hicolor 2>/dev/null || true
fi
if [ -x /usr/bin/update-desktop-database ]; then
	update-desktop-database /usr/share/applications 2>/dev/null || true
fi
DEBPOST

cat > "${PKG_DIR}/DEBIAN/postrm" << 'DEBPOSTRM'
#!/bin/sh
set -e
if [ -x /usr/bin/gtk-update-icon-cache ]; then
	gtk-update-icon-cache -f -t /usr/share/icons/hicolor 2>/dev/null || true
fi
if [ -x /usr/bin/update-desktop-database ]; then
	update-desktop-database /usr/share/applications 2>/dev/null || true
fi
DEBPOSTRM

chmod 755 "${PKG_DIR}/DEBIAN/postinst" "${PKG_DIR}/DEBIAN/postrm"

dpkg-deb --root-owner-group --build "${PKG_DIR}" "${DIST_DIR}/${PKG_NAME}.deb"
rm -rf "${PKG_DIR}"

echo "Package created: ${DIST_DIR}/${PKG_NAME}.deb"

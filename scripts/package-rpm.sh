#!/bin/bash
set -e

VERSION="${1:?Usage: $0 <version>}"
DIST_DIR="${2:-dist}"
BINARY_DIR="${3:-target/release}"
RPM_ARCH="${4:-x86_64}"

if ! command -v rpmbuild >/dev/null 2>&1; then
	echo "Error: rpmbuild not found. Install 'rpm-build' package."
	echo "  Ubuntu/Debian: sudo apt install rpm"
	echo "  Fedora/RHEL:   sudo dnf install rpm-build"
	exit 1
fi

PKG_NAME="linuxcare-${VERSION}-1.${RPM_ARCH}"
RPM_ROOT="${DIST_DIR}/rpmbuild"

rm -rf "${RPM_ROOT}"
mkdir -p "${RPM_ROOT}"/{BUILD,RPMS,SOURCES,SPECS,SRPMS}
mkdir -p "${RPM_ROOT}/tmp/LinuxCare-${VERSION}"/{usr/local/bin,usr/share/applications,usr/share/icons/hicolor/scalable/apps,usr/share/doc/linuxcare}

cp "${BINARY_DIR}/linuxcare" "${RPM_ROOT}/tmp/LinuxCare-${VERSION}/usr/local/bin/"
cp "${BINARY_DIR}/linuxcare-cli" "${RPM_ROOT}/tmp/LinuxCare-${VERSION}/usr/local/bin/"
cp data/linuxcare.desktop "${RPM_ROOT}/tmp/LinuxCare-${VERSION}/usr/share/applications/"
cp icons/scalable/apps/linuxcare.svg "${RPM_ROOT}/tmp/LinuxCare-${VERSION}/usr/share/icons/hicolor/scalable/apps/"
cp README.md "${RPM_ROOT}/tmp/LinuxCare-${VERSION}/usr/share/doc/linuxcare/"
cp LICENSE "${RPM_ROOT}/tmp/LinuxCare-${VERSION}/usr/share/doc/linuxcare/"

SPEC_FILE="${DIST_DIR}/linuxcare.spec"
cat > "${SPEC_FILE}" << RPMSPEC
Name:           linuxcare
Version:        ${VERSION}
Release:        1%{?dist}
Summary:        A comprehensive Linux system management tool for GNOME desktop
License:        MIT
URL:            https://github.com/linuxcare/linuxcare
BuildArch:      ${RPM_ARCH}
Requires:       gtk4 >= 4.6, libadwaita >= 1.2

%description
LinuxCare provides an intuitive interface for managing desktop
environments, system settings, networking, software packages,
disk usage, system cleanup, and much more.

%install
mkdir -p %{buildroot}/usr/local/bin
mkdir -p %{buildroot}/usr/share/applications
mkdir -p %{buildroot}/usr/share/icons/hicolor/scalable/apps
mkdir -p %{buildroot}/usr/share/doc/linuxcare
cp -r %{_topdir}/tmp/LinuxCare-${VERSION}/* %{buildroot}/

%files
/usr/local/bin/linuxcare
/usr/local/bin/linuxcare-cli
/usr/share/applications/linuxcare.desktop
/usr/share/icons/hicolor/scalable/apps/linuxcare.svg
/usr/share/doc/linuxcare/README.md
/usr/share/doc/linuxcare/LICENSE

%post
if [ -x /usr/bin/gtk-update-icon-cache ]; then
    gtk-update-icon-cache -f -t /usr/share/icons/hicolor 2>/dev/null || true
fi
if [ -x /usr/bin/update-desktop-database ]; then
    update-desktop-database /usr/share/applications 2>/dev/null || true
fi

%postun
if [ \$1 -eq 0 ]; then
    if [ -x /usr/bin/gtk-update-icon-cache ]; then
        gtk-update-icon-cache -f -t /usr/share/icons/hicolor 2>/dev/null || true
    fi
    if [ -x /usr/bin/update-desktop-database ]; then
        update-desktop-database /usr/share/applications 2>/dev/null || true
    fi
fi
RPMSPEC

rpmbuild --define "_topdir ${RPM_ROOT}" --buildroot "${RPM_ROOT}/tmp/LinuxCare-${VERSION}" -bb "${SPEC_FILE}"
cp "${RPM_ROOT}/RPMS/${RPM_ARCH}/"*.rpm "${DIST_DIR}/"
mv "${DIST_DIR}/${PKG_NAME}"*.rpm "${DIST_DIR}/${PKG_NAME}.rpm" 2>/dev/null || true
rm -rf "${RPM_ROOT}" "${SPEC_FILE}"

echo "Package created: ${DIST_DIR}/${PKG_NAME}.rpm"

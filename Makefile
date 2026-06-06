.PHONY: all build release clean install uninstall help
.PHONY: package package-tgz package-deb package-rpm package-all

VERSION := $(shell grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')
TARGET_TRIPLE := x86_64-unknown-linux-gnu
DEB_ARCH := amd64
RPM_ARCH := x86_64
DIST_DIR := dist
PKG_PREFIX := linuxcare-$(VERSION)

# Default target
all: build

build:
	cargo build

release:
	cargo build --release

clean:
	cargo clean

install: release
	sudo mkdir -p /usr/local/bin
	sudo mkdir -p /usr/share/applications
	sudo mkdir -p /usr/share/icons/hicolor/scalable/apps
	sudo cp target/release/linuxcare /usr/local/bin/
	sudo cp target/release/linuxcare-cli /usr/local/bin/
	sudo cp data/linuxcare.desktop /usr/share/applications/
	sudo cp icons/scalable/apps/linuxcare.svg /usr/share/icons/hicolor/scalable/apps/
	sudo gtk-update-icon-cache -f -t /usr/share/icons/hicolor || true
	sudo update-desktop-database /usr/share/applications || true
	@echo "LinuxCare $(VERSION) installed successfully!"

uninstall:
	sudo rm -f /usr/local/bin/linuxcare
	sudo rm -f /usr/local/bin/linuxcare-cli
	sudo rm -f /usr/share/applications/linuxcare.desktop
	sudo rm -f /usr/share/icons/hicolor/scalable/apps/linuxcare.svg
	sudo gtk-update-icon-cache -f -t /usr/share/icons/hicolor 2>/dev/null || true
	sudo update-desktop-database /usr/share/applications 2>/dev/null || true
	@echo "LinuxCare uninstalled successfully!"

run:
	cargo run

run-cli:
	cargo run --bin linuxcare-cli

check:
	cargo check

test:
	cargo test

fmt:
	cargo fmt

clippy:
	cargo clippy

package: package-tgz

package-tgz: release
	@PKG_NAME="$(PKG_PREFIX)-$(TARGET_TRIPLE)"; \
	PKG_DIR="$(DIST_DIR)/$$PKG_NAME"; \
	rm -rf "$$PKG_DIR"; \
	mkdir -p "$$PKG_DIR"; \
	cp target/release/linuxcare "$$PKG_DIR/"; \
	cp target/release/linuxcare-cli "$$PKG_DIR/"; \
	cp data/linuxcare.desktop "$$PKG_DIR/"; \
	cp icons/scalable/apps/linuxcare.svg "$$PKG_DIR/"; \
	cp install.sh "$$PKG_DIR/"; \
	cp uninstall.sh "$$PKG_DIR/"; \
	cp README.md "$$PKG_DIR/"; \
	cp LICENSE "$$PKG_DIR/"; \
	chmod +x "$$PKG_DIR/install.sh" "$$PKG_DIR/uninstall.sh"; \
	tar -czf "$(DIST_DIR)/$$PKG_NAME.tar.gz" -C "$(DIST_DIR)" "$$PKG_NAME"; \
	rm -rf "$$PKG_DIR"; \
	echo "Package created: $(DIST_DIR)/$$PKG_NAME.tar.gz"

package-deb: release
	@chmod +x scripts/package-deb.sh
	@scripts/package-deb.sh "$(VERSION)" "$(DIST_DIR)" target/release "$(DEB_ARCH)"

package-rpm: release
	@chmod +x scripts/package-rpm.sh
	@scripts/package-rpm.sh "$(VERSION)" "$(DIST_DIR)" target/release "$(RPM_ARCH)"

package-all: package-tgz package-deb
	@echo ""
	@echo "All packages created in $(DIST_DIR)/"
	@ls -lh $(DIST_DIR)/*.tar.gz $(DIST_DIR)/*.deb 2>/dev/null || true
	@echo ""
	@echo "Note: RPM requires 'rpmbuild'. Run 'make package-rpm' separately if available."

help:
	@echo "LinuxCare Build System"
	@echo "======================"
	@echo ""
	@echo "Version: $(VERSION)"
	@echo ""
	@echo "Build Targets:"
	@echo "  all          - Build the project (default)"
	@echo "  build        - Build debug mode"
	@echo "  release      - Build release mode"
	@echo "  clean        - Clean build artifacts"
	@echo ""
	@echo "Package Targets:"
	@echo "  package      - Create binary tarball (alias for package-tgz)"
	@echo "  package-tgz  - Create .tar.gz binary archive"
	@echo "  package-deb  - Create .deb Debian package"
	@echo "  package-rpm  - Create .rpm RPM package"
	@echo "  package-all  - Create all available packages"
	@echo ""
	@echo "Install Targets:"
	@echo "  install      - Install to /usr/local"
	@echo "  uninstall    - Remove from /usr/local"
	@echo ""
	@echo "Dev Targets:"
	@echo "  run          - Run GUI version"
	@echo "  run-cli      - Run CLI version"
	@echo "  check        - Check for warnings"
	@echo "  test         - Run tests"
	@echo "  fmt          - Format code"
	@echo "  clippy       - Run clippy"
	@echo "  help         - Show this help"

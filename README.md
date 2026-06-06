# LinuxCare

[English](README.md) | [简体中文](README.zh-CN.md) | [日本語](README.ja-JP.md) | [한국어](README.ko-KR.md)

A comprehensive Linux system management tool built with Rust and GTK4/libadwaita for the GNOME desktop environment.

LinuxCare provides a unified graphical interface to manage desktop settings, system configuration, software packages, network, firewall, drivers, backups, and system optimization — all in one application.

---

## Features

| Module | Description |
|--------|-------------|
| **Monitor** | Real-time system resource monitoring (CPU, memory, disk, network) |
| **Process** | Process viewer with kill, priority adjustment, and search |
| **Desktop** | Desktop icon themes, wallpapers, workspaces, fractional scaling |
| **System** | Power management, display settings, night light, sound settings |
| **Network** | WiFi management, VPN, proxy, DNS configuration, diagnostics |
| **Software** | Package management via APT, Flatpak, and Snap with search/install/remove |
| **Disk** | Disk usage monitoring and partition management |
| **Cleanup** | Clean APT/Snap/Flatpak cache, logs, temp files, browser cache, duplicate files |
| **Startup** | Manage autostart applications |
| **Service** | Manage systemd services |
| **User** | User account management |
| **Driver** | Hardware detection, driver recommendations, kernel module management |
| **Firewall** | UFW firewall management with rules, rate limiting, application profiles |
| **Backup** | Backup/restore via rsync, Timeshift, Borg; scheduled backups via cron |
| **Log** | System log viewer |
| **Menu** | Context menu and application menu management |
| **Shortcut** | Desktop and keyboard shortcut management |
| **Optimizer** | System optimization scanner (kernel tuning, services, filesystem, network, power, memory) |

---

## Screenshots

> TODO: Add screenshots here

---

## Requirements

- **Rust** 1.70 or later
- **GTK 4** development libraries (gtk4 v0.9)
- **libadwaita** development libraries (v0.7.2)
- **GNOME** desktop environment (recommended for full functionality)
- Linux kernel 5.x+

### System Dependencies

#### Ubuntu / Debian

```bash
sudo apt update
sudo apt install -y build-essential cargo libgtk-4-dev libadwaita-1-dev
```

#### Fedora

```bash
sudo dnf install -y rust-gtk4-devel libadwaita-devel
```

#### Arch Linux / Manjaro

```bash
sudo pacman -S base-devel rust gtk4 libadwaita
```

#### openSUSE

```bash
sudo zypper install -y gtk4-devel libadwaita-devel rustup
```

---

## Building

### Development Build

```bash
make build
# or
cargo build
```

### Release Build

```bash
make release
# or
cargo build --release
```

### Run Tests

```bash
make test
# or
cargo test
```

### Format & Lint

```bash
make fmt       # Format code
make clippy    # Run clippy lints
```

---

## Installation

### Quick Install (recommended)

```bash
make install
```

This will:
1. Build the release binary
2. Copy `linuxcare` and `linuxcare-cli` to `/usr/local/bin/`
3. Install the desktop entry to `/usr/share/applications/`
4. Install the application icon

### Manual Install

```bash
# Build
cargo build --release

# Install binaries
sudo cp target/release/linuxcare /usr/local/bin/
sudo cp target/release/linuxcare-cli /usr/local/bin/

# Install desktop file
sudo cp data/linuxcare.desktop /usr/share/applications/

# Install icon
sudo cp icons/scalable/apps/linuxcare.svg /usr/share/icons/hicolor/scalable/apps/
sudo gtk-update-icon-cache -f -t /usr/share/icons/hicolor
sudo update-desktop-database /usr/share/applications
```

### Uninstall

```bash
make uninstall
```

---

## Usage

### GUI Version

Launch from the GNOME application menu, or run:

```bash
linuxcare
```

The application provides a sidebar with 18 modules. Click any module to access its features.

#### Language Support

LinuxCare supports 4 languages:
- English (default)
- 简体中文 (Chinese Simplified)
- 日本語 (Japanese)
- 한국어 (Korean)

Switch languages using the language selector in the top toolbar. The application will restart with the new language.

### CLI Version

```bash
linuxcare-cli <command> [options]
```

#### Software Management

```bash
# List installed packages
linuxcare-cli software list

# Search for a package
linuxcare-cli software search firefox

# Install a package
linuxcare-cli software install firefox

# Remove a package
linuxcare-cli software remove firefox
```

#### System Cleanup

```bash
# Clean all caches
linuxcare-cli cleanup all

# Clean specific cache
linuxcare-cli cleanup apt
linuxcare-cli cleanup snap
linuxcare-cli cleanup flatpak
```

#### Disk Usage

```bash
linuxcare-cli disk
```

#### System Information

```bash
linuxcare-cli system
```

---

## Project Structure

```
linuxcare/
├── src/
│   ├── main.rs              # GUI entry point
│   ├── cli.rs               # CLI entry point
│   ├── utils.rs             # Shared utility functions (run_command, spawn_bg, etc.)
│   ├── i18n.rs              # Internationalization (4 languages)
│   └── modules/
│       ├── mod.rs           # Module declarations
│       ├── monitor.rs       # System resource monitoring
│       ├── process.rs       # Process management
│       ├── desktop.rs       # Desktop environment settings
│       ├── system.rs        # System settings (power, display, sound)
│       ├── network.rs       # Network/WiFi/VPN/Proxy/DNS management
│       ├── software.rs      # Package management (APT/Flatpak/Snap)
│       ├── disk.rs          # Disk usage and partition management
│       ├── cleanup.rs       # System cleanup and optimization
│       ├── startup.rs       # Autostart application management
│       ├── service.rs       # Systemd service management
│       ├── user.rs          # User account management
│       ├── driver.rs        # Hardware detection and driver management
│       ├── firewall.rs      # UFW firewall management
│       ├── backup.rs        # Backup and restore management
│       ├── logview.rs       # System log viewer
│       ├── menu.rs          # Context menu management
│       ├── shortcut.rs      # Keyboard shortcut management
│       └── optimizer.rs     # System optimization scanner
├── data/
│   ├── linuxcare.desktop    # Desktop entry file
│   └── resources.gresource.xml
├── icons/
│   └── scalable/apps/linuxcare.svg
├── Cargo.toml               # Rust project configuration
├── Cargo.lock               # Dependency lock file
├── Makefile                 # Build system
├── LICENSE                  # MIT License
└── README.md
```

---

## Open Source Dependencies

This project is built upon the following open source projects:

### Core Framework

| Crate | Version | License | Description |
|-------|---------|---------|-------------|
| [gtk4-rs](https://github.com/gtk-rs/gtk4-rs) | 0.9 | MIT | Rust bindings for GTK4 |
| [libadwaita-rs](https://github.com/gtk-rs/libadwaita-rs) | 0.7.2 | MIT | Rust bindings for libadwaita (GNOME Adwaita) |
| [sysinfo](https://github.com/GuillaumeGomez/sysinfo) | 0.31 | MIT | Cross-platform system information |
| [nix](https://github.com/nix-rust/nix) | 0.29 | MIT | Rust friendly bindings to \*nix APIs |
| [anyhow](https://github.com/dtolnay/anyhow) | 1.0 | MIT | Flexible error handling |
| [serde](https://github.com/serde-rs/serde) | 1.0 | MIT/Apache-2.0 | Serialization framework |
| [dirs](https://github.com/soc/dirs-rs) | 5.0 | MIT | Standard directory paths |
| [regex](https://github.com/rust-lang/regex) | 1.10 | MIT/Apache-2.0 | Regular expressions |
| [log](https://github.com/rust-lang/log) | 0.4 | MIT/Apache-2.0 | Logging facade |
| [env_logger](https://github.com/rust-cli/env_logger) | 0.11 | MIT/Apache-2.0 | Logging implementation |

### Inspired By

- **GNOME Settings** — System settings UI patterns
- **GNOME Tweaks** — Desktop customization approach
- **Stacer** — System optimizer and cleaner
- **BleachBit** — System cleanup inspiration
- **UFW / Gufw** — Firewall management patterns
- **Timeshift** — Backup and restore approach

---

## License

This project is licensed under the **MIT License**.

```
MIT License

Copyright (c) 2026 Xinghe-0203, 刘畅

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

See [LICENSE](LICENSE) for the full text.

---

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

---

## Author

**Xinghe-0203, 刘畅**

---

## Acknowledgments

- The [GTK-rs](https://github.com/gtk-rs) community for excellent Rust bindings
- The [libadwaita](https://gnome.pages.gitlab.gnome.org/libadwaita/) team for modern GNOME widgets
- All open-source projects that inspired this tool

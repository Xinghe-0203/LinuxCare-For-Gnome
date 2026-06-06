# LinuxCare

[English](README.md) | [简体中文](README.zh-CN.md) | [日本語](README.ja-JP.md) | [한국어](README.ko-KR.md)

一个基于 Rust 和 GTK4/libadwaita 构建的综合性 Linux 系统管理工具，专为 GNOME 桌面环境设计。

LinuxCare 提供统一的图形化界面，用于管理桌面设置、系统配置、软件包、网络、防火墙、驱动、备份和系统优化——所有功能集成在一个应用中。

---

## 功能特性

| 模块 | 说明 |
|------|------|
| **系统监控** | 实时监控系统资源（CPU、内存、磁盘、网络） |
| **进程管理** | 进程查看器，支持终止进程、调整优先级和搜索 |
| **桌面管理** | 桌面图标主题、壁纸、工作区、分数缩放 |
| **系统设置** | 电源管理、显示设置、夜灯模式、声音设置 |
| **网络管理** | WiFi 管理、VPN、代理、DNS 配置、网络诊断 |
| **软件管理** | 通过 APT、Flatpak、Snap 管理软件包（搜索/安装/卸载） |
| **磁盘管理** | 磁盘使用监控和分区管理 |
| **系统清理** | 清理 APT/Snap/Flatpak 缓存、日志、临时文件、浏览器缓存、重复文件 |
| **启动管理** | 管理开机自启动应用 |
| **服务管理** | 管理 systemd 服务 |
| **用户管理** | 用户账户管理 |
| **驱动管理** | 硬件检测、驱动推荐、内核模块管理 |
| **防火墙** | UFW 防火墙管理，支持规则、速率限制、应用配置文件 |
| **备份管理** | 通过 rsync、Timeshift、Borg 进行备份/恢复；支持 cron 定时备份 |
| **日志查看** | 系统日志查看器 |
| **菜单管理** | 上下文菜单和应用菜单管理 |
| **快捷键管理** | 桌面快捷键和键盘快捷键管理 |
| **系统优化** | 系统优化扫描器（内核调优、服务管理、文件系统、网络、电源、内存） |

---

## 系统要求

- **Rust** 1.70 或更高版本
- **GTK 4** 开发库（gtk4 v0.9）
- **libadwaita** 开发库（v0.7.2）
- **GNOME** 桌面环境（推荐，以获得完整功能）
- Linux 内核 5.x+

### 系统依赖安装

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

## 构建

### 开发构建

```bash
make build
# 或
cargo build
```

### 发布构建

```bash
make release
# 或
cargo build --release
```

### 运行测试

```bash
make test
# 或
cargo test
```

### 格式化与代码检查

```bash
make fmt       # 格式化代码
make clippy    # 运行 clippy 检查
```

---

## 安装

### 快速安装（推荐）

```bash
make install
```

此命令将：
1. 构建发布版本
2. 将 `linuxcare` 和 `linuxcare-cli` 复制到 `/usr/local/bin/`
3. 安装桌面快捷方式到 `/usr/share/applications/`
4. 安装应用程序图标

### 手动安装

```bash
# 构建
cargo build --release

# 安装二进制文件
sudo cp target/release/linuxcare /usr/local/bin/
sudo cp target/release/linuxcare-cli /usr/local/bin/

# 安装桌面文件
sudo cp data/linuxcare.desktop /usr/share/applications/

# 安装图标
sudo cp icons/scalable/apps/linuxcare.svg /usr/share/icons/hicolor/scalable/apps/
sudo gtk-update-icon-cache -f -t /usr/share/icons/hicolor
sudo update-desktop-database /usr/share/applications
```

### 卸载

```bash
make uninstall
```

---

## 使用说明

### 图形界面版本

从 GNOME 应用菜单启动，或运行：

```bash
linuxcare
```

应用提供侧边栏导航，包含 18 个功能模块。点击任意模块即可访问其功能。

#### 语言支持

LinuxCare 支持 4 种语言：
- English（英语，默认）
- 简体中文
- 日本語（日语）
- 한국어（韩语）

使用顶部工具栏的语言选择器切换语言。切换后应用将自动重启。

### 命令行版本

```bash
linuxcare-cli <命令> [选项]
```

#### 软件管理

```bash
# 列出已安装的软件包
linuxcare-cli software list

# 搜索软件包
linuxcare-cli software search firefox

# 安装软件包
linuxcare-cli software install firefox

# 卸载软件包
linuxcare-cli software remove firefox
```

#### 系统清理

```bash
# 清理所有缓存
linuxcare-cli cleanup all

# 清理特定缓存
linuxcare-cli cleanup apt
linuxcare-cli cleanup snap
linuxcare-cli cleanup flatpak
```

#### 磁盘使用

```bash
linuxcare-cli disk
```

#### 系统信息

```bash
linuxcare-cli system
```

---

## 项目结构

```
linuxcare/
├── src/
│   ├── main.rs              # GUI 入口
│   ├── cli.rs               # CLI 入口
│   ├── utils.rs             # 共享工具函数
│   ├── i18n.rs              # 国际化（4 种语言）
│   └── modules/
│       ├── mod.rs           # 模块声明
│       ├── monitor.rs       # 系统资源监控
│       ├── process.rs       # 进程管理
│       ├── desktop.rs       # 桌面环境设置
│       ├── system.rs        # 系统设置（电源、显示、声音）
│       ├── network.rs       # 网络/WiFi/VPN/代理/DNS 管理
│       ├── software.rs      # 软件包管理（APT/Flatpak/Snap）
│       ├── disk.rs          # 磁盘使用和分区管理
│       ├── cleanup.rs       # 系统清理和优化
│       ├── startup.rs       # 开机自启动管理
│       ├── service.rs       # Systemd 服务管理
│       ├── user.rs          # 用户账户管理
│       ├── driver.rs        # 硬件检测和驱动管理
│       ├── firewall.rs      # UFW 防火墙管理
│       ├── backup.rs        # 备份和恢复管理
│       ├── logview.rs       # 系统日志查看
│       ├── menu.rs          # 上下文菜单管理
│       ├── shortcut.rs      # 键盘快捷键管理
│       └── optimizer.rs     # 系统优化扫描器
├── data/
│   ├── linuxcare.desktop    # 桌面快捷方式
│   └── resources.gresource.xml
├── icons/
│   └── scalable/apps/linuxcare.svg
├── Cargo.toml               # Rust 项目配置
├── Cargo.lock               # 依赖锁定文件
├── Makefile                 # 构建系统
├── LICENSE                  # MIT 许可证
└── README.md
```

---

## 开源依赖

本项目基于以下开源项目构建：

### 核心框架

| 依赖 | 版本 | 许可证 | 说明 |
|------|------|--------|------|
| [gtk4-rs](https://github.com/gtk-rs/gtk4-rs) | 0.9 | MIT | GTK4 的 Rust 绑定 |
| [libadwaita-rs](https://github.com/gtk-rs/libadwaita-rs) | 0.7.2 | MIT | libadwaita 的 Rust 绑定 |
| [sysinfo](https://github.com/GuillaumeGomez/sysinfo) | 0.31 | MIT | 跨平台系统信息 |
| [nix](https://github.com/nix-rust/nix) | 0.29 | MIT | \*nix API 的 Rust 友好绑定 |
| [anyhow](https://github.com/dtolnay/anyhow) | 1.0 | MIT | 灵活的错误处理 |
| [serde](https://github.com/serde-rs/serde) | 1.0 | MIT/Apache-2.0 | 序列化框架 |
| [dirs](https://github.com/soc/dirs-rs) | 5.0 | MIT | 标准目录路径 |
| [regex](https://github.com/rust-lang/regex) | 1.10 | MIT/Apache-2.0 | 正则表达式 |
| [log](https://github.com/rust-lang/log) | 0.4 | MIT/Apache-2.0 | 日志门面 |
| [env_logger](https://github.com/rust-cli/env_logger) | 0.11 | MIT/Apache-2.0 | 日志实现 |

### 致敬项目

- **GNOME Settings** — 系统设置 UI 设计模式
- **GNOME Tweaks** — 桌面自定义方法
- **Stacer** — 系统优化器和清理器
- **BleachBit** — 系统清理灵感
- **UFW / Gufw** — 防火墙管理模式
- **Timeshift** — 备份和恢复方法

---

## 许可证

本项目采用 **MIT 许可证**。

详情请参阅 [LICENSE](LICENSE) 文件。

---

## 贡献

欢迎贡献！请随时提交 Pull Request。

1. Fork 本仓库
2. 创建你的功能分支 (`git checkout -b feature/amazing-feature`)
3. 提交你的更改 (`git commit -m '添加某个功能'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 打开一个 Pull Request

---

## 作者

**Xinghe-0203, 刘畅**

---

## 致谢

- [GTK-rs](https://github.com/gtk-rs) 社区提供的优秀 Rust 绑定
- [libadwaita](https://gnome.pages.gitlab.gnome.org/libadwaita/) 团队提供的现代 GNOME 组件
- 所有启发本项目的开源项目

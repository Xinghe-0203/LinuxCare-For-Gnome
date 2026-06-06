# LinuxCare

[English](README.md) | [简体中文](README.zh-CN.md) | [日本語](README.ja-JP.md) | [한국어](README.ko-KR.md)

Rust と GTK4/libadwaita で構築された、GNOME デスクトップ環境向けの包括的な Linux システム管理ツールです。

LinuxCare は、デスクトップ設定、システム設定、ソフトウェアパッケージ、ネットワーク、ファイアウォール、ドライバー、バックアップ、システム最適化を統一されたグラフィカルインターフェースで管理できます。

---

## 機能

| モジュール | 説明 |
|-----------|------|
| **モニター** | システムリソースのリアルタイム監視（CPU、メモリ、ディスク、ネットワーク） |
| **プロセス** | プロセスビューア、プロセス終了、優先度調整、検索 |
| **デスクトップ** | デスクトップアイコンテーマ、壁紙、ワークスペース、分数スケーリング |
| **システム** | 電源管理、ディスプレイ設定、ナイトライト、サウンド設定 |
| **ネットワーク** | WiFi管理、VPN、プロキシ、DNS設定、診断 |
| **ソフトウェア** | APT、Flatpak、Snapによるパッケージ管理（検索/インストール/アンインストール） |
| **ディスク** | ディスク使用量の監視とパーティション管理 |
| **クリーンアップ** | APT/Snap/Flatpakキャッシュ、ログ、一時ファイル、ブラウザキャッシュ、重複ファイルのクリーンアップ |
| **スタートアップ** | オートスタートアプリケーションの管理 |
| **サービス** | systemdサービスの管理 |
| **ユーザー** | ユーザーアカウント管理 |
| **ドライバー** | ハードウェア検出、ドライバー推奨、カーネルモジュール管理 |
| **ファイアウォール** | UFWファイアウォール管理（ルール、レート制限、アプリケーションプロファイル） |
| **バックアップ** | rsync、Timeshift、Borgによるバックアップ/リストア、cron定期バックアップ |
| **ログ** | システムログビューア |
| **メニュー** | コンテキストメニューとアプリケーションメニューの管理 |
| **ショートカット** | デスクトップショートカットとキーボードショートカットの管理 |
| **オプティマイザー** | システム最適化スキャナー（カーネルチューニング、サービス管理、ファイルシステム、ネットワーク、電源、メモリ） |

---

## 必要要件

- **Rust** 1.70 以降
- **GTK 4** 開発ライブラリ（gtk4 v0.9）
- **libadwaita** 開発ライブラリ（v0.7.2）
- **GNOME** デスクトップ環境（推奨）
- Linux カーネル 5.x+

### システム依存関係のインストール

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

---

## ビルド

### 開発ビルド

```bash
make build
```

### リリースビルド

```bash
make release
```

### テスト実行

```bash
make test
```

---

## インストール

### クイックインストール（推奨）

```bash
make install
```

### アンインストール

```bash
make uninstall
```

---

## 使い方

### GUI版

GNOME アプリケーションメニューから起動するか、以下を実行：

```bash
linuxcare
```

サイドバーに18のモジュールが表示されます。モジュールをクリックするとその機能にアクセスできます。

#### 言語サポート

LinuxCare は4つの言語をサポートしています：
- English（英語、デフォルト）
- 简体中文（中国語）
- 日本語
- 한국어（韓国語）

上部ツールバーの言語セレクターで言語を切り替えられます。

### CLI版

```bash
linuxcare-cli <コマンド> [オプション]
```

#### 例

```bash
linuxcare-cli software list          # インストール済みパッケージの表示
linuxcare-cli software search firefox # パッケージの検索
linuxcare-cli software install firefox # パッケージのインストール
linuxcare-cli cleanup all            # すべてのキャッシュをクリーン
linuxcare-cli disk                   # ディスク使用量の表示
linuxcare-cli system                 # システム情報の表示
```

---

## ライセンス

本プロジェクトは **MIT ライセンス** の下で公開されています。詳細は [LICENSE](LICENSE) を参照してください。

---

## 貢献

貢献は歓迎します！プルリクエストを自由に送信してください。

---

## オープンソース依存関係

| パッケージ | バージョン | ライセンス | 説明 |
|-----------|-----------|-----------|------|
| [gtk4-rs](https://github.com/gtk-rs/gtk4-rs) | 0.9 | MIT | GTK4のRustバインディング |
| [libadwaita-rs](https://github.com/gtk-rs/libadwaita-rs) | 0.7.2 | MIT | libadwaitaのRustバインディング |
| [sysinfo](https://github.com/GuillaumeGomez/sysinfo) | 0.31 | MIT | クロスプラットフォームシステム情報 |
| [nix](https://github.com/nix-rust/nix) | 0.29 | MIT | \*nix APIのRustバインディング |
| [anyhow](https://github.com/dtolnay/anyhow) | 1.0 | MIT | エラーハンドリング |
| [serde](https://github.com/serde-rs/serde) | 1.0 | MIT/Apache-2.0 | シリアライゼーション |

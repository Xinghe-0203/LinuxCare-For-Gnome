# LinuxCare

[English](README.md) | [简体中文](README.zh-CN.md) | [日本語](README.ja-JP.md) | [한국어](README.ko-KR.md)

Rust와 GTK4/libadwaita로 구축된 GNOME 데스크톱 환경용 종합 Linux 시스템 관리 도구입니다.

LinuxCare는 데스크톱 설정, 시스템 구성, 소프트웨어 패키지, 네트워크, 방화벽, 드라이버, 백업, 시스템 최적화를 하나의 통합 그래픽 인터페이스로 관리할 수 있습니다.

---

## 기능

| 모듈 | 설명 |
|------|------|
| **모니터** | 시스템 리소스 실시간 모니터링 (CPU, 메모리, 디스크, 네트워크) |
| **프로세스** | 프로세스 뷰어, 프로세스 종료, 우선순위 조정, 검색 |
| **데스크톱** | 데스크톱 아이콘 테마, 배경화면, 워크스페이스, 분수 스케일링 |
| **시스템** | 전원 관리, 디스플레이 설정, 야간 조명, 사운드 설정 |
| **네트워크** | WiFi 관리, VPN, 프록시, DNS 구성, 진단 |
| **소프트웨어** | APT, Flatpak, Snap을 통한 패키지 관리 (검색/설치/제거) |
| **디스크** | 디스크 사용량 모니터링 및 파티션 관리 |
| **정리** | APT/Snap/Flatpak 캐시, 로그, 임시 파일, 브라우저 캐시, 중복 파일 정리 |
| **시작 프로그램** | 자동 시작 애플리케이션 관리 |
| **서비스** | systemd 서비스 관리 |
| **사용자** | 사용자 계정 관리 |
| **드라이버** | 하드웨어 감지, 드라이버 추천, 커널 모듈 관리 |
| **방화벽** | UFW 방화벽 관리 (규칙, 속도 제한, 애플리케이션 프로필) |
| **백업** | rsync, Timeshift, Borg를 위한 백업/복원, cron 예약 백업 |
| **로그** | 시스템 로그 뷰어 |
| **메뉴** | 컨텍스트 메뉴 및 애플리케이션 메뉴 관리 |
| **단축키** | 데스크톱 단축키 및 키보드 단축키 관리 |
| **최적화** | 시스템 최적화 스캐너 (커널 튜닝, 서비스 관리, 파일 시스템, 네트워크, 전원, 메모리) |

---

## 시스템 요구 사양

- **Rust** 1.70 이상
- **GTK 4** 개발 라이브러리 (gtk4 v0.9)
- **libadwaita** 개발 라이브러리 (v0.7.2)
- **GNOME** 데스크톱 환경 (권장)
- Linux 커널 5.x+

### 시스템 의존성 설치

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

## 빌드

### 개발 빌드

```bash
make build
```

### 릴리스 빌드

```bash
make release
```

### 테스트 실행

```bash
make test
```

---

## 설치

### 빠른 설치 (권장)

```bash
make install
```

### 제거

```bash
make uninstall
```

---

## 사용법

### GUI 버전

GNOME 애플리케이션 메뉴에서 실행하거나 다음을 입력:

```bash
linuxcare
```

사이드바에 18개의 모듈이 표시됩니다. 모듈을 클릭하면 해당 기능에 접근할 수 있습니다.

#### 언어 지원

LinuxCare는 4개 언어를 지원합니다:
- English (영어, 기본)
- 简体中文 (중국어)
- 日本語 (일본어)
- 한국어

상단 툴바의 언어 선택기로 언어를 전환할 수 있습니다.

### CLI 버전

```bash
linuxcare-cli <명령어> [옵션]
```

#### 예제

```bash
linuxcare-cli software list          # 설치된 패키지 목록
linuxcare-cli software search firefox # 패키지 검색
linuxcare-cli software install firefox # 패키지 설치
linuxcare-cli cleanup all            # 모든 캐시 정리
linuxcare-cli disk                   # 디스크 사용량 표시
linuxcare-cli system                 # 시스템 정보 표시
```

---

## 라이선스

이 프로젝트는 **MIT 라이선스**로 배포됩니다. 자세한 내용은 [LICENSE](LICENSE)를 참조하세요.

---

## 기여

기여를 환영합니다! Pull Request를 자유롭게 제출해 주세요.

---

## 오픈소스 의존성

| 패키지 | 버전 | 라이선스 | 설명 |
|--------|------|---------|------|
| [gtk4-rs](https://github.com/gtk-rs/gtk4-rs) | 0.9 | MIT | GTK4의 Rust 바인딩 |
| [libadwaita-rs](https://github.com/gtk-rs/libadwaita-rs) | 0.7.2 | MIT | libadwaita의 Rust 바인딩 |
| [sysinfo](https://github.com/GuillaumeGomez/sysinfo) | 0.31 | MIT | 크로스 플랫폼 시스템 정보 |
| [nix](https://github.com/nix-rust/nix) | 0.29 | MIT | \*nix API의 Rust 바인딩 |
| [anyhow](https://github.com/dtolnay/anyhow) | 1.0 | MIT | 에러 처리 |
| [serde](https://github.com/serde-rs/serde) | 1.0 | MIT/Apache-2.0 | 직렬화 프레임워크 |

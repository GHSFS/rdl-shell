# rdl-shell

> Windows Explorer shell extension for [remote-dl](https://github.com/GHSFS/remote-dl) — adds a "Send to remote-dl" entry to the right-click menu of `.url` (Internet Shortcut) files.

[![Build](https://github.com/GHSFS/rdl-shell/actions/workflows/build.yml/badge.svg)](https://github.com/GHSFS/rdl-shell/actions/workflows/build.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](./LICENSE)
[![Platform](https://img.shields.io/badge/platform-Windows%20x64-blue)](#installation)

[English](#english) · [한국어](#한국어)

---

## English

### Overview

`rdl-shell.dll` is a Windows Shell context-menu handler that integrates the
[`rdl`](https://github.com/GHSFS/remote-dl) CLI client into Explorer. Once
registered, right-clicking on a `.url` Internet Shortcut file shows a new
menu entry — **Send to remote-dl** — that reads the URL from the shortcut
and queues it on the configured edge worker.

This is the third piece of the personal-use `remote-dl` ecosystem:

| Tool | Surface | Repo |
|---|---|---|
| `rdl.exe` | terminal CLI | [`remote-dl`](https://github.com/GHSFS/remote-dl) |
| `rdl-tray.exe` | system tray + clipboard watcher | [`rdl-tray`](https://github.com/GHSFS/rdl-tray) |
| **`rdl-shell.dll`** | **Explorer right-click menu** | this repo |

All three share the same on-disk configuration written by the CLI
(`%APPDATA%\rdl\config.json`).

### Architecture

The DLL implements the standard four COM entry points and two Shell
interfaces:

| Symbol | Purpose |
|---|---|
| `DllGetClassObject` | Returns the `IClassFactory` for our CLSID. |
| `DllCanUnloadNow` | Reports whether outstanding objects keep the DLL pinned. |
| `DllRegisterServer` | Installs registry entries (called by `regsvr32`). |
| `DllUnregisterServer` | Removes registry entries (called by `regsvr32 /u`). |
| `IShellExtInit` | Receives the file selection from Explorer. |
| `IContextMenu` | Renders the menu item and reacts to `InvokeCommand`. |

### Installation

> Requires administrator privileges only for the registration step.

1. Download `rdl-shell.dll` from the
   [Releases](https://github.com/GHSFS/rdl-shell/releases) page.
2. Place it in a stable location (e.g. `C:\Tools\rdl-shell.dll`). Do not
   move it later — the registry entry is path-bound.
3. Open an elevated PowerShell and run:

   ```powershell
   regsvr32 C:\Tools\rdl-shell.dll
   ```

4. Restart Explorer (or log out and back in) so it reloads context-menu
   handlers.

### Removal

```powershell
regsvr32 /u C:\Tools\rdl-shell.dll
```

After this, the file may be deleted.

### Build from source

Requires Rust 1.75+ and the MSVC toolchain.

```bash
git clone https://github.com/GHSFS/rdl-shell.git
cd rdl-shell
cargo build --release
# Output: target/x86_64-pc-windows-msvc/release/rdl_shell.dll
```

### Configuration

There is no `rdl-shell`-specific configuration. The DLL reads
`%APPDATA%\rdl\config.json` written by the `rdl` CLI. Authenticate once with
the CLI and all three tools work.

### License

MIT. See [LICENSE](./LICENSE).

### Disclaimer

This is a personal-use shell extension. Shell extensions run inside
`explorer.exe`; if you build a modified version that crashes, Explorer will
restart. The released build is well-behaved but you install it at your own
risk.

---

## 한국어

### 개요

`rdl-shell.dll`은 Windows Explorer 셸 컨텍스트 메뉴 핸들러로,
[`rdl`](https://github.com/GHSFS/remote-dl) CLI 클라이언트를 Explorer에
통합합니다. 등록 후, `.url` Internet Shortcut 파일을 우클릭하면 새 메뉴
항목 — **Send to remote-dl** — 가 추가되어 단축에 저장된 URL을 워커에
큐잉합니다.

`remote-dl` 개인용 생태계의 세 번째 도구:

| 도구 | 인터페이스 | 레포 |
|---|---|---|
| `rdl.exe` | 터미널 CLI | [`remote-dl`](https://github.com/GHSFS/remote-dl) |
| `rdl-tray.exe` | 시스템 트레이 + 클립보드 감시 | [`rdl-tray`](https://github.com/GHSFS/rdl-tray) |
| **`rdl-shell.dll`** | **Explorer 우클릭 메뉴** | 이 레포 |

세 도구 모두 CLI가 작성한 동일한 디스크 설정 파일
(`%APPDATA%\rdl\config.json`)을 공유합니다.

### 아키텍처

DLL이 구현하는 표준 COM 진입점 4개 + 셸 인터페이스 2개:

| 심볼 | 역할 |
|---|---|
| `DllGetClassObject` | CLSID에 대한 `IClassFactory` 반환 |
| `DllCanUnloadNow` | 외부 참조가 남아있는지 보고 |
| `DllRegisterServer` | `regsvr32` 호출 시 레지스트리 항목 설치 |
| `DllUnregisterServer` | `regsvr32 /u` 호출 시 항목 제거 |
| `IShellExtInit` | Explorer로부터 선택된 파일 받음 |
| `IContextMenu` | 메뉴 항목 렌더링 + `InvokeCommand` 처리 |

### 설치

> 관리자 권한은 레지스트리 등록 단계에서만 필요합니다.

1. [Releases](https://github.com/GHSFS/rdl-shell/releases) 페이지에서
   `rdl-shell.dll` 다운로드
2. 영구 경로에 배치 (예: `C:\Tools\rdl-shell.dll`). 이후 이동 금지 —
   레지스트리 항목이 경로에 바인딩됩니다.
3. 관리자 PowerShell에서:

   ```powershell
   regsvr32 C:\Tools\rdl-shell.dll
   ```

4. Explorer 재시작 (또는 로그오프/로그인) — 컨텍스트 메뉴 핸들러 다시 로드

### 제거

```powershell
regsvr32 /u C:\Tools\rdl-shell.dll
```

이후 DLL 파일 삭제 가능.

### 소스 빌드

Rust 1.75+ 및 MSVC 툴체인 필요.

```bash
git clone https://github.com/GHSFS/rdl-shell.git
cd rdl-shell
cargo build --release
# 결과물: target/x86_64-pc-windows-msvc/release/rdl_shell.dll
```

### 설정

`rdl-shell` 전용 설정은 없습니다. CLI가 작성한
`%APPDATA%\rdl\config.json`을 그대로 사용하므로, CLI에서 한 번 인증하면 세
도구 모두 동작합니다.

### 라이선스

MIT. [LICENSE](./LICENSE) 참고.

### 면책

개인용 셸 확장입니다. 셸 확장은 `explorer.exe` 안에서 실행되므로, 수정된
버전이 크래시하면 Explorer가 재시작됩니다. 릴리스 빌드는 안정적이지만
설치는 본인 책임 하에 진행하세요.

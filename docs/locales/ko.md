# rdl-shell

<div align="center">

<p align="center">
  <a href="../../README.md">🇺🇸 English</a> · <strong>🇰🇷 한국어</strong> · <a href="./zh-CN.md">🇨🇳 中文</a> · <a href="./es.md">🇪🇸 Español</a> · <a href="./hi.md">🇮🇳 हिन्दी</a><br />
  <a href="./ar.md">🇸🇦 العربية</a> · <a href="./pt-BR.md">🇧🇷 Português</a> · <a href="./ru.md">🇷🇺 Русский</a> · <a href="./fr.md">🇫🇷 Français</a> · <a href="./id.md">🇮🇩 Bahasa Indonesia</a>
</p>

> [remote-dl](https://github.com/GHSFS/remote-dl)을 위한 Windows Explorer 셸 확장 — `.url` (인터넷 바로가기) 파일의 우클릭 메뉴에 "Send to remote-dl" 항목을 추가합니다.

[![Build](https://github.com/GHSFS/rdl-shell/actions/workflows/build.yml/badge.svg)](https://github.com/GHSFS/rdl-shell/actions/workflows/build.yml)
[![Tests](https://github.com/GHSFS/rdl-shell/actions/workflows/test.yml/badge.svg)](https://github.com/GHSFS/rdl-shell/actions/workflows/test.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](../../LICENSE)
[![Platform](https://img.shields.io/badge/platform-Windows%20x64-blue)](#설치)

</div>

---

### 개요

`rdl-shell.dll`은 Windows Explorer 셸 컨텍스트 메뉴 핸들러로, [`rdl`](https://github.com/GHSFS/remote-dl) CLI 클라이언트를 Explorer에 통합합니다. 등록 후, `.url` 인터넷 바로가기 파일을 우클릭하면 새 메뉴 항목 — **Send to remote-dl** (다중 선택 시 **Send N shortcuts to remote-dl**) — 가 추가되어 바로가기에 저장된 URL을 백엔드 워커에 큐잉합니다.

#### ✨ 주요 핵심 기능 (v0.2.0)
- **다중 파일 일괄 큐잉 (Multi-File Selection)**: 여러 개의 `.url` 바로가기를 동시 선택하여 원클릭으로 일괄 전송 지원.
- **Explorer UI 프리징 방지 (Non-Blocking Async)**: 백그라운드 워커 스레드 분기 및 `DllLockGuard` RAII 수명 가드로 탐색기 먹통 0% 보장.
- **범용 인코딩 완벽 호환**: UTF-8, UTF-16LE(BOM 유무 무관), UTF-16BE 및 대소문자 무관 `url=` 키 자동 감지 및 디코딩.
- **엄격한 URL 스키마 검증**: `http://` 및 `https://` 외의 비정상 스키마 사전 차단.
- **연결 풀링 최적화**: `reqwest` 클라이언트 캐싱(`OnceLock`)을 통해 TCP Keep-Alive 및 DNS 캐시 재활용.

`remote-dl` 개인용 생태계의 세 번째 도구:

| 도구 | 인터페이스 | 레포지토리 |
|---|---|---|
| `rdl.exe` | 터미널 CLI | [`remote-dl`](https://github.com/GHSFS/remote-dl) |
| `rdl-tray.exe` | 시스템 트레이 + 클립보드 감시 | [`rdl-tray`](https://github.com/GHSFS/rdl-tray) |
| **`rdl-shell.dll`** | **Explorer 우클릭 메뉴** | 이 레포지토리 |

세 도구 모두 CLI가 작성한 동일한 디스크 설정 파일(`%APPDATA%\rdl\config.json`)을 공유합니다.

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

### 프로젝트 구조

```
rdl-shell/
├── Cargo.toml             패키지 매니페스트, crate-type=cdylib (DLL 출력)
├── rust-toolchain.toml    툴체인 고정 (stable / x86_64-pc-windows-msvc)
├── .cargo/config.toml     빌드 타겟 + 정적 MSVC CRT
├── build.rs               리소스 컴파일 + /DEF: 링커 인자 주입
├── src/
│   ├── lib.rs             CLSID, REF_COUNT, DLL_HMODULE, 4개 COM 진입점
│   ├── factory.rs         IClassFactory 구현
│   ├── handler.rs         IShellExtInit + IContextMenu 구현
│   ├── registry.rs        DllRegisterServer / DllUnregisterServer
│   ├── client.rs          reqwest::blocking 클라이언트, POST /api/dl
│   ├── config.rs          %APPDATA%\rdl\config.json 읽기 + DPAPI 복호화
│   └── error.rs           crate 전체 에러 타입
├── tests/smoke.rs         빌드 검증용 스모크 테스트
├── resources/
│   ├── app.rc             Win32 리소스 스크립트 (매니페스트 + VERSIONINFO)
│   ├── app.manifest       OS 호환성 선언
│   └── rdl-shell.def      COM export를 이름으로 노출하는 모듈 정의 파일
└── .github/workflows/
    ├── build.yml          cargo build + dumpbin /EXPORTS 검증 CI
    └── test.yml           fmt / clippy / test CI
```

### 호환성

| 항목 | 지원 사양 |
|---|---|
| 운영체제 | Windows 10 1809+ 및 Windows 11 |
| 아키텍처 | x86_64 전용 (Explorer 프로세스 비트수와 일치) |
| Rust 툴체인 | 1.75+ (`rust-toolchain.toml`에서 stable 고정) |
| 링커 | MSVC (Visual Studio Build Tools 2022) |
| 레지스트리 등록 | 머신 단위 (`HKCR`) — `regsvr32` 실행 시 관리자 권한 필요 |

DLL은 MSVC 런타임을 정적 링크하므로 Windows 기본 시스템 DLL(`KERNEL32.DLL`, `USER32.DLL`, `SHELL32.DLL`, `ADVAPI32.DLL`, `OLE32.DLL`, `CRYPT32.DLL`) 외에 별도의 외부 DLL 의존성이 없습니다.

### 보안 고려사항

- **저장 시 토큰 암호화** — Bearer 토큰은 디스크 기록 전 Windows DPAPI로 암호화되며 생성한 사용자 계정에서만 복호화됩니다.
- **전송 구간 암호화** — `reqwest`는 `https_only=true`로 빌드되어 TLS 통신만 허용하며 토큰은 `Authorization` 헤더로만 전송됩니다.
- **런타임 권한 분리** — 최초 1회 `regsvr32` 등록에만 관리자 권한이 필요하며, 실제 우클릭 핸들러는 일반 사용자 권한의 Explorer 프로세스 안에서 동작합니다.
- **텔레메트리 없음** — 설정된 워커 URL 외의 외부 네트워크 호출을 일절 수행하지 않습니다.
- **멱등성 보장** — `regsvr32 /u`를 여러 번 실행해도 안전하게 처리됩니다.

### 문제 해결

| 증상 | 주요 원인 | 해결 방법 |
|---|---|---|
| 우클릭 메뉴가 나타나지 않음 | Explorer가 이전 핸들러 캐시 유지 | Explorer 재시작 (`taskkill /IM explorer.exe /F && start explorer`) |
| `regsvr32`에서 `DllRegisterServer failed` 발생 | x64 DLL에 32비트 `regsvr32` 실행 또는 비관리자 실행 | 관리자 권한 프롬프트에서 `C:\Windows\System32\regsvr32.exe` 명시 실행 |
| 메뉴 클릭 시 아무 동작 없음 | `rdl` CLI 초기 인증 미완료 | `rdl auth login --token <토큰>` 1회 실행 |
| `server rejected credentials (401)` 알림 | 토큰 만료 또는 폐기 | 텔레그램 봇에서 새 토큰 발급 후 재로그인 |
| `regsvr32 /u` 후 DLL 파일 삭제 불가 | Explorer 프로세스에 아직 DLL이 매핑됨 | Explorer 재시작 또는 로그오프 후 삭제 |

### 기여하기

PR 제출 전 아래 세 가지 검사를 통과해야 합니다:

```bash
cargo fmt
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
```

### 감사의 글

- [`windows`](https://crates.io/crates/windows) — 공식 Win32 바인딩 및 `#[implement]` COM vtable 매크로
- [`reqwest`](https://crates.io/crates/reqwest) + [`rustls`](https://crates.io/crates/rustls) — TLS 전용 HTTP 클라이언트
- [`directories`](https://crates.io/crates/directories) — `rdl` CLI와 공유하는 설정 경로 탐색
- [`embed-resource`](https://crates.io/crates/embed-resource) — Win32 매니페스트 및 `.def` 리소스 컴파일

### 설치

> 관리자 권한은 레지스트리 등록 단계에서만 필요합니다.

1. [Releases](https://github.com/GHSFS/rdl-shell/releases) 페이지에서 `rdl-shell.dll` 다운로드
2. 영구 경로에 배치 (예: `C:\Tools\rdl-shell.dll`). 이후 이동 금지 — 레지스트리 항목이 경로에 바인딩됩니다.
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

`rdl-shell` 전용 설정은 없습니다. CLI가 작성한 `%APPDATA%\rdl\config.json`을 그대로 사용하므로, CLI에서 한 번 인증하면 세 도구 모두 동작합니다.

### 라이선스

MIT. [LICENSE](../../LICENSE) 참고.

### 면책

개인용 셸 확장입니다. 셸 확장은 `explorer.exe` 안에서 실행되므로, 수정된 버전이 크래시하면 Explorer가 재시작됩니다. 릴리스 빌드는 안정적이지만 설치는 본인 책임 하에 진행하세요.

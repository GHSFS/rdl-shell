# rdl-shell

> Windows Explorer shell extension for [remote-dl](https://github.com/GHSFS/remote-dl) — adds a "Send to remote-dl" entry to the right-click menu of `.url` (Internet Shortcut) files.

[![Build](https://github.com/GHSFS/rdl-shell/actions/workflows/build.yml/badge.svg)](https://github.com/GHSFS/rdl-shell/actions/workflows/build.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](./LICENSE)
[![Platform](https://img.shields.io/badge/platform-Windows%20x64-blue)](#installation)

[English](#english) · [한국어](#한국어) · [日本語](#日本語) · [中文](#中文) · [Русский](#русский) · [Tiếng Việt](#tiếng-việt) · [Türkçe](#türkçe) · [Deutsch](#deutsch) · [Español](#español) · [Português](#português)

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

### Repository layout

```
rdl-shell/
├── Cargo.toml                 Package manifest. Crate type = cdylib so the
│                              build emits a .dll instead of an .exe.
├── Cargo.lock                 Pinned dependency versions
├── rust-toolchain.toml        Toolchain pin (stable, x86_64-pc-windows-msvc)
├── .cargo/
│   └── config.toml            Build target + rustflags (static MSVC CRT)
├── .gitignore                 Excludes target/, secrets, IDE state
├── build.rs                   Compiles resources/app.rc and adds /DEF: to
│                              the linker arguments so the four COM exports
│                              are visible by name in the PE export table
├── src/
│   ├── lib.rs                 Crate root. Defines CLSID_RDL_SHELL, the
│   │                          REF_COUNT atomic for DllCanUnloadNow,
│   │                          DLL_HMODULE captured in DllMain for self-path
│   │                          resolution, and the four #[no_mangle] COM
│   │                          entry points
│   ├── factory.rs             IClassFactory implementation. CreateInstance
│   │                          returns a fresh RdlShellHandler per right-click;
│   │                          LockServer adjusts REF_COUNT
│   ├── handler.rs             Implements both shell interfaces. Initialize
│   │                          parses the CF_HDROP DROPFILES struct directly
│   │                          (no DragQueryFileW dependency); QueryContextMenu
│   │                          inserts the menu entry; InvokeCommand reads the
│   │                          .url file and POSTs to /api/dl;
│   │                          GetCommandString reports verb / help text
│   │                          for both ANSI and wide variants
│   ├── registry.rs            DllRegisterServer / DllUnregisterServer. Writes
│   │                          HKCR\CLSID\{...}\InprocServer32 and the
│   │                          HKCR\InternetShortcut\shellex\ContextMenuHandlers
│   │                          handler key; SHChangeNotify nudges Explorer to
│   │                          refresh its association cache
│   ├── client.rs              reqwest::blocking client; POST /api/dl with
│   │                          DPAPI-decrypted bearer token
│   ├── config.rs              Reads %APPDATA%\rdl\config.json written by the
│   │                          rdl CLI; DPAPI unprotect for the token field
│   └── error.rs               Crate-wide thiserror enum
├── tests/
│   └── smoke.rs               Build-only smoke check that the cdylib target
│                              is reachable
├── resources/
│   ├── app.rc                 Win32 resource script (manifest reference,
│   │                          VERSIONINFO with FILETYPE=VFT_DLL)
│   ├── app.manifest           OS compatibility declaration
│   └── rdl-shell.def          Module-definition file. Lists the four COM
│                              exports by name so regsvr32 can dlsym them
└── .github/workflows/
    ├── build.yml              CI: cargo build --release; verifies the four
    │                          required COM exports are present via
    │                          dumpbin /EXPORTS; uploads dll artifact;
    │                          cuts a Release on tag push
    └── test.yml               CI: cargo fmt --check, cargo clippy -D warnings,
                               cargo test --all-targets
```

### Compatibility

| Axis | Supported |
|---|---|
| Operating system | Windows 10 1809+ and Windows 11 |
| Architecture | x86_64 only (matches the dominant Explorer process bitness) |
| Rust toolchain | 1.75+ (`rust-toolchain.toml` pins stable) |
| Linker | MSVC (Visual Studio Build Tools 2022) |
| Registration | Per-machine (`HKCR`) — `regsvr32` requires elevation |

The DLL is statically linked against the MSVC runtime, so it has no DLL
dependencies beyond what Windows itself ships
(`KERNEL32.DLL`, `USER32.DLL`, `SHELL32.DLL`, `ADVAPI32.DLL`, `OLE32.DLL`,
`CRYPT32.DLL`).

### Security considerations

- **Token at rest** — the bearer token is DPAPI-wrapped before it touches
  disk and only the originating Windows user account can decrypt it.
- **Token in transit** — `reqwest` is built with `https_only=true`; the
  token is sent in the `Authorization` header over TLS, never in the URL.
- **No elevation at runtime** — only the one-time `regsvr32` registration
  step needs an elevated prompt. The actual context-menu handler runs in
  the unprivileged Explorer process.
- **No telemetry** — the DLL makes no network calls other than to the
  configured worker URL.
- **DllUnregisterServer is idempotent** — running `regsvr32 /u` twice is
  safe; the second call simply finds nothing to remove.

### Troubleshooting

| Symptom | Likely cause | Resolution |
|---|---|---|
| Menu entry never appears | Explorer cached the old handlers | Restart Explorer (`taskkill /IM explorer.exe /F && start explorer`) |
| `regsvr32` says "module loaded but DllRegisterServer failed" | running 32-bit `regsvr32` on x64 DLL | Use `C:\Windows\System32\regsvr32.exe` (the 64-bit one) explicitly |
| Menu entry is visible but click does nothing | config not written by the rdl CLI yet | Run `rdl auth login --token <token>` once |
| "server rejected credentials (401)" balloon | token revoked | Issue a new token via the Telegram bot |
| File can't be deleted after `regsvr32 /u` | Explorer still has the DLL mapped | Restart Explorer or sign out / back in |

### Contributing

This is a personal-use shell extension, but PRs that add support for `*` /
`Directory\Background` handlers, multi-file selection, or richer menu items
(submenu with destination folders) are welcome.

Before opening a PR:

```bash
cargo fmt
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
```

The CI runs the same three checks plus a `dumpbin /EXPORTS` step that
verifies the four COM entry points are visible by name.

### Acknowledgements

- [`windows`](https://crates.io/crates/windows) — official Microsoft Win32
  bindings; `#[implement]` macro generates the COM vtables.
- [`reqwest`](https://crates.io/crates/reqwest) +
  [`rustls`](https://crates.io/crates/rustls) — TLS-only HTTP client.
- [`directories`](https://crates.io/crates/directories) — config path
  resolution shared with the `rdl` CLI.
- [`embed-resource`](https://crates.io/crates/embed-resource) — compiles the
  Win32 manifest + `.def` file references into the PE.

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

---

## 日本語

### 概要

`rdl-shell.dll` は Windows シェルのコンテキストメニューハンドラで、
[`rdl`](https://github.com/GHSFS/remote-dl) CLI クライアントを Explorer に
統合します。登録後、`.url` インターネットショートカットファイルを右クリック
すると、新しいメニューエントリ — **Send to remote-dl** — が表示され、
ショートカットから URL を読み取ってエッジワーカーにキューイングします。

これは個人利用の `remote-dl` エコシステムの 3 つ目のツールです:

| ツール | 形態 | リポジトリ |
|---|---|---|
| `rdl.exe` | ターミナル CLI | [`remote-dl`](https://github.com/GHSFS/remote-dl) |
| `rdl-tray.exe` | システムトレイ + クリップボード監視 | [`rdl-tray`](https://github.com/GHSFS/rdl-tray) |
| **`rdl-shell.dll`** | **Explorer 右クリックメニュー** | このリポジトリ |

3 つすべてのツールが CLI が書き込んだ同じディスク設定ファイル
(`%APPDATA%\rdl\config.json`)を共有します。

### アーキテクチャ

DLL が実装する標準 COM エントリポイント 4 つ + シェルインターフェース 2 つ:

| シンボル | 役割 |
|---|---|
| `DllGetClassObject` | CLSID に対応する `IClassFactory` を返す |
| `DllCanUnloadNow` | 残存参照があるかを報告 |
| `DllRegisterServer` | `regsvr32` 呼び出し時にレジストリ項目を設定 |
| `DllUnregisterServer` | `regsvr32 /u` 呼び出し時に項目を削除 |
| `IShellExtInit` | Explorer から選択ファイルを受け取る |
| `IContextMenu` | メニュー項目を描画し `InvokeCommand` を処理 |

### プロジェクト構成

```
rdl-shell/
├── Cargo.toml             パッケージマニフェスト、crate-type=cdylib (DLL 出力)
├── rust-toolchain.toml    ツールチェイン固定 (stable / x86_64-pc-windows-msvc)
├── .cargo/config.toml     ビルドターゲット + 静的 MSVC CRT
├── build.rs               リソースコンパイル + /DEF: リンカ引数の注入
├── src/
│   ├── lib.rs             CLSID、REF_COUNT、DLL_HMODULE、4 つの COM エントリ
│   ├── factory.rs         IClassFactory 実装
│   ├── handler.rs         IShellExtInit + IContextMenu 実装
│   ├── registry.rs        DllRegisterServer / DllUnregisterServer
│   ├── client.rs          reqwest::blocking クライアント、POST /api/dl
│   ├── config.rs          %APPDATA%\rdl\config.json 読み取り + DPAPI 復号
│   └── error.rs           crate 全体のエラー型
├── tests/smoke.rs         ビルド検証用スモークテスト
├── resources/
│   ├── app.rc             Win32 リソーススクリプト (マニフェスト + VERSIONINFO)
│   ├── app.manifest       OS 互換性宣言
│   └── rdl-shell.def      COM エクスポートを名前で公開するモジュール定義
└── .github/workflows/
    ├── build.yml          cargo build + dumpbin /EXPORTS 検証 CI
    └── test.yml           fmt / clippy / test の CI
```

詳細なインストールおよびビルド手順は [English](#english) セクションを参照
してください。

### ライセンス

MIT。[LICENSE](./LICENSE) を参照。

---

## 中文

### 概述

`rdl-shell.dll` 是一个 Windows Shell 上下文菜单处理程序,将
[`rdl`](https://github.com/GHSFS/remote-dl) CLI 客户端集成到 Explorer 中。
注册后,在 `.url` Internet 快捷方式文件上右键单击会显示新菜单项 —
**Send to remote-dl** — 它会从快捷方式中读取 URL 并将其加入到所配置的边缘
工作器队列中。

这是个人用 `remote-dl` 生态的第三个工具:

| 工具 | 形式 | 仓库 |
|---|---|---|
| `rdl.exe` | 终端 CLI | [`remote-dl`](https://github.com/GHSFS/remote-dl) |
| `rdl-tray.exe` | 系统托盘 + 剪贴板监听 | [`rdl-tray`](https://github.com/GHSFS/rdl-tray) |
| **`rdl-shell.dll`** | **Explorer 右键菜单** | 本仓库 |

三个工具共享 CLI 写入的同一份磁盘配置文件
(`%APPDATA%\rdl\config.json`)。

### 架构

DLL 实现的 4 个标准 COM 入口点 + 2 个 Shell 接口:

| 符号 | 用途 |
|---|---|
| `DllGetClassObject` | 返回对应于本 CLSID 的 `IClassFactory` |
| `DllCanUnloadNow` | 报告是否仍有外部引用 |
| `DllRegisterServer` | `regsvr32` 调用时安装注册表项 |
| `DllUnregisterServer` | `regsvr32 /u` 调用时移除注册表项 |
| `IShellExtInit` | 从 Explorer 接收选定的文件 |
| `IContextMenu` | 渲染菜单项并响应 `InvokeCommand` |

### 项目结构

```
rdl-shell/
├── Cargo.toml             包清单,crate-type=cdylib(输出 DLL)
├── rust-toolchain.toml    工具链锁定 (stable / x86_64-pc-windows-msvc)
├── .cargo/config.toml     构建目标 + 静态 MSVC CRT
├── build.rs               编译资源 + 注入 /DEF: 链接器参数
├── src/
│   ├── lib.rs             CLSID、REF_COUNT、DLL_HMODULE、4 个 COM 入口
│   ├── factory.rs         IClassFactory 实现
│   ├── handler.rs         IShellExtInit + IContextMenu 实现
│   ├── registry.rs        DllRegisterServer / DllUnregisterServer
│   ├── client.rs          reqwest::blocking 客户端,POST /api/dl
│   ├── config.rs          读取 %APPDATA%\rdl\config.json + DPAPI 解密
│   └── error.rs           crate 范围错误类型
├── tests/smoke.rs         构建验证冒烟测试
├── resources/
│   ├── app.rc             Win32 资源脚本(清单 + VERSIONINFO)
│   ├── app.manifest       OS 兼容性声明
│   └── rdl-shell.def      按名称公开 COM 导出的模块定义文件
└── .github/workflows/
    ├── build.yml          cargo build + dumpbin /EXPORTS 验证 CI
    └── test.yml           fmt / clippy / test 的 CI
```

详细的安装和构建指南请参阅 [English](#english) 部分。

### 许可证

MIT。详见 [LICENSE](./LICENSE)。

---

## Русский

### Обзор

`rdl-shell.dll` — обработчик контекстного меню Windows Shell, который
интегрирует CLI-клиент [`rdl`](https://github.com/GHSFS/remote-dl) в
Explorer. После регистрации правый клик по файлу интернет-ярлыка `.url`
показывает новый пункт меню — **Send to remote-dl** — который читает URL
из ярлыка и ставит её в очередь на настроенном edge-worker.

Это третий инструмент персональной экосистемы `remote-dl`:

| Инструмент | Тип | Репозиторий |
|---|---|---|
| `rdl.exe` | терминальный CLI | [`remote-dl`](https://github.com/GHSFS/remote-dl) |
| `rdl-tray.exe` | системный трей + слежение за буфером обмена | [`rdl-tray`](https://github.com/GHSFS/rdl-tray) |
| **`rdl-shell.dll`** | **контекстное меню Explorer** | этот репозиторий |

Все три инструмента используют один и тот же файл конфигурации, написанный
CLI (`%APPDATA%\rdl\config.json`).

### Архитектура

DLL реализует четыре стандартных COM-входа и два Shell-интерфейса:

| Символ | Назначение |
|---|---|
| `DllGetClassObject` | Возвращает `IClassFactory` для нашего CLSID |
| `DllCanUnloadNow` | Сообщает, удерживают ли DLL внешние ссылки |
| `DllRegisterServer` | Создаёт записи реестра (вызывается `regsvr32`) |
| `DllUnregisterServer` | Удаляет записи (`regsvr32 /u`) |
| `IShellExtInit` | Получает выделение файлов от Explorer |
| `IContextMenu` | Рендерит пункт меню и обрабатывает `InvokeCommand` |

### Структура проекта

```
rdl-shell/
├── Cargo.toml             Манифест пакета, crate-type=cdylib (вывод DLL)
├── rust-toolchain.toml    Закрепление toolchain (stable / x86_64-pc-windows-msvc)
├── .cargo/config.toml     Цель сборки + статическая MSVC CRT
├── build.rs               Компиляция ресурсов + добавление /DEF: к компоновщику
├── src/
│   ├── lib.rs             CLSID, REF_COUNT, DLL_HMODULE, 4 точки входа COM
│   ├── factory.rs         Реализация IClassFactory
│   ├── handler.rs         Реализация IShellExtInit + IContextMenu
│   ├── registry.rs        DllRegisterServer / DllUnregisterServer
│   ├── client.rs          Клиент reqwest::blocking, POST /api/dl
│   ├── config.rs          Чтение %APPDATA%\rdl\config.json + DPAPI расшифровка
│   └── error.rs           Тип ошибки уровня крейта
├── tests/smoke.rs         Smoke-тест проверки сборки
├── resources/
│   ├── app.rc             Win32 resource script (manifest + VERSIONINFO)
│   ├── app.manifest       Декларация совместимости с ОС
│   └── rdl-shell.def      Module-definition: имена COM-экспортов
└── .github/workflows/
    ├── build.yml          CI cargo build + проверка dumpbin /EXPORTS
    └── test.yml           CI fmt / clippy / test
```

Подробные инструкции по установке и сборке см. в разделе [English](#english).

### Лицензия

MIT. См. [LICENSE](./LICENSE).

---

## Tiếng Việt

### Tổng quan

`rdl-shell.dll` là một trình xử lý menu ngữ cảnh Windows Shell, tích hợp
client CLI [`rdl`](https://github.com/GHSFS/remote-dl) vào Explorer. Sau
khi đăng ký, click chuột phải vào tệp Internet Shortcut `.url` sẽ hiển thị
một mục menu mới — **Send to remote-dl** — đọc URL từ shortcut và đưa nó
vào hàng đợi trên edge worker đã cấu hình.

Đây là công cụ thứ ba trong hệ sinh thái cá nhân `remote-dl`:

| Công cụ | Hình thức | Repository |
|---|---|---|
| `rdl.exe` | CLI terminal | [`remote-dl`](https://github.com/GHSFS/remote-dl) |
| `rdl-tray.exe` | system tray + theo dõi clipboard | [`rdl-tray`](https://github.com/GHSFS/rdl-tray) |
| **`rdl-shell.dll`** | **menu chuột phải Explorer** | repository này |

Cả ba công cụ đều chia sẻ cùng một tệp cấu hình trên đĩa do CLI viết
(`%APPDATA%\rdl\config.json`).

### Kiến trúc

DLL triển khai 4 entry point COM chuẩn + 2 interface Shell:

| Ký hiệu | Mục đích |
|---|---|
| `DllGetClassObject` | Trả về `IClassFactory` cho CLSID của chúng ta |
| `DllCanUnloadNow` | Báo cáo có tham chiếu nào còn đang giữ DLL không |
| `DllRegisterServer` | Cài đặt mục registry khi `regsvr32` gọi |
| `DllUnregisterServer` | Gỡ bỏ mục registry khi `regsvr32 /u` gọi |
| `IShellExtInit` | Nhận file được chọn từ Explorer |
| `IContextMenu` | Vẽ mục menu và xử lý `InvokeCommand` |

### Cấu trúc dự án

```
rdl-shell/
├── Cargo.toml             Manifest gói, crate-type=cdylib (xuất DLL)
├── rust-toolchain.toml    Cố định toolchain (stable / x86_64-pc-windows-msvc)
├── .cargo/config.toml     Mục tiêu build + MSVC CRT tĩnh
├── build.rs               Biên dịch resource + thêm /DEF: vào linker
├── src/
│   ├── lib.rs             CLSID, REF_COUNT, DLL_HMODULE, 4 entry point COM
│   ├── factory.rs         Triển khai IClassFactory
│   ├── handler.rs         Triển khai IShellExtInit + IContextMenu
│   ├── registry.rs        DllRegisterServer / DllUnregisterServer
│   ├── client.rs          Client reqwest::blocking, POST /api/dl
│   ├── config.rs          Đọc %APPDATA%\rdl\config.json + giải mã DPAPI
│   └── error.rs           Kiểu lỗi của crate
├── tests/smoke.rs         Smoke test xác nhận build
├── resources/
│   ├── app.rc             Resource script Win32 (manifest + VERSIONINFO)
│   ├── app.manifest       Khai báo tương thích OS
│   └── rdl-shell.def      Module-definition công bố COM export theo tên
└── .github/workflows/
    ├── build.yml          CI cargo build + kiểm tra dumpbin /EXPORTS
    └── test.yml           CI fmt / clippy / test
```

Hướng dẫn cài đặt và build chi tiết xem ở phần [English](#english).

### Giấy phép

MIT. Xem [LICENSE](./LICENSE).

---

## Türkçe

### Genel Bakış

`rdl-shell.dll`, [`rdl`](https://github.com/GHSFS/remote-dl) CLI istemcisini
Explorer'a entegre eden bir Windows Shell bağlam menüsü işleyicisidir.
Kaydedildikten sonra, bir `.url` İnternet Kısayolu dosyasına sağ
tıkladığınızda yeni bir menü öğesi — **Send to remote-dl** — görünür;
kısayoldan URL'yi okur ve yapılandırılmış edge worker'a kuyruğa alır.

Bu, kişisel `remote-dl` ekosisteminin üçüncü aracıdır:

| Araç | Biçim | Depo |
|---|---|---|
| `rdl.exe` | terminal CLI | [`remote-dl`](https://github.com/GHSFS/remote-dl) |
| `rdl-tray.exe` | sistem tepsisi + pano gözleyici | [`rdl-tray`](https://github.com/GHSFS/rdl-tray) |
| **`rdl-shell.dll`** | **Explorer sağ tık menüsü** | bu depo |

Üç araç da CLI tarafından yazılan aynı disk yapılandırma dosyasını paylaşır
(`%APPDATA%\rdl\config.json`).

### Mimari

DLL, dört standart COM giriş noktasını ve iki Shell arayüzünü uygular:

| Sembol | Amaç |
|---|---|
| `DllGetClassObject` | CLSID için `IClassFactory` döndürür |
| `DllCanUnloadNow` | Bekleyen referansların DLL'i tutup tutmadığını bildirir |
| `DllRegisterServer` | `regsvr32` çağrıldığında kayıt defteri girdilerini yükler |
| `DllUnregisterServer` | `regsvr32 /u` çağrıldığında girdileri kaldırır |
| `IShellExtInit` | Explorer'dan dosya seçimini alır |
| `IContextMenu` | Menü öğesini çizer ve `InvokeCommand`'a yanıt verir |

### Proje yapısı

```
rdl-shell/
├── Cargo.toml             Paket manifesti, crate-type=cdylib (DLL çıktısı)
├── rust-toolchain.toml    Toolchain sabitleme (stable / x86_64-pc-windows-msvc)
├── .cargo/config.toml     Derleme hedefi + statik MSVC CRT
├── build.rs               Kaynakları derler + bağlayıcıya /DEF: ekler
├── src/
│   ├── lib.rs             CLSID, REF_COUNT, DLL_HMODULE, 4 COM giriş noktası
│   ├── factory.rs         IClassFactory uygulaması
│   ├── handler.rs         IShellExtInit + IContextMenu uygulaması
│   ├── registry.rs        DllRegisterServer / DllUnregisterServer
│   ├── client.rs          reqwest::blocking istemcisi, POST /api/dl
│   ├── config.rs          %APPDATA%\rdl\config.json okuma + DPAPI çözme
│   └── error.rs           Crate genelinde hata türü
├── tests/smoke.rs         Derleme doğrulama smoke testi
├── resources/
│   ├── app.rc             Win32 resource script (manifest + VERSIONINFO)
│   ├── app.manifest       OS uyumluluk bildirimi
│   └── rdl-shell.def      COM dışa aktarımlarını adıyla yayınlayan modül tanımı
└── .github/workflows/
    ├── build.yml          cargo build + dumpbin /EXPORTS doğrulama CI
    └── test.yml           fmt / clippy / test CI
```

Ayrıntılı kurulum ve derleme talimatları için [English](#english) bölümüne
bakın.

### Lisans

MIT. [LICENSE](./LICENSE) dosyasına bakın.

---

## Deutsch

### Überblick

`rdl-shell.dll` ist ein Windows-Shell-Kontextmenü-Handler, der den
CLI-Client [`rdl`](https://github.com/GHSFS/remote-dl) in den Explorer
integriert. Nach der Registrierung zeigt ein Rechtsklick auf eine
`.url`-Internet-Verknüpfungsdatei einen neuen Menüeintrag — **Send to
remote-dl** — der die URL aus der Verknüpfung liest und sie auf dem
konfigurierten Edge-Worker einreiht.

Dies ist das dritte Werkzeug des persönlichen `remote-dl`-Ökosystems:

| Werkzeug | Form | Repository |
|---|---|---|
| `rdl.exe` | Terminal-CLI | [`remote-dl`](https://github.com/GHSFS/remote-dl) |
| `rdl-tray.exe` | Systemtray + Zwischenablage-Watcher | [`rdl-tray`](https://github.com/GHSFS/rdl-tray) |
| **`rdl-shell.dll`** | **Explorer-Rechtsklickmenü** | dieses Repository |

Alle drei Werkzeuge teilen sich dieselbe von der CLI geschriebene
Konfigurationsdatei (`%APPDATA%\rdl\config.json`).

### Architektur

Die DLL implementiert die vier standardmäßigen COM-Einstiegspunkte und zwei
Shell-Schnittstellen:

| Symbol | Zweck |
|---|---|
| `DllGetClassObject` | Gibt die `IClassFactory` für unsere CLSID zurück |
| `DllCanUnloadNow` | Meldet, ob ausstehende Objekte die DLL festhalten |
| `DllRegisterServer` | Installiert Registrierungseinträge (von `regsvr32` aufgerufen) |
| `DllUnregisterServer` | Entfernt Einträge (von `regsvr32 /u` aufgerufen) |
| `IShellExtInit` | Empfängt die Dateiauswahl vom Explorer |
| `IContextMenu` | Rendert den Menüeintrag und reagiert auf `InvokeCommand` |

### Projektstruktur

```
rdl-shell/
├── Cargo.toml             Paketmanifest, crate-type=cdylib (DLL-Ausgabe)
├── rust-toolchain.toml    Toolchain-Pin (stable / x86_64-pc-windows-msvc)
├── .cargo/config.toml     Build-Ziel + statische MSVC-CRT
├── build.rs               Kompiliert Ressourcen + fügt /DEF: zum Linker hinzu
├── src/
│   ├── lib.rs             CLSID, REF_COUNT, DLL_HMODULE, 4 COM-Einstiegspunkte
│   ├── factory.rs         IClassFactory-Implementierung
│   ├── handler.rs         IShellExtInit + IContextMenu-Implementierung
│   ├── registry.rs        DllRegisterServer / DllUnregisterServer
│   ├── client.rs          reqwest::blocking-Client, POST /api/dl
│   ├── config.rs          %APPDATA%\rdl\config.json lesen + DPAPI entschlüsseln
│   └── error.rs           Crate-weiter Fehlertyp
├── tests/smoke.rs         Build-Verifikations-Smoke-Test
├── resources/
│   ├── app.rc             Win32-Resource-Script (Manifest + VERSIONINFO)
│   ├── app.manifest       Betriebssystem-Kompatibilitätserklärung
│   └── rdl-shell.def      Modul-Definition: COM-Exports nach Namen
└── .github/workflows/
    ├── build.yml          CI cargo build + dumpbin /EXPORTS-Verifikation
    └── test.yml           CI fmt / clippy / test
```

Ausführliche Installations- und Build-Anweisungen findest du im Abschnitt
[English](#english).

### Lizenz

MIT. Siehe [LICENSE](./LICENSE).

---

## Español

### Descripción general

`rdl-shell.dll` es un manejador de menú contextual de Windows Shell que
integra el cliente CLI [`rdl`](https://github.com/GHSFS/remote-dl) en
Explorer. Una vez registrado, al hacer clic derecho sobre un archivo de
acceso directo de Internet `.url` aparece una nueva entrada de menú —
**Send to remote-dl** — que lee la URL del acceso directo y la encola en el
edge worker configurado.

Esta es la tercera herramienta del ecosistema personal `remote-dl`:

| Herramienta | Forma | Repositorio |
|---|---|---|
| `rdl.exe` | CLI de terminal | [`remote-dl`](https://github.com/GHSFS/remote-dl) |
| `rdl-tray.exe` | bandeja del sistema + vigilancia del portapapeles | [`rdl-tray`](https://github.com/GHSFS/rdl-tray) |
| **`rdl-shell.dll`** | **menú contextual de Explorer** | este repositorio |

Las tres herramientas comparten el mismo archivo de configuración en disco
escrito por la CLI (`%APPDATA%\rdl\config.json`).

### Arquitectura

El DLL implementa los cuatro puntos de entrada COM estándar y dos
interfaces de Shell:

| Símbolo | Propósito |
|---|---|
| `DllGetClassObject` | Devuelve el `IClassFactory` para nuestro CLSID |
| `DllCanUnloadNow` | Informa si quedan referencias que retengan el DLL |
| `DllRegisterServer` | Instala entradas del registro (llamado por `regsvr32`) |
| `DllUnregisterServer` | Elimina las entradas (llamado por `regsvr32 /u`) |
| `IShellExtInit` | Recibe la selección de archivos desde Explorer |
| `IContextMenu` | Renderiza la entrada del menú y reacciona a `InvokeCommand` |

### Estructura del proyecto

```
rdl-shell/
├── Cargo.toml             Manifiesto del paquete, crate-type=cdylib (salida DLL)
├── rust-toolchain.toml    Fijación del toolchain (stable / x86_64-pc-windows-msvc)
├── .cargo/config.toml     Destino de build + CRT estática de MSVC
├── build.rs               Compila recursos + añade /DEF: al linker
├── src/
│   ├── lib.rs             CLSID, REF_COUNT, DLL_HMODULE, 4 puntos de entrada COM
│   ├── factory.rs         Implementación de IClassFactory
│   ├── handler.rs         Implementación de IShellExtInit + IContextMenu
│   ├── registry.rs        DllRegisterServer / DllUnregisterServer
│   ├── client.rs          Cliente reqwest::blocking, POST /api/dl
│   ├── config.rs          Lectura de %APPDATA%\rdl\config.json + descifrado DPAPI
│   └── error.rs           Tipo de error del crate
├── tests/smoke.rs         Smoke test de verificación de build
├── resources/
│   ├── app.rc             Script de recursos Win32 (manifiesto + VERSIONINFO)
│   ├── app.manifest       Declaración de compatibilidad de OS
│   └── rdl-shell.def      Module-definition: exports COM por nombre
└── .github/workflows/
    ├── build.yml          CI cargo build + verificación dumpbin /EXPORTS
    └── test.yml           CI fmt / clippy / test
```

Para instrucciones detalladas de instalación y compilación, consulta la
sección [English](#english).

### Licencia

MIT. Consulta [LICENSE](./LICENSE).

---

## Português

### Visão geral

`rdl-shell.dll` é um manipulador de menu de contexto do Windows Shell que
integra o cliente CLI [`rdl`](https://github.com/GHSFS/remote-dl) no
Explorer. Após o registro, clicar com o botão direito em um arquivo de
atalho da Internet `.url` mostra uma nova entrada de menu — **Send to
remote-dl** — que lê a URL do atalho e a coloca na fila do edge worker
configurado.

Esta é a terceira ferramenta do ecossistema pessoal `remote-dl`:

| Ferramenta | Formato | Repositório |
|---|---|---|
| `rdl.exe` | CLI de terminal | [`remote-dl`](https://github.com/GHSFS/remote-dl) |
| `rdl-tray.exe` | bandeja do sistema + monitor da área de transferência | [`rdl-tray`](https://github.com/GHSFS/rdl-tray) |
| **`rdl-shell.dll`** | **menu de clique direito do Explorer** | este repositório |

Todas as três ferramentas compartilham o mesmo arquivo de configuração em
disco escrito pela CLI (`%APPDATA%\rdl\config.json`).

### Arquitetura

O DLL implementa os quatro pontos de entrada COM padrão e duas interfaces
de Shell:

| Símbolo | Propósito |
|---|---|
| `DllGetClassObject` | Retorna o `IClassFactory` para nosso CLSID |
| `DllCanUnloadNow` | Informa se há referências externas mantendo o DLL |
| `DllRegisterServer` | Instala as entradas do registro (chamado por `regsvr32`) |
| `DllUnregisterServer` | Remove as entradas (chamado por `regsvr32 /u`) |
| `IShellExtInit` | Recebe a seleção de arquivos do Explorer |
| `IContextMenu` | Desenha a entrada do menu e responde a `InvokeCommand` |

### Estrutura do projeto

```
rdl-shell/
├── Cargo.toml             Manifesto do pacote, crate-type=cdylib (saída DLL)
├── rust-toolchain.toml    Fixação do toolchain (stable / x86_64-pc-windows-msvc)
├── .cargo/config.toml     Alvo de build + CRT estática do MSVC
├── build.rs               Compila recursos + adiciona /DEF: ao linker
├── src/
│   ├── lib.rs             CLSID, REF_COUNT, DLL_HMODULE, 4 entry points COM
│   ├── factory.rs         Implementação de IClassFactory
│   ├── handler.rs         Implementação de IShellExtInit + IContextMenu
│   ├── registry.rs        DllRegisterServer / DllUnregisterServer
│   ├── client.rs          Cliente reqwest::blocking, POST /api/dl
│   ├── config.rs          Leitura de %APPDATA%\rdl\config.json + DPAPI
│   └── error.rs           Tipo de erro do crate
├── tests/smoke.rs         Smoke test de verificação de build
├── resources/
│   ├── app.rc             Script de recursos Win32 (manifesto + VERSIONINFO)
│   ├── app.manifest       Declaração de compatibilidade de OS
│   └── rdl-shell.def      Module-definition: exports COM por nome
└── .github/workflows/
    ├── build.yml          CI cargo build + verificação dumpbin /EXPORTS
    └── test.yml           CI fmt / clippy / test
```

Para instruções detalhadas de instalação e compilação, veja a seção
[English](#english).

### Licença

MIT. Veja [LICENSE](./LICENSE).

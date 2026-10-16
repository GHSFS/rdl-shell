<div align="center">

<p align="center">
  <strong>🇺🇸 English</strong> · <a href="./docs/locales/ko.md">🇰🇷 한국어</a> · <a href="./docs/locales/zh-CN.md">🇨🇳 中文</a> · <a href="./docs/locales/es.md">🇪🇸 Español</a> · <a href="./docs/locales/hi.md">🇮🇳 हिन्दी</a><br />
  <a href="./docs/locales/ar.md">🇸🇦 العربية</a> · <a href="./docs/locales/pt-BR.md">🇧🇷 Português</a> · <a href="./docs/locales/ru.md">🇷🇺 Русский</a> · <a href="./docs/locales/fr.md">🇫🇷 Français</a> · <a href="./docs/locales/id.md">🇮🇩 Bahasa Indonesia</a>
</p>

# rdl-shell

> Windows Explorer shell extension for [remote-dl](https://github.com/GHSFS/remote-dl) — adds a "Send to remote-dl" entry to the right-click menu of `.url` (Internet Shortcut) files.

[![Build](https://github.com/GHSFS/rdl-shell/actions/workflows/build.yml/badge.svg)](https://github.com/GHSFS/rdl-shell/actions/workflows/build.yml)
[![Tests](https://github.com/GHSFS/rdl-shell/actions/workflows/test.yml/badge.svg)](https://github.com/GHSFS/rdl-shell/actions/workflows/test.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](./LICENSE)
[![Platform](https://img.shields.io/badge/platform-Windows%20x64-blue)](#installation)

</div>

---

### Overview

`rdl-shell.dll` is a Windows Shell context-menu handler that integrates the
[`rdl`](https://github.com/GHSFS/remote-dl) CLI client into Explorer. Once
registered, right-clicking on a `.url` Internet Shortcut file shows a new
menu entry — **Send to remote-dl** (or **Send N shortcuts to remote-dl**) — that reads the URL from the shortcut
and queues it on the configured edge worker.

#### ✨ Key Features (v0.2.0)
- **Multi-File Selection**: Select multiple `.url` files at once; queues all shortcuts in one click.
- **Zero Explorer Freezing**: Non-blocking asynchronous worker thread with RAII `DllLockGuard` lifetime pinning.
- **Universal Shortcut Encoding**: Automatic parsing for UTF-8, UTF-16LE (with/without BOM), and UTF-16BE shortcuts.
- **Strict URL Validation**: Verifies `http://` and `https://` schemas prior to dispatch.
- **Connection Pooling**: Cached `reqwest` client with keep-alive and DNS caching for sub-millisecond execution.

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

# Changelog

All notable changes to `rdl-shell` are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [0.2.0] - 2026-10-17

### Added
- **Multi-File Selection**: Support selecting multiple `.url` files at once; queues all shortcuts in one click with dynamic label ("Send N shortcuts to remote-dl").
- **Non-Blocking Async Dispatch**: Offloads network queuing to an asynchronous worker thread in `InvokeCommand`, preventing Explorer UI freezing (0ms UI return).
- **RAII DllLockGuard**: Safely pins the COM DLL in memory during background dispatch via `crate::lock()` and `crate::unlock()`.
- **Universal Shortcut Encoding**: Automatic detection and decoding for UTF-8, UTF-16LE (with/without BOM), and UTF-16BE files, with case-insensitive `url=` key parsing.
- **Strict URL Scheme Validation**: Enforces `http://` and `https://` schemes, rejecting non-standard/malicious schemes before network dispatch.
- **Connection Pooling**: Reuses cached `reqwest` client with keep-alive and DNS caching for sub-millisecond dispatch.
- **Enhanced CI Release Packaging**: Automates zip archive creation (`rdl-shell-v*.-x64.zip`) and `SHA256SUMS.txt` generation in GitHub Actions.

## [0.1.0] - 2026-10-16

### Added
- Initial Windows Explorer shell extension for [remote-dl](https://github.com/GHSFS/remote-dl).
- Implements `IShellExtInit` + `IContextMenu` via the `windows` crate's COM macros.
- `IClassFactory` that hands out one handler per right-click.
- Self-registration via `DllRegisterServer` — installs registry keys under
  `HKCR\CLSID\{6e0d3a1c-...}` and
  `HKCR\InternetShortcut\shellex\ContextMenuHandlers\rdl-shell`.
- Self-removal via `DllUnregisterServer` (`regsvr32 /u rdl-shell.dll`).
- `.url` (Internet Shortcut) file parser — extracts the `URL=` line and
  POSTs it to `<worker>/api/dl`.
- Shares config storage with the `rdl` CLI (`%APPDATA%\rdl\config.json`).

### Planned
- Multi-file selection (currently uses only the first file).
- Optional submenu with destination folder shortcuts.
- Toast notification on success/failure (would need a sibling tray process).

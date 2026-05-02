# Changelog

All notable changes to `rdl-shell` are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

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

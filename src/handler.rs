//! Implements the two COM interfaces Explorer needs to add a context-menu
//! item: `IShellExtInit` (gets the file selection from Explorer) and
//! `IContextMenu` (renders the menu entry and reacts to clicks).

#![allow(non_snake_case)]

use crate::client;
use crate::config::Config;
use std::sync::Mutex;
use windows::core::{implement, PCWSTR, PSTR};
use windows::Win32::Foundation::{E_FAIL, E_INVALIDARG, HGLOBAL};
use windows::Win32::System::Com::{
    IDataObject, DVASPECT_CONTENT, FORMATETC, STGMEDIUM, TYMED_HGLOBAL,
};
use windows::Win32::System::Memory::{GlobalLock, GlobalUnlock};
use windows::Win32::System::Ole::CF_HDROP;
use windows::Win32::System::Registry::HKEY;
use windows::Win32::UI::Shell::{
    IContextMenu, IContextMenu_Impl, IShellExtInit, IShellExtInit_Impl,
    CMINVOKECOMMANDINFO,
};
use windows::Win32::UI::WindowsAndMessaging::{AppendMenuW, HMENU, MF_BYPOSITION, MF_STRING};

const MENU_TEXT: &str = "Send to remote-dl";
const VERB_ANSI: &str = "rdlshell.send";
const HELP_TEXT: &str = "Queue this URL via the remote-dl backend.";

/// One handler instance per right-click. Carries the file path Explorer
/// selected.
#[implement(IShellExtInit, IContextMenu)]
#[derive(Default)]
pub struct RdlShellHandler {
    selected: Mutex<Option<String>>,
}

impl IShellExtInit_Impl for RdlShellHandler_Impl {
    fn Initialize(
        &self,
        _pidlfolder: *const windows::Win32::UI::Shell::Common::ITEMIDLIST,
        pdtobj: Option<&IDataObject>,
        _hkey_progid: HKEY,
    ) -> windows::core::Result<()> {
        let data = pdtobj.ok_or_else(|| windows::core::Error::from(E_INVALIDARG))?;
        let path = unsafe { read_first_path(data) }?;
        if let Ok(mut slot) = self.selected.lock() {
            *slot = Some(path);
        }
        Ok(())
    }
}

impl IContextMenu_Impl for RdlShellHandler_Impl {
    fn QueryContextMenu(
        &self,
        hmenu: HMENU,
        indexmenu: u32,
        idcmdfirst: u32,
        _idcmdlast: u32,
        _uflags: u32,
    ) -> windows::core::Result<()> {
        unsafe {
            let label = wide(MENU_TEXT);
            AppendMenuW(
                hmenu,
                MF_STRING | MF_BYPOSITION,
                idcmdfirst as usize,
                PCWSTR(label.as_ptr()),
            )?;
            let _ = indexmenu;
            // The HRESULT.SeverityCode encoded as the number of commands
            // added is conveyed by the trait wrapper in this windows-rs
            // version; returning Ok(()) is sufficient here.
            Ok(())
        }
    }

    fn InvokeCommand(&self, pici: *const CMINVOKECOMMANDINFO) -> windows::core::Result<()> {
        if pici.is_null() {
            return Err(E_INVALIDARG.into());
        }
        let path = self
            .selected
            .lock()
            .ok()
            .and_then(|g| g.clone())
            .ok_or_else(|| windows::core::Error::from(E_FAIL))?;

        let url = read_url_from_url_file(&path).ok_or_else(|| windows::core::Error::from(E_FAIL))?;

        // Best-effort dispatch — we deliberately don't surface UI here. A
        // future revision could pop a tray balloon via a sibling process.
        let cfg = Config::load().map_err(|_| windows::core::Error::from(E_FAIL))?;
        client::queue(&cfg, &url).map_err(|_| windows::core::Error::from(E_FAIL))?;
        Ok(())
    }

    fn GetCommandString(
        &self,
        _idcmd: usize,
        utype: u32,
        _preserved: *const u32,
        pszname: PSTR,
        cchmax: u32,
    ) -> windows::core::Result<()> {
        unsafe {
            const GCS_VERBA: u32 = 0x00000000;
            const GCS_VERBW: u32 = 0x00000004;
            const GCS_HELPTEXTA: u32 = 0x00000001;
            const GCS_HELPTEXTW: u32 = 0x00000005;

            match utype {
                GCS_VERBA => write_ansi(pszname.0, cchmax, VERB_ANSI),
                GCS_VERBW => write_wide(pszname.0 as *mut u16, cchmax, VERB_ANSI),
                GCS_HELPTEXTA => write_ansi(pszname.0, cchmax, HELP_TEXT),
                GCS_HELPTEXTW => write_wide(pszname.0 as *mut u16, cchmax, HELP_TEXT),
                _ => Ok(()),
            }
        }
    }
}

unsafe fn write_ansi(dst: *mut u8, cap: u32, text: &str) -> windows::core::Result<()> {
    if dst.is_null() || cap == 0 {
        return Ok(());
    }
    let bytes = text.as_bytes();
    let n = bytes.len().min(cap as usize - 1);
    std::ptr::copy_nonoverlapping(bytes.as_ptr(), dst, n);
    *dst.add(n) = 0;
    Ok(())
}

unsafe fn write_wide(dst: *mut u16, cap: u32, text: &str) -> windows::core::Result<()> {
    if dst.is_null() || cap == 0 {
        return Ok(());
    }
    let buf: Vec<u16> = text.encode_utf16().collect();
    let n = buf.len().min(cap as usize - 1);
    std::ptr::copy_nonoverlapping(buf.as_ptr(), dst, n);
    *dst.add(n) = 0;
    Ok(())
}

/// Reads the first file path from an `IDataObject` carrying `CF_HDROP` data.
///
/// We parse the `DROPFILES` struct directly rather than calling
/// `DragQueryFileW`, because the latter's signature has churned across
/// `windows-rs` versions and a manual parse keeps the build hermetic.
unsafe fn read_first_path(data: &IDataObject) -> windows::core::Result<String> {
    let mut fmt = FORMATETC {
        cfFormat: CF_HDROP.0 as u16,
        ptd: std::ptr::null_mut(),
        dwAspect: DVASPECT_CONTENT.0 as u32,
        lindex: -1,
        tymed: TYMED_HGLOBAL.0 as u32,
    };
    let stg: STGMEDIUM = data.GetData(&fmt)?;

    let hglobal = HGLOBAL(stg.u.hGlobal.0);
    let base = GlobalLock(hglobal) as *const u8;
    if base.is_null() {
        return Err(E_FAIL.into());
    }

    // DROPFILES { pFiles: u32, pt: POINT, fNC: BOOL, fWide: BOOL }
    // The file list starts at base + pFiles, then is a sequence of
    // null-terminated strings (wide if fWide != 0) ending with a double-NUL.
    let p_files = *(base as *const u32);
    let f_wide = *(base.add(16) as *const i32) != 0;
    let list = base.add(p_files as usize);

    let result = if f_wide {
        let p = list as *const u16;
        let mut len = 0usize;
        while *p.add(len) != 0 && len < 0x4000 {
            len += 1;
        }
        let slice = std::slice::from_raw_parts(p, len);
        String::from_utf16_lossy(slice)
    } else {
        let mut len = 0usize;
        while *list.add(len) != 0 && len < 0x4000 {
            len += 1;
        }
        let slice = std::slice::from_raw_parts(list, len);
        String::from_utf8_lossy(slice).into_owned()
    };

    let _ = GlobalUnlock(hglobal);
    if result.is_empty() {
        Err(E_FAIL.into())
    } else {
        Ok(result)
    }
}

/// `.url` files are simple INI-format text. The URL we want is the
/// `URL=` line under `[InternetShortcut]`.
fn read_url_from_url_file(path: &str) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    for line in content.lines() {
        if let Some(rest) = line.trim().strip_prefix("URL=") {
            return Some(rest.trim().to_string());
        }
    }
    None
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_internet_shortcut_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.url");
        std::fs::write(
            &path,
            b"[InternetShortcut]\r\nURL=https://example.com/x\r\n",
        )
        .unwrap();
        assert_eq!(
            read_url_from_url_file(path.to_str().unwrap()),
            Some("https://example.com/x".to_string())
        );
    }

    #[test]
    fn returns_none_when_no_url_line() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nope.url");
        std::fs::write(&path, b"[Some]\r\nKey=value\r\n").unwrap();
        assert!(read_url_from_url_file(path.to_str().unwrap()).is_none());
    }
}

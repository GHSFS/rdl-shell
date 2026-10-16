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
    IContextMenu, IContextMenu_Impl, IShellExtInit, IShellExtInit_Impl, CMINVOKECOMMANDINFO,
};
use windows::Win32::UI::WindowsAndMessaging::{AppendMenuW, HMENU, MF_BYPOSITION, MF_STRING};

const VERB_ANSI: &str = "rdlshell.send";
const HELP_TEXT: &str = "Queue this URL via the remote-dl backend.";

/// RAII guard to keep DLL pinned in memory during asynchronous background dispatch.
struct DllLockGuard;

impl DllLockGuard {
    fn new() -> Self {
        crate::lock();
        Self
    }
}

impl Drop for DllLockGuard {
    fn drop(&mut self) {
        crate::unlock();
    }
}

/// One handler instance per right-click. Carries all selected file paths.
#[implement(IShellExtInit, IContextMenu)]
#[derive(Default)]
pub struct RdlShellHandler {
    selected: Mutex<Vec<String>>,
}

impl IShellExtInit_Impl for RdlShellHandler_Impl {
    fn Initialize(
        &self,
        _pidlfolder: *const windows::Win32::UI::Shell::Common::ITEMIDLIST,
        pdtobj: Option<&IDataObject>,
        _hkey_progid: HKEY,
    ) -> windows::core::Result<()> {
        let data = pdtobj.ok_or_else(|| windows::core::Error::from(E_INVALIDARG))?;
        let paths = unsafe { read_selected_paths(data) }?;
        if let Ok(mut slot) = self.selected.lock() {
            *slot = paths;
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
            let count = self.selected.lock().map(|s| s.len()).unwrap_or(0);
            let menu_text = if count > 1 {
                format!("Send {count} shortcuts to remote-dl")
            } else {
                "Send to remote-dl".to_string()
            };

            let label = wide(&menu_text);
            AppendMenuW(
                hmenu,
                MF_STRING | MF_BYPOSITION,
                idcmdfirst as usize,
                PCWSTR(label.as_ptr()),
            )?;
            let _ = indexmenu;
            Ok(())
        }
    }

    fn InvokeCommand(&self, pici: *const CMINVOKECOMMANDINFO) -> windows::core::Result<()> {
        if pici.is_null() {
            return Err(E_INVALIDARG.into());
        }

        let paths: Vec<String> = self
            .selected
            .lock()
            .ok()
            .map(|g| g.clone())
            .unwrap_or_default();

        if paths.is_empty() {
            return Err(E_FAIL.into());
        }

        // Spawn a background thread to prevent blocking Explorer's UI thread.
        // DllLockGuard ensures the DLL will not be unloaded while the worker is executing.
        std::thread::spawn(move || {
            let _guard = DllLockGuard::new();

            let cfg = match Config::load() {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("rdl-shell: failed to load config: {e}");
                    return;
                }
            };

            let mut urls = Vec::with_capacity(paths.len());
            for path in &paths {
                if let Some(url) = read_url_from_url_file(path) {
                    urls.push(url);
                }
            }

            if urls.is_empty() {
                return;
            }

            let results = client::queue_all(&cfg, &urls);
            for (url, res) in urls.iter().zip(results) {
                if let Err(e) = res {
                    eprintln!("rdl-shell: failed to queue {url}: {e}");
                }
            }
        });

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

/// Reads all file paths from an `IDataObject` carrying `CF_HDROP` data.
///
/// Iterates through the double-NUL terminated list in DROPFILES.
unsafe fn read_selected_paths(data: &IDataObject) -> windows::core::Result<Vec<String>> {
    let fmt = FORMATETC {
        cfFormat: CF_HDROP.0,
        ptd: std::ptr::null_mut(),
        dwAspect: DVASPECT_CONTENT.0,
        lindex: -1,
        tymed: TYMED_HGLOBAL.0 as u32,
    };
    let stg: STGMEDIUM = data.GetData(&fmt)?;

    let hglobal = HGLOBAL(stg.u.hGlobal.0);
    let base = GlobalLock(hglobal) as *const u8;
    if base.is_null() {
        return Err(E_FAIL.into());
    }

    let p_files = *(base as *const u32);
    let f_wide = *(base.add(16) as *const i32) != 0;
    let list = base.add(p_files as usize);

    let mut paths = Vec::new();

    if f_wide {
        let mut p = list as *const u16;
        loop {
            let mut len = 0usize;
            while *p.add(len) != 0 && len < 0x4000 {
                len += 1;
            }
            if len == 0 {
                break;
            }
            let slice = std::slice::from_raw_parts(p, len);
            let s = String::from_utf16_lossy(slice);
            if !s.is_empty() {
                paths.push(s);
            }
            p = p.add(len + 1);
        }
    } else {
        let mut p = list;
        loop {
            let mut len = 0usize;
            while *p.add(len) != 0 && len < 0x4000 {
                len += 1;
            }
            if len == 0 {
                break;
            }
            let slice = std::slice::from_raw_parts(p, len);
            let s = String::from_utf8_lossy(slice).into_owned();
            if !s.is_empty() {
                paths.push(s);
            }
            p = p.add(len + 1);
        }
    }

    let _ = GlobalUnlock(hglobal);
    if paths.is_empty() {
        Err(E_FAIL.into())
    } else {
        Ok(paths)
    }
}

/// Decodes the raw bytes of an `.url` file into a string, supporting UTF-8,
/// UTF-16LE (with/without BOM), and UTF-16BE.
pub fn decode_url_file_bytes(bytes: &[u8]) -> Option<String> {
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return String::from_utf8(bytes[3..].to_vec()).ok();
    }
    if bytes.starts_with(&[0xFF, 0xFE]) {
        let u16_slice: Vec<u16> = bytes[2..]
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();
        return Some(String::from_utf16_lossy(&u16_slice));
    }
    if bytes.starts_with(&[0xFE, 0xFF]) {
        let u16_slice: Vec<u16> = bytes[2..]
            .chunks_exact(2)
            .map(|c| u16::from_be_bytes([c[0], c[1]]))
            .collect();
        return Some(String::from_utf16_lossy(&u16_slice));
    }

    // Check for UTF-16LE without BOM: first ascii chars often have null in odd positions
    if bytes.len() >= 4 && bytes[1] == 0 && bytes[3] == 0 {
        let u16_slice: Vec<u16> = bytes
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();
        return Some(String::from_utf16_lossy(&u16_slice));
    }

    // Fall back to UTF-8
    String::from_utf8(bytes.to_vec())
        .ok()
        .or_else(|| Some(String::from_utf8_lossy(bytes).into_owned()))
}

/// `.url` files are INI-format text. The URL we want is the
/// `URL=` line under `[InternetShortcut]`. Case-insensitive match on the key.
pub fn read_url_from_url_file(path: &str) -> Option<String> {
    let bytes = std::fs::read(path).ok()?;
    let content = decode_url_file_bytes(&bytes)?;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.to_ascii_lowercase().starts_with("url=") {
            let val = trimmed[4..].trim();
            if !val.is_empty() {
                return Some(val.to_string());
            }
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
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("test_rdl_shortcut.url");
        std::fs::write(
            &path,
            b"[InternetShortcut]\r\nURL=https://example.com/x\r\n",
        )
        .unwrap();
        assert_eq!(
            read_url_from_url_file(path.to_str().unwrap()),
            Some("https://example.com/x".to_string())
        );
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn returns_none_when_no_url_line() {
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("nope_rdl_shortcut.url");
        std::fs::write(&path, b"[Some]\r\nKey=value\r\n").unwrap();
        assert!(read_url_from_url_file(path.to_str().unwrap()).is_none());
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn parses_utf16le_shortcut_file() {
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("utf16_rdl_shortcut.url");
        let text = "[InternetShortcut]\r\nURL=https://example.com/utf16\r\n";
        let mut bytes = vec![0xFF, 0xFE];
        for u in text.encode_utf16() {
            bytes.extend_from_slice(&u.to_le_bytes());
        }
        std::fs::write(&path, &bytes).unwrap();
        assert_eq!(
            read_url_from_url_file(path.to_str().unwrap()),
            Some("https://example.com/utf16".to_string())
        );
        let _ = std::fs::remove_file(&path);
    }
}

//! Writes / removes the registry entries that make Explorer load this
//! DLL when the user right-clicks on an `InternetShortcut` (.url) file.
//!
//! Layout:
//!
//! ```text
//! HKCR\CLSID\{6e0d3a1c-9b4f-4c2a-a391-88d2fe6b7021}\(default) = "rdl-shell context menu"
//! HKCR\CLSID\{...}\InprocServer32\(default)        = <path to rdl-shell.dll>
//! HKCR\CLSID\{...}\InprocServer32\ThreadingModel   = "Apartment"
//! HKCR\InternetShortcut\shellex\ContextMenuHandlers\rdl-shell\(default) = "{6e0d3a1c-...}"
//! ```

use crate::error::{Error, Result};
use crate::CLSID_RDL_SHELL;
use std::path::PathBuf;
use windows::core::PCWSTR;
use windows::Win32::Foundation::MAX_PATH;
use windows::Win32::System::LibraryLoader::GetModuleFileNameW;
use windows::Win32::System::Registry::{
    RegCloseKey, RegCreateKeyExW, RegDeleteTreeW, RegSetValueExW, HKEY, HKEY_CLASSES_ROOT,
    KEY_WRITE, REG_OPEN_CREATE_OPTIONS, REG_SZ,
};

const FRIENDLY_NAME: &str = "rdl-shell context menu";
const HANDLER_KEY: &str = r"InternetShortcut\shellex\ContextMenuHandlers\rdl-shell";

pub fn register() -> Result<()> {
    let dll = current_module_path()?;
    let clsid = format_clsid(&CLSID_RDL_SHELL);
    let clsid_key = format!(r"CLSID\{clsid}");
    let inproc_key = format!(r"CLSID\{clsid}\InprocServer32");

    write_default(&clsid_key, FRIENDLY_NAME)?;
    write_default(&inproc_key, dll.to_string_lossy().as_ref())?;
    write_string(&inproc_key, "ThreadingModel", "Apartment")?;
    write_default(HANDLER_KEY, &clsid)?;

    notify_shell_changed();
    Ok(())
}

pub fn unregister() -> Result<()> {
    let clsid = format_clsid(&CLSID_RDL_SHELL);
    delete_tree(&format!(r"CLSID\{clsid}"))?;
    delete_tree(HANDLER_KEY)?;
    notify_shell_changed();
    Ok(())
}

fn write_default(subkey: &str, value: &str) -> Result<()> {
    write_value(subkey, None, value)
}

fn write_string(subkey: &str, name: &str, value: &str) -> Result<()> {
    write_value(subkey, Some(name), value)
}

fn write_value(subkey: &str, name: Option<&str>, value: &str) -> Result<()> {
    let subkey_w = wide(subkey);
    let mut hkey = HKEY::default();
    unsafe {
        RegCreateKeyExW(
            HKEY_CLASSES_ROOT,
            PCWSTR(subkey_w.as_ptr()),
            0,
            PCWSTR::null(),
            REG_OPEN_CREATE_OPTIONS(0),
            KEY_WRITE,
            None,
            &mut hkey,
            None,
        )
        .ok()
        .map_err(|e| Error::Registry(format!("create {subkey}: {e}")))?;

        let value_w = wide(value);
        let bytes: &[u8] = std::slice::from_raw_parts(
            value_w.as_ptr() as *const u8,
            value_w.len() * std::mem::size_of::<u16>(),
        );

        let name_w = name.map(wide);
        let name_pcwstr = match name_w.as_ref() {
            Some(w) => PCWSTR(w.as_ptr()),
            None => PCWSTR::null(),
        };

        let r = RegSetValueExW(hkey, name_pcwstr, 0, REG_SZ, Some(bytes));
        let _ = RegCloseKey(hkey);
        r.ok()
            .map_err(|e| Error::Registry(format!("set {subkey}\\{name:?}: {e}")))?;
    }
    Ok(())
}

fn delete_tree(subkey: &str) -> Result<()> {
    let subkey_w = wide(subkey);
    unsafe {
        let r = RegDeleteTreeW(HKEY_CLASSES_ROOT, PCWSTR(subkey_w.as_ptr()));
        // Ignore "not found" — unregister is idempotent.
        if r.is_err() {
            let _ = r; // swallow
        }
    }
    Ok(())
}

fn current_module_path() -> Result<PathBuf> {
    use windows::Win32::Foundation::HMODULE;
    let mut buf = vec![0u16; MAX_PATH as usize];
    unsafe {
        let raw = crate::DLL_HMODULE.load(std::sync::atomic::Ordering::SeqCst);
        let hmod = HMODULE(raw as *mut std::ffi::c_void);
        let n = GetModuleFileNameW(hmod, &mut buf);
        if n == 0 {
            return Err(Error::Registry("GetModuleFileNameW failed".into()));
        }
        buf.truncate(n as usize);
    }
    Ok(PathBuf::from(String::from_utf16_lossy(&buf)))
}

fn notify_shell_changed() {
    // SHCNE_ASSOCCHANGED tells Explorer to rebuild its file association cache.
    unsafe {
        windows::Win32::UI::Shell::SHChangeNotify(
            windows::Win32::UI::Shell::SHCNE_ASSOCCHANGED,
            windows::Win32::UI::Shell::SHCNF_IDLIST,
            None,
            None,
        );
    }
}

fn format_clsid(g: &windows::core::GUID) -> String {
    format!(
        "{{{:08x}-{:04x}-{:04x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}}}",
        g.data1,
        g.data2,
        g.data3,
        g.data4[0],
        g.data4[1],
        g.data4[2],
        g.data4[3],
        g.data4[4],
        g.data4[5],
        g.data4[6],
        g.data4[7],
    )
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_clsid_correctly() {
        let s = format_clsid(&CLSID_RDL_SHELL);
        assert_eq!(s, "{6e0d3a1c-9b4f-4c2a-a391-88d2fe6b7021}");
    }
}

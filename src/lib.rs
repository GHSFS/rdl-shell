//! `rdl-shell` — Windows Explorer shell extension that adds a
//! "Send to remote-dl" entry to the context menu of `.url` Internet Shortcut
//! files.
//!
//! This crate is built as a `cdylib` and exports the four COM entry points
//! (`DllGetClassObject`, `DllCanUnloadNow`, `DllRegisterServer`,
//! `DllUnregisterServer`) that `regsvr32.exe` looks for when registering a
//! shell extension.
//!
//! See `README.md` for installation and removal.

#![cfg(windows)]
#![allow(non_snake_case)]

mod client;
mod config;
mod error;
mod factory;
mod handler;
mod registry;

use std::sync::atomic::{AtomicI32, AtomicIsize, Ordering};
use windows::core::{IUnknown, Interface, GUID, HRESULT};
use windows::Win32::Foundation::{
    BOOL, CLASS_E_CLASSNOTAVAILABLE, E_NOINTERFACE, E_POINTER, S_FALSE, S_OK,
};
use windows::Win32::System::Com::IClassFactory;

/// CLSID assigned to this shell extension.
///
/// Generated once and never changed. The registry entries written by
/// `DllRegisterServer` reference this same value.
pub const CLSID_RDL_SHELL: GUID = GUID::from_values(
    0x6e0d3a1c,
    0x9b4f,
    0x4c2a,
    [0xa3, 0x91, 0x88, 0xd2, 0xfe, 0x6b, 0x70, 0x21],
);

/// Tracks the number of outstanding COM objects this DLL has handed out.
/// `DllCanUnloadNow` returns `S_OK` only when this is zero.
pub(crate) static REF_COUNT: AtomicI32 = AtomicI32::new(0);

/// Captured at `DllMain(DLL_PROCESS_ATTACH)` so registry self-registration
/// can resolve our own DLL path via `GetModuleFileNameW`.
pub(crate) static DLL_HMODULE: AtomicIsize = AtomicIsize::new(0);

pub(crate) fn lock() {
    REF_COUNT.fetch_add(1, Ordering::SeqCst);
}

pub(crate) fn unlock() {
    REF_COUNT.fetch_sub(1, Ordering::SeqCst);
}

/// COM entry point — returns an `IClassFactory` for our CLSID.
#[no_mangle]
pub unsafe extern "system" fn DllGetClassObject(
    rclsid: *const GUID,
    riid: *const GUID,
    ppv: *mut *mut std::ffi::c_void,
) -> HRESULT {
    if rclsid.is_null() || riid.is_null() || ppv.is_null() {
        return E_POINTER;
    }
    *ppv = std::ptr::null_mut();

    if *rclsid != CLSID_RDL_SHELL {
        return CLASS_E_CLASSNOTAVAILABLE;
    }
    if *riid != IClassFactory::IID && *riid != IUnknown::IID {
        return E_NOINTERFACE;
    }

    let factory: IClassFactory = factory::ClassFactory::default().into();
    *ppv = std::mem::transmute(factory);
    S_OK
}

/// COM entry point — `S_OK` means the DLL can be unloaded.
#[no_mangle]
pub extern "system" fn DllCanUnloadNow() -> HRESULT {
    if REF_COUNT.load(Ordering::SeqCst) == 0 {
        S_OK
    } else {
        S_FALSE
    }
}

/// COM entry point — installs registry entries so Explorer picks up the
/// extension. Called by `regsvr32.exe rdl-shell.dll`.
#[no_mangle]
pub extern "system" fn DllRegisterServer() -> HRESULT {
    match registry::register() {
        Ok(()) => S_OK,
        Err(e) => {
            eprintln!("DllRegisterServer: {e:#}");
            HRESULT(0x80004005u32 as i32) // E_FAIL
        }
    }
}

/// COM entry point — removes the registry entries written by
/// `DllRegisterServer`. Called by `regsvr32.exe /u rdl-shell.dll`.
#[no_mangle]
pub extern "system" fn DllUnregisterServer() -> HRESULT {
    match registry::unregister() {
        Ok(()) => S_OK,
        Err(e) => {
            eprintln!("DllUnregisterServer: {e:#}");
            HRESULT(0x80004005u32 as i32)
        }
    }
}

/// Standard `DllMain`. Captures the DLL's HMODULE so the self-registration
/// code can find its own filesystem path later.
#[no_mangle]
pub extern "system" fn DllMain(
    hinst: isize,
    reason: u32,
    _reserved: *mut std::ffi::c_void,
) -> BOOL {
    const DLL_PROCESS_ATTACH: u32 = 1;
    if reason == DLL_PROCESS_ATTACH {
        DLL_HMODULE.store(hinst, Ordering::SeqCst);
    }
    BOOL(1)
}

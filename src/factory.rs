//! `IClassFactory` implementation that hands out `RdlShellHandler` instances.

#![allow(non_snake_case)]

use crate::handler::RdlShellHandler;
use windows::core::{implement, IUnknown, Interface, GUID};
use windows::Win32::Foundation::{BOOL, CLASS_E_NOAGGREGATION, E_NOINTERFACE};
use windows::Win32::System::Com::IClassFactory;
use windows::Win32::System::Com::IClassFactory_Impl;
use windows::Win32::UI::Shell::{IContextMenu, IShellExtInit};

#[implement(IClassFactory)]
#[derive(Default)]
pub struct ClassFactory;

impl IClassFactory_Impl for ClassFactory_Impl {
    fn CreateInstance(
        &self,
        outer: Option<&IUnknown>,
        riid: *const GUID,
        ppv: *mut *mut std::ffi::c_void,
    ) -> windows::core::Result<()> {
        unsafe {
            if !ppv.is_null() {
                *ppv = std::ptr::null_mut();
            }
            if outer.is_some() {
                return Err(CLASS_E_NOAGGREGATION.into());
            }
            if riid.is_null() || ppv.is_null() {
                return Err(E_NOINTERFACE.into());
            }

            let handler = RdlShellHandler::default();
            let unk: IUnknown = handler.into();

            let iid = *riid;
            if iid == IShellExtInit::IID
                || iid == IContextMenu::IID
                || iid == IUnknown::IID
            {
                let iface: IUnknown = unk.cast()?;
                *ppv = std::mem::transmute(iface);
                Ok(())
            } else {
                Err(E_NOINTERFACE.into())
            }
        }
    }

    fn LockServer(&self, lock: BOOL) -> windows::core::Result<()> {
        if lock.as_bool() {
            crate::lock();
        } else {
            crate::unlock();
        }
        Ok(())
    }
}

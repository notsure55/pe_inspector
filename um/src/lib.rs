#![allow(dead_code)]

use anyhow::Result;
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::System::Memory::{
    VirtualAllocEx, MEM_COMMIT, MEM_RESERVE, PAGE_EXECUTE_READWRITE, PAGE_PROTECTION_FLAGS,
    VIRTUAL_ALLOCATION_TYPE,
};
use windows::Win32::System::Threading::{OpenProcess, PROCESS_ALL_ACCESS};

type PVOID = *const core::ffi::c_void;

mod driver;
mod um;

#[derive(Debug)]
enum Handle {
    Kernel(HANDLE),
    User(HANDLE),
}

impl std::ops::Drop for Handle {
    fn drop(&mut self) {
        match self {
            Self::Kernel(h) => unsafe { CloseHandle(*h).unwrap() },
            Self::User(h) => unsafe { CloseHandle(*h).unwrap() },
        }
    }
}

#[derive(Debug)]
pub struct Process {
    handle: Handle,
}

impl Process {
    pub fn from_name(name: impl AsRef<str>) -> Result<Self> {
        let handle = {
            if let Ok(handle) = driver::access_driver() {
                println!("[+] Found handle to kernel driver attaching to process!");

                driver::attach_process(handle, name)?;

                Handle::Kernel(handle)
            } else {
                let pid = um::find_pid(name)?;

                let handle = Handle::User(unsafe { OpenProcess(PROCESS_ALL_ACCESS, false, pid)? });

                handle
            }
        };

        Ok(Self { handle })
    }
    pub fn alloc_memory(
        &self,
        address: Option<PVOID>,
        size: usize,
        alloc_type: Option<VIRTUAL_ALLOCATION_TYPE>,
        protection_flags: Option<PAGE_PROTECTION_FLAGS>,
    ) -> PVOID {
        match self.handle {
            Handle::Kernel(_) => std::ptr::null(),
            Handle::User(h) => unsafe {
                VirtualAllocEx(
                    h,
                    address,
                    size,
                    alloc_type.unwrap_or(MEM_COMMIT | MEM_RESERVE),
                    protection_flags.unwrap_or(PAGE_EXECUTE_READWRITE),
                )
            },
        }
    }
}

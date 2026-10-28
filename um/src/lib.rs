#![allow(dead_code)]

use anyhow::{anyhow, Result};
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::System::Diagnostics::Debug::{ReadProcessMemory, WriteProcessMemory};
use windows::Win32::System::Memory::{
    VirtualAllocEx, VirtualProtectEx, MEM_COMMIT, MEM_RESERVE, PAGE_EXECUTE_READWRITE,
    PAGE_PROTECTION_FLAGS, VIRTUAL_ALLOCATION_TYPE,
};
use windows::Win32::System::Threading::{OpenProcess, PROCESS_ALL_ACCESS};

use core::ffi::c_void;
use std::ptr::NonNull;

pub use shared::Va;

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
    pub fn read_virtual_memory<T>(&self, va: Va) -> Result<T> {
        let buffer = self.read_virtual_memory_to_buffer(va, std::mem::size_of::<T>())?;
        Ok(unsafe { buffer.as_ptr().cast::<T>().read() })
    }
    pub fn read_virtual_memory_to_buffer(&self, va: Va, size: usize) -> Result<Vec<u8>> {
        match self.handle {
            Handle::Kernel(h) => Ok(driver::read_virtual_memory(h, va, size)?),
            Handle::User(h) => {
                let mut buffer = vec![0u8; size];
                let mut bytes_read: usize = 0;

                unsafe {
                    ReadProcessMemory(
                        h,
                        va.into(),
                        buffer.as_mut_ptr() as _,
                        size,
                        Some(&mut bytes_read),
                    )?
                };

                if bytes_read != size {
                    Err(anyhow!(
                        "Failed to read correct bytes from ReadProcessMemory at address {va:?}"
                    ))
                } else {
                    Ok(buffer)
                }
            }
        }
    }
    pub fn write_virtual_memory<T>(&self, va: Va, value: T) -> Result<()> {
        let size = std::mem::size_of::<T>();

        let bytes = unsafe { std::slice::from_raw_parts((&value as *const T).cast::<u8>(), size) };

        self.write_virtual_memory_from_buffer(va, &bytes)
    }
    pub fn write_virtual_memory_from_buffer(&self, va: Va, bytes: &[u8]) -> Result<()> {
        match self.handle {
            Handle::Kernel(h) => driver::write_virtual_memory(h, va, bytes)?,
            Handle::User(h) => {
                let mut bytes_written = 0;

                unsafe {
                    WriteProcessMemory(
                        h,
                        va.into(),
                        bytes.as_ptr() as _,
                        bytes.len(),
                        Some(&mut bytes_written),
                    )?
                };

                if bytes_written != bytes.len() {
                    return Err(anyhow!(
                        "Bytes written not = to bytes.len() in WriteProcessMemory @ address {va:?}"
                    ));
                }
            }
        }

        Ok(())
    }
    pub fn alloc_memory(
        &self,
        address: Option<NonNull<c_void>>,
        size: usize,
        alloc_type: Option<VIRTUAL_ALLOCATION_TYPE>,
        protection_flags: Option<PAGE_PROTECTION_FLAGS>,
    ) -> *const c_void {
        match self.handle {
            Handle::Kernel(_) => std::ptr::null(),
            Handle::User(h) => unsafe {
                VirtualAllocEx(
                    h,
                    address.and_then(|a| Some(a.as_ptr() as *const _)),
                    size,
                    alloc_type.unwrap_or(MEM_COMMIT | MEM_RESERVE),
                    protection_flags.unwrap_or(PAGE_EXECUTE_READWRITE),
                )
            },
        }
    }
    pub fn overwrite_memory_protections(
        &self,
        address: NonNull<c_void>,
        size: usize,
        flags: PAGE_PROTECTION_FLAGS,
    ) -> Result<PAGE_PROTECTION_FLAGS> {
        let mut old_protect: PAGE_PROTECTION_FLAGS = unsafe { std::mem::zeroed() };
        match self.handle {
            Handle::Kernel(_) => todo!("Handle kernel virtual protect"),
            Handle::User(h) => unsafe {
                VirtualProtectEx(h, address.as_ptr(), size, flags, &mut old_protect)?;
            },
        }

        Ok(old_protect)
    }
}

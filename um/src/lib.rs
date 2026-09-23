#![allow(dead_code)]

use anyhow::{Result, anyhow};
use std::collections::BTreeMap;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, MODULEENTRY32W, Module32FirstW, Module32NextW, PROCESSENTRY32W,
    Process32FirstW, Process32NextW, TH32CS_SNAPMODULE, TH32CS_SNAPPROCESS,
};
use windows::Win32::System::Memory::{
    MEM_COMMIT, MEM_RESERVE, PAGE_EXECUTE_READWRITE, PAGE_PROTECTION_FLAGS,
    VIRTUAL_ALLOCATION_TYPE, VirtualAllocEx,
};
use windows::Win32::System::Threading::{OpenProcess, PROCESS_ALL_ACCESS};
use windows_core::PCWSTR;

type PVOID = *const core::ffi::c_void;

#[derive(Debug)]
pub struct Process {
    pid: u32,
    handle: HANDLE,
    modules: BTreeMap<String, MODULEENTRY32W>,
}

impl Process {
    fn find_pid(name: impl AsRef<str>) -> Result<u32> {
        let handle = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) }?;

        let mut p_entry = PROCESSENTRY32W {
            dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };

        if unsafe { Process32FirstW(handle, &mut p_entry) }.is_ok() {
            loop {
                let process_name =
                    unsafe { PCWSTR::from_raw(p_entry.szExeFile.as_ptr()).to_string()? };

                if name.as_ref() == process_name {
                    return Ok(p_entry.th32ProcessID);
                }

                if unsafe { Process32NextW(handle, &mut p_entry) }.is_err() {
                    break;
                }
            }
        }

        Err(anyhow!("Couldnt find process {}", name.as_ref()))
    }
    fn cache_modules(pid: u32) -> Result<BTreeMap<String, MODULEENTRY32W>> {
        let handle = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPMODULE, pid)? };

        let mut m_entry = MODULEENTRY32W {
            dwSize: std::mem::size_of::<MODULEENTRY32W>() as u32,
            ..Default::default()
        };

        let mut modules = BTreeMap::new();

        if unsafe { Module32FirstW(handle, &mut m_entry) }.is_ok() {
            loop {
                let module_name =
                    unsafe { PCWSTR::from_raw(m_entry.szModule.as_ptr()).to_string()? };

                modules.insert(module_name, m_entry);

                if unsafe { Module32NextW(handle, &mut m_entry) }.is_err() {
                    break;
                }
            }
        }

        Ok(modules)
    }
    pub fn from_name(name: impl AsRef<str>) -> Result<Self> {
        let pid = Self::find_pid(name)?;

        let handle = unsafe { OpenProcess(PROCESS_ALL_ACCESS, false, pid)? };

        let modules = Self::cache_modules(pid)?;

        Ok(Self {
            pid,
            handle,
            modules,
        })
    }
    pub fn alloc_memory(
        &self,
        address: Option<PVOID>,
        size: usize,
        alloc_type: Option<VIRTUAL_ALLOCATION_TYPE>,
        protection_flags: Option<PAGE_PROTECTION_FLAGS>,
    ) -> PVOID {
        unsafe {
            VirtualAllocEx(
                self.handle,
                address,
                size,
                alloc_type.unwrap_or(MEM_COMMIT | MEM_RESERVE),
                protection_flags.unwrap_or(PAGE_EXECUTE_READWRITE),
            )
        }
    }
}

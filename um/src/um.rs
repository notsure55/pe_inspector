use anyhow::{anyhow, Result};
use std::collections::BTreeMap;
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Module32FirstW, Module32NextW, Process32FirstW, Process32NextW,
    MODULEENTRY32W, PROCESSENTRY32W, TH32CS_SNAPMODULE, TH32CS_SNAPPROCESS,
};
use windows_core::PCWSTR;

type PVOID = *const core::ffi::c_void;

pub fn find_pid(name: impl AsRef<str>) -> Result<u32> {
    let handle = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) }?;

    let mut p_entry = PROCESSENTRY32W {
        dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
        ..Default::default()
    };

    if unsafe { Process32FirstW(handle, &mut p_entry) }.is_ok() {
        loop {
            let process_name = unsafe { PCWSTR::from_raw(p_entry.szExeFile.as_ptr()).to_string()? };

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

pub fn cache_modules(pid: u32) -> Result<BTreeMap<String, MODULEENTRY32W>> {
    let handle = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPMODULE, pid)? };

    let mut m_entry = MODULEENTRY32W {
        dwSize: std::mem::size_of::<MODULEENTRY32W>() as u32,
        ..Default::default()
    };

    let mut modules = BTreeMap::new();

    if unsafe { Module32FirstW(handle, &mut m_entry) }.is_ok() {
        loop {
            let module_name = unsafe { PCWSTR::from_raw(m_entry.szModule.as_ptr()).to_string()? };

            modules.insert(module_name, m_entry);

            if unsafe { Module32NextW(handle, &mut m_entry) }.is_err() {
                break;
            }
        }
    }

    Ok(modules)
}

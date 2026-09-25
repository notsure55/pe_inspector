use anyhow::Result;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::Foundation::{GENERIC_READ, GENERIC_WRITE};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, FILE_FLAGS_AND_ATTRIBUTES, FILE_SHARE_MODE, OPEN_EXISTING,
};
use windows_core::{w, PCWSTR};

use shared::IOCTL_ATTACH_PROCESS;
use windows::Win32::System::IO::DeviceIoControl;

pub fn access_driver() -> Result<HANDLE> {
    Ok(unsafe {
        CreateFileW(
            w!("\\\\.\\PeInspector1"),
            (GENERIC_READ | GENERIC_WRITE).0,
            FILE_SHARE_MODE(0),
            None,
            OPEN_EXISTING,
            FILE_FLAGS_AND_ATTRIBUTES(0),
            None,
        )
    }?)
}

pub fn attach_process(handle: HANDLE, name: impl AsRef<str>) -> Result<()> {
    let name = name.as_ref();
    let name = PCWSTR::from_raw(shared::wide!(name));

    unsafe {
        DeviceIoControl(
            handle,
            IOCTL_ATTACH_PROCESS,
            Some(&name as *const _ as _),
            std::mem::size_of::<usize>() as u32,
            None,
            0,
            None,
            None,
        )?;
    }

    Ok(())
}

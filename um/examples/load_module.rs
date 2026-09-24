#![allow(non_snake_case)]

use anyhow::Result;

use windows::Win32::System::LibraryLoader::{/*GetProcAddress,*/ LoadLibraryW};
use windows_core::w;

fn main() -> Result<()> {
    let _ = unsafe { LoadLibraryW(w!("sample1.dll"))? };

    std::thread::sleep(std::time::Duration::from_secs(5));

    Ok(())
}

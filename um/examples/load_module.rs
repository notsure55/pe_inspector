#![allow(non_snake_case)]

use anyhow::Result;

use windows::Win32::Foundation::FreeLibrary;
use windows::Win32::System::LibraryLoader::{/*GetProcAddress,*/ LoadLibraryW};
use windows_core::w;

fn main() -> Result<()> {
    loop {
        let _ = load_module();
    }
}

fn load_module() -> Result<()> {
    let lib = unsafe { LoadLibraryW(w!("sample1.dll"))? };
    unsafe { FreeLibrary(lib)? };
    Ok(())
}

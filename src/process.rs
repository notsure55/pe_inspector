#![allow(non_camel_case_types)]

extern crate alloc;

use alloc::string::String;
use alloc::string::ToString;
use wdk::println;
use wdk_sys::STATUS_UNSUCCESSFUL;

use windows_types::kernel::eprocess::{Eprocess, EPROCESS};
use windows_types::kernel::result::Result;

static mut CURRENT_PROCESS: Option<*mut EPROCESS> = None;

pub fn get_eprocess() -> Eprocess {
    if let Some(ptr) = unsafe { *(&raw mut CURRENT_PROCESS) } {
        if !ptr.is_null() {
            return Eprocess::from_raw(ptr);
        }
    }

    Eprocess::from_current()
}

pub fn set_eprocess(raw: *mut EPROCESS) {
    unsafe { *(&raw mut CURRENT_PROCESS) = Some(raw) };
}

pub fn from_name(process_name: String) -> Result<()> {
    let mut process = get_eprocess();

    if let Some(original_name) = process.image_name() {
        let original_name = original_name.to_string();
        // Already attached to the process name
        if process_name.contains(&original_name) {
            return Result::Ok(());
        }

        loop {
            process = process.next_process();

            if let Some(name) = process.image_name() {
                if process_name.contains(name) {
                    set_eprocess(process.raw);
                    return Result::Ok(());
                }

                if original_name == name {
                    println!("Failed to find process!");
                    break;
                }
            }
        }
    }

    Result::Status(STATUS_UNSUCCESSFUL)
}

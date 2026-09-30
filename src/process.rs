#![allow(non_camel_case_types)]

extern crate alloc;

use alloc::string::String;
use alloc::string::ToString;
use wdk::println;
use wdk_sys::{STATUS_SUCCESS, STATUS_UNSUCCESSFUL};

use windows_types::kernel::eprocess::Eprocess;
use windows_types::kernel::result::Result;

static mut PROCESS: Eprocess = Eprocess {
    raw: core::ptr::null_mut(),
    kapc_state: core::cell::UnsafeCell::new(None),
};

pub fn get_eprocess() -> &'static Eprocess {
    unsafe { (&raw const PROCESS).as_ref_unchecked() }
}
pub fn get_mut_eprocess() -> &'static mut Eprocess {
    unsafe { (&raw mut PROCESS).as_mut_unchecked() }
}

// TODO: make memory reading functions
pub fn from_name(process_name: String) -> Result<()> {
    let process = get_mut_eprocess();
    process.raw = Eprocess::from_current().raw;

    let original_name = process.image_name().unwrap().to_string();

    loop {
        process.next_process();

        if let Some(name) = process.image_name() {
            if process_name.contains(name) {
                println!("Found process => {}", process_name);
                return Result::Status(STATUS_SUCCESS);
            }

            if original_name == name {
                println!("Failed to find process!");
                break;
            }
        }
    }

    Result::Status(STATUS_UNSUCCESSFUL)
}

#![allow(non_camel_case_types)]

use super::result::Result;

extern crate alloc;

use alloc::string::String;
use alloc::string::ToString;
use wdk::println;
use wdk_sys::{STATUS_SUCCESS, STATUS_UNSUCCESSFUL};

use windows_types::kernel::eprocess::Eprocess;

// TODO: make memory reading functions
pub fn from_name(process_name: String) -> Result<()> {
    let mut process = Eprocess::from_current();

    let original_name = process.image_name().unwrap().to_string();

    loop {
        process = process.next_process();

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

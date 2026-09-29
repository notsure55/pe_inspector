#![allow(non_camel_case_types)]

use super::result::Result;
use wdk::println;
use wdk_sys::ntddk::IoGetCurrentProcess;
use wdk_sys::{
    HANDLE, LIST_ENTRY, PEPROCESS, PPEB, PVOID, STATUS_SUCCESS, STATUS_UNSUCCESSFUL, UNICODE_STRING,
};

use windows_types::kernel::{eprocess::Eprocess, unicode_string::UnicodeString};

pub fn from_name(process_name: &UnicodeString) -> Result<()> {
    let process = Eprocess::from_current();
    let original_name = &process.peb.process_parameters.image_path_name;

    loop {
        let name = &process.peb.process_parameters.image_path_name;

        let list_entry = &process.pcb.process_list_entry;

        let list_entry_addr = list_entry as *const _ as usize;

        let next_process = unsafe {
            list_entry
                .Flink
                .cast::<*mut LIST_ENTRY>()
                .read()
                .byte_offset(!((list_entry_addr - process.raw as usize) as isize))
                .cast::<Eprocess>()
        };

        println!("Whus up we found a name {name}");

        if process_name == name {
            println!("Whus up we found the right name! {name}");
            return Result::Status(STATUS_SUCCESS);
        }

        if original_name == name {
            println!("Failed to find process!");
            break;
        }
    }

    Result::Status(STATUS_UNSUCCESSFUL)
}

#![no_std]

use wdk_sys::{FILE_ANY_ACCESS, METHOD_BUFFERED};

macro_rules! ctl_code {
    ($device_type: literal, $function: literal, $method: expr, $access: expr) => {
        $device_type << 16 | $access << 14 | $function << 2 | $method
    };
}

#[macro_export]
macro_rules! wide {
    ($str: literal) => {{
        extern crate alloc;

        concat!($str, "\0")
            .encode_utf16()
            .collect::<alloc::vec::Vec<u16>>()
            .as_ptr()
    }};
    ($str: expr) => {{
        extern crate alloc;

        $str.encode_utf16()
            .into_iter()
            .chain(std::iter::once(0))
            .collect::<alloc::vec::Vec<u16>>()
            .as_ptr()
    }};
}

pub const IOCTL_ATTACH_PROCESS: u32 = ctl_code!(0x8000, 0x801, METHOD_BUFFERED, FILE_ANY_ACCESS);

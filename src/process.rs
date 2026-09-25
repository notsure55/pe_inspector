use super::result::Result;
use wdk_sys::ntddk::PsGetCurrentProcessId;
use wdk_sys::{STATUS_SUCCESS, UNICODE_STRING};

pub fn from_name(_name: &UNICODE_STRING) -> Result<()> {
    let _pid = unsafe { PsGetCurrentProcessId() };

    Result::Status(STATUS_SUCCESS)
}

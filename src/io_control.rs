#![allow(unused_unsafe)]

use super::process;
use shared::IOCTL_ATTACH_PROCESS;
use wdk::println;
use wdk_sys::ntddk::{/*__int2c,*/ IofCompleteRequest};
use wdk_sys::{DEVICE_OBJECT, IRP, STATUS_BAD_DATA, STATUS_SUCCESS, STATUS_UNSUCCESSFUL};

use super::result::Result;
use crate::unicode_str_from_wide_ptr;

macro_rules! return_request {
    ($irp: ident, $status: expr, $info: expr) => {
        $irp.IoStatus.__bindgen_anon_1.Status = $status;
        $irp.IoStatus.Information = 0;
        unsafe { IofCompleteRequest($irp, 0) };

        return Result::Status($status)
    };
}

macro_rules! nt_assert {
    ($expr: expr) => {{
        if (($expr) != true) {
            let line = core::line!();
            println!("Failed nt assert on line {line}");
            //__int2c();
            false
        } else {
            true
        }
    }};
}

macro_rules! io_get_current_irp_stack_location {
    ($irp: ident) => {{
        if !nt_assert!($irp.CurrentLocation <= $irp.StackCount + 1) {
            return_request!($irp, STATUS_UNSUCCESSFUL, 0);
        }

        $irp.Tail
            .Overlay
            .__bindgen_anon_2
            .__bindgen_anon_1
            .CurrentStackLocation
    }};
}

pub unsafe extern "C" fn device_io_control(
    _device_object: &mut DEVICE_OBJECT,
    irp: &mut IRP,
) -> Result<()> {
    let stack_location = unsafe { io_get_current_irp_stack_location!(irp).as_ref_unchecked() };
    let dic = unsafe { stack_location.Parameters.DeviceIoControl };

    match dic.IoControlCode {
        IOCTL_ATTACH_PROCESS => {
            if dic.InputBufferLength != core::mem::size_of::<usize>() as u32 {
                return_request!(irp, STATUS_BAD_DATA, 0);
            }

            let process_name = unicode_str_from_wide_ptr!(unsafe {
                irp.AssociatedIrp.SystemBuffer.cast::<*const u16>().read()
            });

            process::from_name(&process_name);

            return_request!(irp, STATUS_SUCCESS, 0);
        }
        _ => {}
    }

    return_request!(irp, STATUS_SUCCESS, 0);
}

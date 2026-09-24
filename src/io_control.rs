use shared::IOCTL_ATTACH_PROCESS;
use wdk::println;
use wdk_sys::ntddk::{/*__int2c,*/ IofCompleteRequest};
use wdk_sys::{DEVICE_OBJECT, IRP, STATUS_BAD_DATA, STATUS_SUCCESS};

use super::result::Result;

macro_rules! invalid_request {
    ($irp: ident, $status: expr) => {
        $irp.IoStatus.__bindgen_anon_1.Status = $status;
        $irp.IoStatus.Information = 0;
        unsafe { IofCompleteRequest($irp, 0) };

        return Result::Status($status)
    };
}

macro_rules! nt_assert {
    ($expr: expr) => {
        if (($expr) != false) {
            let line = core::line!();
            println!("Failed nt assert on line {line}");
            //__int2c();
        }
    };
}

macro_rules! io_get_current_irp_stack_location {
    ($irp: expr) => {{
        nt_assert!($irp.CurrentLocation <= $irp.StackCount + 1);

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
            if dic.InputBufferLength < core::mem::size_of::<u64>() as u32 {
                invalid_request!(irp, STATUS_BAD_DATA);
            }
        }
        _ => {}
    }

    irp.IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;

    irp.IoStatus.Information = 0;

    unsafe { IofCompleteRequest(irp, 0) };

    Result::Status(STATUS_SUCCESS)
}

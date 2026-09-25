#![no_std]
#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]

use shared::wide;
use wdk::println;
use wdk_sys::ntddk::{
    IoCreateSymbolicLink, IoDeleteDevice, IoDeleteSymbolicLink, IofCompleteRequest,
    RtlInitUnicodeString,
};
use wdk_sys::{
    DEVICE_OBJECT, DRIVER_OBJECT, IRP, IRP_MJ_CLOSE, IRP_MJ_CREATE, IRP_MJ_DEVICE_CONTROL,
    NTSTATUS, NT_SUCCESS, PCUNICODE_STRING, STATUS_SUCCESS, UNICODE_STRING,
};

mod device_object;
mod io_control;
pub mod process;
mod result;

use device_object::DeviceObject;
use result::Result;

#[cfg(not(test))]
extern crate wdk_panic;

#[cfg(not(test))]
use wdk_alloc::WdkAllocator;

#[cfg(not(test))]
#[global_allocator]
static GLOBAL_ALLOCATOR: WdkAllocator = WdkAllocator;

// SAFETY: "DriverEntry" is the required symbol name for Windows driver entry points.
// No other function in this compilation unit exports this name, preventing symbol conflicts.
#[unsafe(export_name = "DriverEntry")] // WDF expects a symbol with the name DriverEntry
pub unsafe extern "system" fn driver_entry(
    driver_object: &mut DRIVER_OBJECT,
    _registry_path: PCUNICODE_STRING,
) -> Result<()> {
    println!("Hey we are in the driver entry!");

    let driver_unload = unsafe {
        core::mem::transmute::<
            unsafe extern "C" fn(&mut DRIVER_OBJECT),
            unsafe extern "C" fn(*mut DRIVER_OBJECT),
        >(driver_unload)
    };

    driver_object.DriverUnload = Some(driver_unload);

    let device_io_control = unsafe {
        core::mem::transmute::<
            unsafe extern "C" fn(&mut DEVICE_OBJECT, &mut IRP) -> Result<()>,
            unsafe extern "C" fn(*mut DEVICE_OBJECT, *mut IRP) -> i32,
        >(io_control::device_io_control)
    };

    driver_object.MajorFunction[IRP_MJ_DEVICE_CONTROL as usize] = Some(device_io_control);

    let device_create_close = unsafe {
        core::mem::transmute::<
            unsafe extern "C" fn(&mut DEVICE_OBJECT, &mut IRP) -> NTSTATUS,
            unsafe extern "C" fn(*mut DEVICE_OBJECT, *mut IRP) -> i32,
        >(device_create_close)
    };

    driver_object.MajorFunction[IRP_MJ_CLOSE as usize] = Some(device_create_close);
    driver_object.MajorFunction[IRP_MJ_CREATE as usize] = Some(device_create_close);

    let mut device_name = unicode_str!("\\Device\\PeInspector1");

    let device_object = DeviceObject::new(driver_object, device_name)?;

    let mut sym_name = unicode_str!("\\??\\PeInspector1");
    check_status!(
        unsafe { IoCreateSymbolicLink(&mut sym_name, &mut device_name) },
        unsafe { IoDeleteDevice(device_object.0) }
    );

    Result::Status(STATUS_SUCCESS)
}

unsafe extern "C" fn device_create_close(
    _device_object: &mut DEVICE_OBJECT,
    irp: &mut IRP,
) -> NTSTATUS {
    irp.IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    irp.IoStatus.Information = 0;
    unsafe { IofCompleteRequest(irp, 0) };

    STATUS_SUCCESS
}

unsafe extern "C" fn driver_unload(driver_object: &mut DRIVER_OBJECT) {
    println!("Unloading driver!");

    let mut sym_name = unicode_str!("\\??\\PeInspector1");

    let _ = unsafe { IoDeleteSymbolicLink(&mut sym_name) };
    unsafe { IoDeleteDevice(driver_object.DeviceObject) };
}

#[macro_export]
macro_rules! unicode_str {
    ($str:expr) => {{
        let mut string: UNICODE_STRING = Default::default();
        unsafe { RtlInitUnicodeString(&mut string, wide!($str)) };
        string
    }};
}

#[macro_export]
macro_rules! unicode_str_from_wide_ptr {
    ($ptr:expr) => {{
        use wdk_sys::ntddk::RtlInitUnicodeString;
        use wdk_sys::UNICODE_STRING;

        let mut string: UNICODE_STRING = Default::default();
        unsafe { RtlInitUnicodeString(&mut string, $ptr) };
        string
    }};
}

#[macro_export]
macro_rules! check_status {
    ($status: expr) => {{
        let result = $status;

        if !NT_SUCCESS(result) {
            return Err(result);
        }
    }};
    ($status: expr, $failure: expr) => {{
        let result = $status;

        if !NT_SUCCESS(result) {
            $failure;
            return Result::Status(result);
        }
    }};
}

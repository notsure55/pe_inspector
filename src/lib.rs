#![no_std]
extern crate alloc;

use core::result;
use wdk::println;
use wdk_sys::ntddk::{
    IoCreateSymbolicLink, IoDeleteDevice, IoDeleteSymbolicLink, IofCompleteRequest,
    RtlInitUnicodeString,
};
use wdk_sys::{
    DEVICE_OBJECT, DRIVER_OBJECT, IRP, IRP_MJ_CLOSE, IRP_MJ_CREATE, IRP_MJ_DEVICE_CONTROL,
    NTSTATUS, NT_SUCCESS, PCUNICODE_STRING, STATUS_SUCCESS, UNICODE_STRING, WCHAR,
};

mod device_object;
use device_object::DeviceObject;

pub type Result<T> = result::Result<T, NTSTATUS>;

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
) -> Result<NTSTATUS> {
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
            unsafe extern "C" fn(&mut DEVICE_OBJECT, &mut IRP) -> NTSTATUS,
            unsafe extern "C" fn(*mut DEVICE_OBJECT, *mut IRP) -> i32,
        >(device_io_control)
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
        unsafe { IoDeleteDevice(device_object.raw) }
    );

    Ok(STATUS_SUCCESS)
}

unsafe extern "C" fn device_io_control(
    _device_object: &mut DEVICE_OBJECT,
    _irp: &mut IRP,
) -> NTSTATUS {
    0
}

unsafe extern "C" fn device_create_close(
    _device_object: &mut DEVICE_OBJECT,
    irp: &mut IRP,
) -> NTSTATUS {
    irp.IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    irp.IoStatus.Information = 0;
    IofCompleteRequest(irp, 0);

    STATUS_SUCCESS
}

unsafe extern "C" fn driver_unload(driver_object: &mut DRIVER_OBJECT) {
    println!("Unloading driver!");

    let mut sym_name = unicode_str!("\\??\\PeDisector1");

    unsafe { IoDeleteSymbolicLink(&mut sym_name) };
    unsafe { IoDeleteDevice(driver_object.DeviceObject) };
}

#[macro_export]
macro_rules! wide {
    ($str:literal) => {
        concat!($str, "\0")
            .encode_utf16()
            .collect::<alloc::vec::Vec<WCHAR>>()
            .as_ptr()
    };
}

#[macro_export]
macro_rules! unicode_str {
    ($str:literal) => {{
        let mut string: UNICODE_STRING = Default::default();
        unsafe { RtlInitUnicodeString(&mut string, wide!($str)) };
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
            return Err(result);
        }
    }};
}

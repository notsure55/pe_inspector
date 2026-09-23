use wdk_sys::{
    DEVICE_OBJECT, DRIVER_OBJECT, FILE_DEVICE_SECURE_OPEN, FILE_DEVICE_UNKNOWN, NT_SUCCESS,
    UNICODE_STRING,
};

use wdk_sys::ntddk::IoCreateDevice;

use crate::Result;

pub struct DeviceObject {
    pub raw: *mut DEVICE_OBJECT,
    pub name: UNICODE_STRING,
}

impl DeviceObject {
    pub fn new(driver_object: &mut DRIVER_OBJECT, mut name: UNICODE_STRING) -> Result<Self> {
        let mut device_object: *mut DEVICE_OBJECT = core::ptr::null_mut();

        let status = unsafe {
            IoCreateDevice(
                driver_object,
                0,
                &mut name,
                FILE_DEVICE_UNKNOWN,
                FILE_DEVICE_SECURE_OPEN,
                0,
                &mut device_object,
            )
        };

        if !NT_SUCCESS(status) {
            Err(status)
        } else {
            Ok(Self {
                raw: device_object,
                name,
            })
        }
    }
}

use wdk_sys::{
    DEVICE_OBJECT, DRIVER_OBJECT, FILE_DEVICE_SECURE_OPEN, FILE_DEVICE_UNKNOWN, NT_SUCCESS,
    UNICODE_STRING,
};

use core::ops::{Deref, DerefMut};
use wdk_sys::ntddk::IoCreateDevice;

use super::result::Result;

pub struct DeviceObject(pub *mut DEVICE_OBJECT);

impl Deref for DeviceObject {
    type Target = DEVICE_OBJECT;
    fn deref(&self) -> &Self::Target {
        unsafe { self.0.as_ref_unchecked() }
    }
}

impl DerefMut for DeviceObject {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.0.as_mut_unchecked() }
    }
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
            Result::Status(status)
        } else {
            Result::Ok(Self { 0: device_object })
        }
    }
}

use std::mem::MaybeUninit;
use garn_proc_macros::ffi_error_propagation;
use crate::interface::error_handling::{ffi_error, ffi_no_error, Error};
use crate::mutex::Mutex;

#[cfg(cbindgen)]
pub struct Mutex {}

#[unsafe(no_mangle)]
pub extern "C" fn garn_mutex_new() -> *mut MaybeUninit<Mutex> {
    Box::into_raw(Box::new(MaybeUninit::uninit()))
}

#[unsafe(no_mangle)]
#[ffi_error_propagation]
pub unsafe extern "C" fn garn_mutex_lock(mutex: *const Mutex) -> Error {
    let Some(mutex) = (unsafe {mutex.as_ref()}) else {
        return ffi_error!(NonNullReferenceViolation, mutex);
    };

    mutex.lock();

    ffi_no_error!()
}

#[unsafe(no_mangle)]
#[ffi_error_propagation]
pub unsafe extern "C" fn garn_mutex_unlock(mutex: *const Mutex) -> Error {
    let Some(mutex) = (unsafe {mutex.as_ref()}) else {
        return ffi_error!(NonNullReferenceViolation, mutex);
    };

    mutex.unlock();

    ffi_no_error!()
}

#[unsafe(no_mangle)]
#[ffi_error_propagation]
pub unsafe extern "C" fn garn_mutex_try_lock(mutex: *const Mutex) -> Error {
    let Some(mutex) = (unsafe {mutex.as_ref()}) else {
        return ffi_error!(NonNullReferenceViolation, mutex);
    };

    mutex.try_lock();

    ffi_no_error!()
}
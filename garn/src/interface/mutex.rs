use crate::interface::error_handling::{Error, ffi_error, ffi_no_error};
use crate::mutex::Mutex;
use garn_proc_macros::ffi_error_propagation;
use std::mem::MaybeUninit;

#[cfg(cbindgen)]
pub struct Mutex {}

#[unsafe(no_mangle)]
#[ffi_error_propagation]
pub unsafe extern "C" fn garn_mutex_lock(mutex: *const Mutex) -> Error {
    let Some(mutex) = (unsafe { mutex.as_ref() }) else {
        return ffi_error!(NonNullReferenceViolation, mutex);
    };

    mutex.lock();

    ffi_no_error!()
}

#[unsafe(no_mangle)]
#[ffi_error_propagation]
pub unsafe extern "C" fn garn_mutex_unlock(mutex: *const Mutex) -> Error {
    let Some(mutex) = (unsafe { mutex.as_ref() }) else {
        return ffi_error!(NonNullReferenceViolation, mutex);
    };

    mutex.unlock();

    ffi_no_error!()
}

#[unsafe(no_mangle)]
#[ffi_error_propagation]
pub unsafe extern "C" fn garn_mutex_try_lock(mutex: *const Mutex) -> Error {
    let Some(mutex) = (unsafe { mutex.as_ref() }) else {
        return ffi_error!(NonNullReferenceViolation, mutex);
    };

    mutex.try_lock();

    ffi_no_error!()
}

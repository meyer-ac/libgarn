use crate::ffi_error_from_partial;
use crate::interface::error_handling::{Error, ffi_error_with_arg, ffi_no_error};
use crate::mutex::Mutex;
use crate::platform_traits::PlatformMutex;
use garn_proc_macros::ffi_error_propagation;

#[cfg(cbindgen)]
pub struct Mutex {}

#[unsafe(no_mangle)]
#[ffi_error_propagation]
pub unsafe extern "C" fn garn_mutex_lock(mutex: *const Mutex) -> *mut Error {
    let Some(mutex) = (unsafe { mutex.as_ref() }) else {
        return ffi_error_with_arg!(NonNullReferenceViolation, mutex);
    };

    if let Err(e) = mutex.lock() {
        return ffi_error_from_partial!(e);
    }

    ffi_no_error!()
}

#[unsafe(no_mangle)]
#[ffi_error_propagation]
pub unsafe extern "C" fn garn_mutex_unlock(mutex: *const Mutex) -> *mut Error {
    let Some(mutex) = (unsafe { mutex.as_ref() }) else {
        return ffi_error_with_arg!(NonNullReferenceViolation, mutex);
    };

    if let Err(e) = mutex.unlock() {
        return ffi_error_from_partial!(e);
    }

    ffi_no_error!()
}

#[unsafe(no_mangle)]
#[ffi_error_propagation]
pub unsafe extern "C" fn garn_mutex_try_lock(mutex: *const Mutex) -> *mut Error {
    let Some(mutex) = (unsafe { mutex.as_ref() }) else {
        return ffi_error_with_arg!(NonNullReferenceViolation, mutex);
    };

    if let Err(e) = mutex.try_lock() {
        return ffi_error_from_partial!(e);
    }

    ffi_no_error!()
}

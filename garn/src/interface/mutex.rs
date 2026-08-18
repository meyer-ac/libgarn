use crate::interface::error_handling::{Error, ffi_error_with_arg, ffi_no_error};
use crate::mutex::Mutex;
use garn_proc_macros::ffi_error_propagation;
use garnshared::platform_traits::PlatformMutex;

#[cfg(cbindgen)]
pub struct Mutex {}

#[unsafe(no_mangle)]
#[ffi_error_propagation]
pub unsafe extern "C" fn garn_mutex_lock(mutex: *const Mutex) -> *mut Error {
    let Some(mutex) = (unsafe { mutex.as_ref() }) else {
        return ffi_error_with_arg!(NonNullReferenceViolation, mutex);
    };

    mutex.lock();

    ffi_no_error!()
}

#[unsafe(no_mangle)]
#[ffi_error_propagation]
pub unsafe extern "C" fn garn_mutex_unlock(mutex: *const Mutex) -> *mut Error {
    let Some(mutex) = (unsafe { mutex.as_ref() }) else {
        return ffi_error_with_arg!(NonNullReferenceViolation, mutex);
    };

    mutex.unlock();

    ffi_no_error!()
}

#[unsafe(no_mangle)]
#[ffi_error_propagation]
pub unsafe extern "C" fn garn_mutex_try_lock(mutex: *const Mutex) -> *mut Error {
    let Some(mutex) = (unsafe { mutex.as_ref() }) else {
        return ffi_error_with_arg!(NonNullReferenceViolation, mutex);
    };

    mutex.try_lock();

    ffi_no_error!()
}

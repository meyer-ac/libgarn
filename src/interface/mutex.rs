use std::mem::MaybeUninit;
use std::os::raw::c_char;
use crate::interface::error_handling::{Error, ErrorType};
use crate::mutex::Mutex;

#[cfg(cbindgen)]
pub struct Mutex {}

#[unsafe(no_mangle)]
pub extern "C" fn garn_mutex_new() -> *mut MaybeUninit<Mutex> {
    Box::into_raw(Box::new(MaybeUninit::uninit()))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn garn_mutex_lock(mutex: *const Mutex) -> Error {
    const FN_NAME: *const c_char = c"garn_mutex_lock".as_ptr();
    const ARG_MUTEX: *const c_char = c"mutex".as_ptr();

    let Some(mutex) = (unsafe {mutex.as_ref()}) else {
        return Error::with_arg(ErrorType::NonNullReferenceViolation, FN_NAME, ARG_MUTEX);
    };

    mutex.lock();

    Error::new(ErrorType::NoError, FN_NAME)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn garn_mutex_unlock(mutex: *const Mutex) -> Error {
    const FN_NAME: *const c_char = c"garn_mutex_unlock".as_ptr();
    const ARG_MUTEX: *const c_char = c"mutex".as_ptr();

    let Some(mutex) = (unsafe {mutex.as_ref()}) else {
        return Error::with_arg(ErrorType::NonNullReferenceViolation, FN_NAME, ARG_MUTEX);
    };

    mutex.unlock();

    Error::new(ErrorType::NoError, FN_NAME)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn garn_mutex_try_lock(mutex: *const Mutex) -> Error {
    const FN_NAME: *const c_char = c"garn_mutex_try_lock".as_ptr();
    const ARG_MUTEX: *const c_char = c"mutex".as_ptr();

    let Some(mutex) = (unsafe {mutex.as_ref()}) else {
        return Error::with_arg(ErrorType::NonNullReferenceViolation, FN_NAME, ARG_MUTEX);
    };

    mutex.try_lock();

    Error::new(ErrorType::NoError, FN_NAME)
}
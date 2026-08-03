use std::ffi::CStr;
use std::mem::MaybeUninit;
use std::os::raw::c_char;
use crate::environment::Environment;
use crate::interface::error_handling::{Error, ErrorType};
use crate::mutex::Mutex;

#[cfg(cbindgen)]
pub struct Environment {}

#[unsafe(no_mangle)]
pub extern "C" fn garn_environment_new() -> *mut MaybeUninit<Environment> {
    Box::into_raw(Box::new(MaybeUninit::uninit()))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn garn_environment_init(env: *mut MaybeUninit<Environment>, name: *const c_char) -> Error {
    const FN_NAME: *const c_char = c"garn_environment_init".as_ptr();
    const ARG_ENV: *const c_char = c"env".as_ptr();
    const ARG_NAME: *const c_char= c"name".as_ptr();

    if env.is_null() {
        return Error::with_arg(ErrorType::NonNullReferenceViolation, FN_NAME, ARG_ENV);
    }

    if name.is_null() {
        return Error::with_arg(ErrorType::NonNullReferenceViolation, FN_NAME, ARG_NAME);
    }

    let Ok(name) = unsafe {CStr::from_ptr(name)}.to_str() else {
        return Error::with_arg(ErrorType::InvalidString, FN_NAME, ARG_NAME);
    };

    let env_obj = Environment::new(name);
    println!("{}", name);

    unsafe {
        (*env).write(env_obj);
    }

    Error::new(ErrorType::NoError, FN_NAME)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn garn_environment_destroy(env: *mut Environment) -> Error {
    const FN_NAME: *const c_char = c"garn_environment_destroy".as_ptr();
    const ARG_ENV: *const c_char = c"env".as_ptr();

    if env.is_null() {
        return Error::with_arg(ErrorType::NonNullReferenceViolation, FN_NAME, ARG_ENV);
    }

    unsafe {
        drop(Box::from_raw(env));
    }

    Error::new(ErrorType::NoError, FN_NAME)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn garn_environment_open_mutex(env: *mut Environment, mutex: *mut MaybeUninit<Mutex>, name: *const c_char) -> Error {
    const FN_NAME: *const c_char = c"garn_environment_open_mutex".as_ptr();
    const ARG_ENV: *const c_char = c"env".as_ptr();
    const ARG_MUTEX: *const c_char = c"mutex".as_ptr();
    const ARG_NAME: *const c_char= c"name".as_ptr();

    let Some(env) = (unsafe {env.as_mut()}) else {
        return Error::with_arg(ErrorType::NonNullReferenceViolation, FN_NAME, ARG_ENV);
    };

    if mutex.is_null() {
        return Error::with_arg(ErrorType::NonNullReferenceViolation, FN_NAME, ARG_MUTEX);
    };

    if name.is_null() {
        return Error::with_arg(ErrorType::NonNullReferenceViolation, FN_NAME, ARG_NAME);
    }

    let Ok(name) = unsafe {CStr::from_ptr(name)}.to_str() else {
        return Error::with_arg(ErrorType::InvalidString, FN_NAME, ARG_NAME);
    };

    let mutex_obj = env.open_mutex(name);
    unsafe {
        (*mutex).write(mutex_obj);
    }

    Error::new(ErrorType::NoError, FN_NAME)
}

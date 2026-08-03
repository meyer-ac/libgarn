use std::ffi::CStr;
use std::mem::MaybeUninit;
use std::os::raw::c_char;
use garn_proc_macros::ffi_error_propagation;
use crate::environment::Environment;
use crate::interface::error_handling::{ffi_error, ffi_no_error, Error};
use crate::mutex::Mutex;

#[cfg(cbindgen)]
pub struct Environment {}

#[unsafe(no_mangle)]
pub extern "C" fn garn_environment_new() -> *mut MaybeUninit<Environment> {
    Box::into_raw(Box::new(MaybeUninit::uninit()))
}

#[unsafe(no_mangle)]
#[ffi_error_propagation]
pub unsafe extern "C" fn garn_environment_init(env: *mut MaybeUninit<Environment>, name: *const c_char) -> Error {
    if env.is_null() {
        return ffi_error!(NonNullReferenceViolation, env);
    }

    if name.is_null() {
        return ffi_error!(NonNullReferenceViolation, name);
    }

    let Ok(name) = unsafe {CStr::from_ptr(name)}.to_str() else {
        return ffi_error!(InvalidString, name);
    };

    let env_obj = Environment::new(name);
    println!("{}", name);

    unsafe {
        (*env).write(env_obj);
    }

    ffi_no_error!()
}

#[unsafe(no_mangle)]
#[ffi_error_propagation]
pub unsafe extern "C" fn garn_environment_destroy(env: *mut Environment) -> Error {
    if env.is_null() {
        return ffi_error!(NonNullReferenceViolation, env);
    }

    unsafe {
        drop(Box::from_raw(env));
    }

    ffi_no_error!()
}

#[unsafe(no_mangle)]
#[ffi_error_propagation]
pub unsafe extern "C" fn garn_environment_open_mutex(env: *mut Environment, mutex: *mut MaybeUninit<Mutex>, name: *const c_char) -> Error {
    let Some(env) = (unsafe {env.as_mut()}) else {
        return ffi_error!(NonNullReferenceViolation, env);
    };

    if mutex.is_null() {
        return ffi_error!(NonNullReferenceViolation, mutex);
    };

    if name.is_null() {
        return ffi_error!(NonNullReferenceViolation, name);
    }

    let Ok(name) = unsafe {CStr::from_ptr(name)}.to_str() else {
        return ffi_error!(InvalidString, name);
    };

    let mutex_obj = env.open_mutex(name);
    unsafe {
        (*mutex).write(mutex_obj);
    }

    ffi_no_error!()
}

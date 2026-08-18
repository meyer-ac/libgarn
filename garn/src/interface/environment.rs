use crate::environment::Environment;
use crate::interface::error_handling::{Error, ffi_no_error};
use crate::mutex::Mutex;
use crate::platform_traits::PlatformEnvironment;
use crate::{ffi_error_from_partial, ffi_error_with_arg};
use garn_proc_macros::ffi_error_propagation;
use std::ffi::CStr;
use std::mem::MaybeUninit;
use std::os::raw::c_char;
use std::thread;

#[cfg(cbindgen)]
pub struct Environment {}

#[unsafe(no_mangle)]
#[ffi_error_propagation]
pub unsafe extern "C" fn garn_environment_init(
    env: *mut MaybeUninit<*mut Environment>,
    name: *const c_char,
) -> *mut Error {
    if env.is_null() {
        return ffi_error_with_arg!(NonNullReferenceViolation, env);
    }

    if name.is_null() {
        return ffi_error_with_arg!(NonNullReferenceViolation, name);
    }

    let Ok(name) = unsafe { CStr::from_ptr(name) }.to_str() else {
        return ffi_error_with_arg!(InvalidString, name);
    };

    let env_ptr = Box::into_raw(Box::new(match Environment::new(name) {
        Ok(res) => res,
        Err(e) => return ffi_error_from_partial!(e),
    }));
    println!("{}", name);

    unsafe {
        (*env).write(env_ptr);
    }

    ffi_no_error!()
}

#[unsafe(no_mangle)]
#[ffi_error_propagation]
pub unsafe extern "C" fn garn_environment_destroy(env: *mut Environment) -> *mut Error {
    {
        let Some(env) = (unsafe { env.as_ref() }) else {
            return ffi_error_with_arg!(NonNullReferenceViolation, env);
        };

        if env.get_owner_thread() != thread::current().id() {
            return ffi_error_with_arg!(ThreadOwnershipViolation, env);
        }
    }

    unsafe {
        drop(Box::from_raw(env));
    }

    ffi_no_error!()
}

#[unsafe(no_mangle)]
#[ffi_error_propagation]
pub unsafe extern "C" fn garn_environment_open_mutex(
    env: *mut Environment,
    mutex: *mut MaybeUninit<*const Mutex>,
    name: *const c_char,
) -> *mut Error {
    {
        let Some(env) = (unsafe { env.as_ref() }) else {
            return ffi_error_with_arg!(NonNullReferenceViolation, env);
        };

        if env.get_owner_thread() != thread::current().id() {
            return ffi_error_with_arg!(ThreadOwnershipViolation, env);
        }
    }

    let env = unsafe { env.as_mut_unchecked() };

    if mutex.is_null() {
        return ffi_error_with_arg!(NonNullReferenceViolation, mutex);
    }

    if name.is_null() {
        return ffi_error_with_arg!(NonNullReferenceViolation, name);
    }

    let Ok(name) = unsafe { CStr::from_ptr(name) }.to_str() else {
        return ffi_error_with_arg!(InvalidString, name);
    };

    let mutex_obj = match env.open_mutex(name) {
        Ok(res) => res,
        Err(e) => return ffi_error_from_partial!(e),
    };

    unsafe {
        (*mutex).write(mutex_obj);
    }

    ffi_no_error!()
}

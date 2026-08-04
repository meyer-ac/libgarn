use crate::environment::Environment;
use crate::interface::error_handling::{Error, ffi_error, ffi_no_error};
use crate::mutex::Mutex;
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
) -> Error {
    if env.is_null() {
        return ffi_error!(NonNullReferenceViolation, env);
    }

    if name.is_null() {
        return ffi_error!(NonNullReferenceViolation, name);
    }

    let Ok(name) = unsafe { CStr::from_ptr(name) }.to_str() else {
        return ffi_error!(InvalidString, name);
    };

    let env_ptr = Box::into_raw(Box::new(Environment::new(name)));
    println!("{}", name);

    unsafe {
        (*env).write(env_ptr);
    }

    ffi_no_error!()
}

#[unsafe(no_mangle)]
#[ffi_error_propagation]
pub unsafe extern "C" fn garn_environment_destroy(env: *mut Environment) -> Error {
    {
        let Some(env) = (unsafe { env.as_ref() }) else {
            return ffi_error!(NonNullReferenceViolation, env);
        };

        if env.get_owner_thread() != thread::current().id() {
            return ffi_error!(ThreadOwnershipViolation, env);
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
) -> Error {
    {
        let Some(env) = (unsafe { env.as_ref() }) else {
            return ffi_error!(NonNullReferenceViolation, env);
        };

        if env.get_owner_thread() != thread::current().id() {
            return ffi_error!(ThreadOwnershipViolation, env);
        }
    }

    let env = unsafe { env.as_mut_unchecked() };

    if mutex.is_null() {
        return ffi_error!(NonNullReferenceViolation, mutex);
    }

    if name.is_null() {
        return ffi_error!(NonNullReferenceViolation, name);
    }

    let Ok(name) = unsafe { CStr::from_ptr(name) }.to_str() else {
        return ffi_error!(InvalidString, name);
    };

    let Some(mutex_obj) = env.open_mutex(name) else {
        return ffi_error!(MutexAlreadyOpened, name);
    };

    unsafe {
        (*mutex).write(std::ptr::from_ref(mutex_obj));
    }

    ffi_no_error!()
}

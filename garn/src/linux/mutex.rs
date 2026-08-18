use crate::interface::error_handling::PartialError;
use crate::platform_traits::PlatformMutex;
use crate::{ffi_partial_error, ffi_partial_error_with_details};
use garnshared::linux::pthread_mutex::PthreadMutex;
use nix::libc;
use nix::libc::{pthread_mutex_lock, pthread_mutex_t, pthread_mutex_trylock, pthread_mutex_unlock};
use std::io::Error;

#[repr(transparent)]
pub struct Mutex(PthreadMutex);

impl PlatformMutex for Mutex {
    fn lock(&self) -> Result<(), PartialError> {
        match unsafe { pthread_mutex_lock(self.0.mutex.get() as *mut pthread_mutex_t) } {
            0 => Ok(()),
            libc::EDEADLK => Err(ffi_partial_error!(MutexNestedLock)),
            _ => Err(ffi_partial_error_with_details!(
                MutexError,
                Error::last_os_error().to_string()
            )),
        }
    }

    fn unlock(&self) -> Result<(), PartialError> {
        match unsafe { pthread_mutex_unlock(self.0.mutex.get() as *mut pthread_mutex_t) } {
            0 => Ok(()),
            libc::EPERM => Err(ffi_partial_error!(MutexUnauthorizedUnlock)),
            _ => Err(ffi_partial_error_with_details!(
                MutexError,
                Error::last_os_error().to_string()
            )),
        }
    }

    fn try_lock(&self) -> Result<(), PartialError> {
        match unsafe { pthread_mutex_trylock(self.0.mutex.get() as *mut pthread_mutex_t) } {
            0 => Ok(()),
            libc::EBUSY => Err(ffi_partial_error!(MutexTrylockFailed)),
            _ => Err(ffi_partial_error_with_details!(
                MutexError,
                Error::last_os_error().to_string()
            )),
        }
    }
}

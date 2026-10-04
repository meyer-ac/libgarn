use crate::interface::error_handling::PartialError;
use crate::platform_traits::PlatformMutex;
use crate::{ffi_partial_error, ffi_partial_error_with_details};
use garnshared::linux::pthread_mutex::PthreadMutex;
use garnshared::pthread_result_errno;
use nix::errno::Errno;
use nix::libc::{
    pthread_mutex_consistent, pthread_mutex_lock, pthread_mutex_t, pthread_mutex_trylock,
    pthread_mutex_unlock,
};

#[repr(transparent)]
pub struct Mutex(PthreadMutex);

// Note: A use-after-destroy detection mechanism that is thread-safe would be too expensive.
// Luckily, this type only ever lives in shared memory when used through the C library, so
// any use-after-destroy almost always results in a SIGSEGV.
// When this type is used through safe Rust though, a use-after-destroy is impossible anyway.
impl PlatformMutex for Mutex {
    fn lock(&self) -> Result<(), PartialError> {
        pthread_result_errno!(unsafe {
            pthread_mutex_lock(self.0.mutex.get().cast::<pthread_mutex_t>())
        })
        .map_err(|e| match e {
            Errno::EDEADLK => ffi_partial_error!(MutexNestedLock),
            Errno::ENOTRECOVERABLE => ffi_partial_error!(PoisonedMutex),
            Errno::EOWNERDEAD => {
                // poison the mutex permanently on purpose by not calling pthread_mutex_consistent
                // and unlocking it
                let _ =
                    unsafe { pthread_mutex_unlock(self.0.mutex.get().cast::<pthread_mutex_t>()) };
                ffi_partial_error!(PoisonedMutex)
            }
            e => ffi_partial_error_with_details!(MutexError, e.to_string()),
        })
    }

    fn lock_lenient(&self) -> Result<(), PartialError> {
        pthread_result_errno!(unsafe {
            pthread_mutex_lock(self.0.mutex.get().cast::<pthread_mutex_t>())
        })
        .or_else(|e| match e {
            Errno::EDEADLK => Err(ffi_partial_error!(MutexNestedLock)),
            Errno::ENOTRECOVERABLE => Err(ffi_partial_error!(PoisonedMutex)),
            Errno::EOWNERDEAD => {
                // not an actual error, just means the previous owner dies before unlocking
                // we just have to reclaim ownership again
                pthread_result_errno!(unsafe {
                    pthread_mutex_consistent(self.0.mutex.get().cast::<pthread_mutex_t>())
                })
                .map_err(|e| ffi_partial_error_with_details!(MutexError, e.to_string()))
            }
            e => Err(ffi_partial_error_with_details!(MutexError, e.to_string())),
        })
    }

    fn try_lock(&self) -> Result<(), PartialError> {
        pthread_result_errno!(unsafe {
            pthread_mutex_trylock(self.0.mutex.get().cast::<pthread_mutex_t>())
        })
        .map_err(|e| match e {
            Errno::EBUSY => ffi_partial_error!(MutexTrylockFailed),
            Errno::EDEADLK => ffi_partial_error!(MutexNestedLock),
            Errno::ENOTRECOVERABLE => ffi_partial_error!(PoisonedMutex),
            Errno::EOWNERDEAD => {
                // poison the mutex permanently on purpose by not calling pthread_mutex_consistent
                // and unlocking it
                let _ =
                    unsafe { pthread_mutex_unlock(self.0.mutex.get().cast::<pthread_mutex_t>()) };
                ffi_partial_error!(PoisonedMutex)
            }
            e => ffi_partial_error_with_details!(MutexError, e.to_string()),
        })
    }

    fn try_lock_lenient(&self) -> Result<(), PartialError> {
        pthread_result_errno!(unsafe {
            pthread_mutex_trylock(self.0.mutex.get().cast::<pthread_mutex_t>())
        })
        .or_else(|e| match e {
            Errno::EBUSY => Err(ffi_partial_error!(MutexTrylockFailed)),
            Errno::ENOTRECOVERABLE => Err(ffi_partial_error!(PoisonedMutex)),
            Errno::EOWNERDEAD => {
                // not an actual error, just means the previous owner dies before unlocking
                // we just have to reclaim ownership again
                pthread_result_errno!(unsafe {
                    pthread_mutex_consistent(self.0.mutex.get().cast::<pthread_mutex_t>())
                })
                .map_err(|e| ffi_partial_error_with_details!(MutexError, e.to_string()))
            }
            e => Err(ffi_partial_error_with_details!(MutexError, e.to_string())),
        })
    }

    fn unlock(&self) -> Result<(), PartialError> {
        pthread_result_errno!(unsafe {
            pthread_mutex_unlock(self.0.mutex.get().cast::<pthread_mutex_t>())
        })
        .map_err(|e| match e {
            Errno::EPERM => ffi_partial_error!(MutexUnauthorizedUnlock),
            e => ffi_partial_error_with_details!(MutexError, e.to_string()),
        })
    }
}

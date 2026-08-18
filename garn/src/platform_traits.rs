use crate::interface::error_handling::PartialError;
use std::thread::ThreadId;

pub trait PlatformEnvironment {
    fn new(name: &str) -> Result<impl PlatformEnvironment, PartialError>;

    #[must_use]
    fn get_owner_thread(&self) -> ThreadId;

    fn open_mutex(&mut self, name: &str) -> Result<*const impl PlatformMutex, PartialError>;
}

pub trait PlatformMutex {
    fn lock(&self) -> Result<(), PartialError>;
    fn unlock(&self) -> Result<(), PartialError>;
    fn try_lock(&self) -> Result<(), PartialError>;
}

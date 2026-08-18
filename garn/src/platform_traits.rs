use crate::interface::error_handling::{Error, PartialError};
use crate::linux::mutex::Mutex;
use garnshared::platform_traits::PlatformMutex;
use std::thread::ThreadId;

pub trait PlatformEnvironment {
    fn new(name: &str) -> Result<impl PlatformEnvironment, PartialError>;

    #[must_use]
    fn get_owner_thread(&self) -> ThreadId;

    #[must_use]
    fn open_mutex(&mut self, name: &str) -> Result<*const impl PlatformMutex, PartialError>;
}

use crate::linux::mutex::Mutex;
use std::thread::ThreadId;

pub trait PlatformEnvironment {
    #[must_use]
    fn new(name: &str) -> Self;

    #[must_use]
    fn get_owner_thread(&self) -> ThreadId;

    #[must_use]
    fn open_mutex(&mut self, name: &str) -> Option<&Mutex>;
}

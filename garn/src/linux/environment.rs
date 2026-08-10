use super::mutex::Mutex;
use crate::platform_traits::PlatformEnvironment;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::thread::{self, ThreadId};

pub struct Environment {
    owner_thread: ThreadId,
    name: String,
    // Box<> is strictly required here to guarantee referential integrity of the mutexes
    // across the FFI boundary.
    open_mutexes: HashMap<String, Box<Mutex>>,
}

impl PlatformEnvironment for Environment {
    fn new(name: &str) -> Self {
        Self {
            owner_thread: thread::current().id(),
            name: name.into(),
            open_mutexes: HashMap::new(),
        }
    }

    fn get_owner_thread(&self) -> ThreadId {
        self.owner_thread
    }

    fn open_mutex(&mut self, name: &str) -> Option<&Mutex> {
        /*
        match self.open_mutexes.entry(name.into()) {
            Entry::Occupied(_) => None,
            Entry::Vacant(e) => Some(&**e.insert(Box::new(Mutex::new()))),
        }
         */
        todo!();
    }
}

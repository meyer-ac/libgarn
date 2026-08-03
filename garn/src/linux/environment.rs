use std::collections::hash_map::Entry;
use std::collections::HashMap;
use super::mutex::Mutex;
use std::thread::{self, ThreadId};

pub struct Environment {
    owner_thread: ThreadId,
    name: String,
    open_mutexes: HashMap<String, Box<Mutex>>,
}

impl Environment {
    pub fn new(name: &str) -> Self {
        Self {
            owner_thread: thread::current().id(),
            name: name.into(),
            open_mutexes: HashMap::new(),
        }
    }

    pub fn get_owner_thread(&self) -> ThreadId {
        self.owner_thread
    }

    pub fn open_mutex(&mut self, name: &str) -> Option<&Mutex> {
        match self.open_mutexes.entry(name.into()) {
            Entry::Occupied(_) => None,
            Entry::Vacant(e) => Some(&**e.insert(Box::new(Mutex::new()))),
        }
    }
}
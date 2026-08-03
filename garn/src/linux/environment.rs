use super::mutex::Mutex;

pub struct Environment {
    name: String,
}

impl Environment {
    pub fn new(name: &str) -> Self {
        Self { name: name.into() }
    }

    pub fn open_mutex(&mut self, name: &str) -> Mutex {
        Mutex::new(name)
    }
}
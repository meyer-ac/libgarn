pub struct Mutex {
    name: String,
}

impl Mutex {
    pub fn new(name: &str) -> Self {
        Self {name: name.into()}
    }

    pub fn lock(&self) {

    }

    pub fn unlock(&self) {

    }

    pub fn try_lock(&self) {

    }
}
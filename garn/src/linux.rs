pub mod environment;
mod shm_consumer;

pub mod mutex {
    pub use garnshared::linux::pthread_mutex::PthreadMutex as Mutex;
}

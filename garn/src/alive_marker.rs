use crate::constants;
use crate::interface::error_handling::raise_unrecoverable_error_custom;
use nix::sys::signal::{Signal, raise};
use nix::unistd::pause;
use std::marker::PhantomData;
use std::ptr;
use std::rc::Rc;

#[repr(transparent)]
pub struct AliveMarker(u64, PhantomData<Rc<()>>); // Deliberately not thread safe

impl AliveMarker {
    pub fn new() -> Self {
        AliveMarker(constants::ALIVE_MARKER, PhantomData)
    }

    pub fn check(&self) {
        // SAFETY: If this function is called from safe Rust, reading this attribute is obviously safe.
        let marker = unsafe { ptr::read_volatile(&raw const self.0) };
        if marker != constants::ALIVE_MARKER {
            raise_unrecoverable_error_custom(
                "Tried to use a resource after its destruction.",
                || {
                    let _ = raise(Signal::SIGSEGV);
                    // For the abnormal case that the SIGSEGV signal got caught
                    pause();
                    std::process::abort();
                },
            );
        }
    }
}

impl Drop for AliveMarker {
    fn drop(&mut self) {
        self.0 = 0;
    }
}

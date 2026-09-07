use crate::interface::error_handling::PartialError;
use crate::{constants, ffi_partial_error};
use std::ptr;

#[repr(transparent)]
pub struct AliveMarker(u64);

impl AliveMarker {
    pub fn new() -> Self {
        AliveMarker(constants::ALIVE_MARKER)
    }

    pub fn check(&self) -> Result<(), PartialError> {
        let marker = unsafe { ptr::read_volatile(&raw const self.0) };
        if marker != constants::ALIVE_MARKER {
            return Err(ffi_partial_error!(UseAfterDestroy));
        }
        Ok(())
    }
}

impl Drop for AliveMarker {
    fn drop(&mut self) {
        self.0 = 0;
    }
}

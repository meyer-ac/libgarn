use std::ffi::c_char;

pub const ALIVE_MARKER: u64 = 0x6C69_6267_6172_6E00;
pub const META_ERROR_FUNCTION_NAME: *const c_char = c"!!! META-ERROR !!!".as_ptr();

use std::ffi::{c_char, CStr};

#[macro_export]
macro_rules! ffi_error {
    ($error_type:ident) => {
        crate::interface::error_handling::Error::new(garn_proc_macros::prefix_error_type!($error_type), FN_NAME)
    };

    ($error_type:ident, $arg:ident) => {
        crate::interface::error_handling::Error::with_arg(garn_proc_macros::prefix_error_type!($error_type), FN_NAME, garn_proc_macros::arg_name_const_identifier!($arg))
    };
}

#[macro_export]
macro_rules! ffi_no_error {
    () => {
        crate::interface::error_handling::ffi_error!(NoError)
    };
}

pub use {ffi_error, ffi_no_error};

pub fn raise_unrecoverable_error(message: &str) -> ! {
    eprintln!("garn: FATAL! {}", message);
    std::process::abort();
}

#[repr(usize)]
#[derive(Copy, Clone, PartialEq)]
pub enum ErrorType {
    NoError = 0,
    NonNullReferenceViolation = 1,
    InvalidString = 2,
}

impl ErrorType {
    fn get_c_error_message(self) -> *const c_char {
        match self {
            ErrorType::NoError => c"No error occurred.",
            ErrorType::NonNullReferenceViolation => c"A null-reference was provided to a garn-function expecting a non-null-reference.",
            ErrorType::InvalidString => c"An invalid UTF8-string was provided to a garn-function."
        }.as_ptr()
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Error {
    error_type: ErrorType,
    error_message: *const c_char,
    fn_name: *const c_char,
    arg_name: *const c_char,
}

impl Error {
    pub fn new(error_type: ErrorType, fn_name: *const c_char) -> Self {
        Error {
            error_type,
            error_message: error_type.get_c_error_message(),
            fn_name,
            arg_name: std::ptr::null()
        }
    }

    pub fn with_arg(error_type: ErrorType, fn_name: *const c_char, arg_name: *const c_char) -> Self {
        Error {
            error_type,
            error_message: error_type.get_c_error_message(),
            fn_name,
            arg_name
        }
    }

    unsafe fn get_error_message(&self) -> &str {
        if self.error_message.is_null() {
            raise_unrecoverable_error("Error object is in an invalid state: error_message is null.");
        }
        unsafe {CStr::from_ptr(self.error_message)}.to_str()
            .unwrap_or_else(|_| raise_unrecoverable_error("Error object is in an invalid state: error_message is not a valid string."))
    }

    unsafe fn get_fn_name(&self) -> &str {
        if self.fn_name.is_null() {
            raise_unrecoverable_error("Error object is in an invalid state: fn_name is null.");
        }
        unsafe {CStr::from_ptr(self.fn_name)}.to_str()
            .unwrap_or_else(|_| raise_unrecoverable_error("Error object is in an invalid state: fn_name is not a valid string."))
    }

    unsafe fn get_arg_name(&self) -> Option<&str> {
        if self.arg_name.is_null() {
            return None
        }
        Some(unsafe {CStr::from_ptr(self.arg_name)}.to_str()
        .unwrap_or_else(|_| raise_unrecoverable_error("Error object is in an invalid state: arg_name is not a valid string.")))
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn garn_error_is_ok(error: Error) -> bool {
    error.error_type == ErrorType::NoError
}

#[unsafe(no_mangle)]
pub extern "C" fn garn_error_get_code(error: Error) -> usize {
    error.error_type as usize
}

#[unsafe(no_mangle)]
pub extern "C" fn garn_error_get_message(error: Error) -> *const c_char {
    error.error_message
}

#[unsafe(no_mangle)]
pub extern "C" fn garn_error_get_function(error: Error) -> *const c_char {
    error.fn_name
}

#[unsafe(no_mangle)]
pub extern "C" fn garn_error_get_argument(error: Error) -> *const c_char {
    error.arg_name
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn garn_error_print(error: Error) {
    eprintln!("garn: {}", unsafe {error.get_error_message()});
    eprintln!("      In function: {}", unsafe {error.get_fn_name()});
    if let Some(arg_name) = unsafe {error.get_arg_name()} {
        eprintln!("      Responsible argument: {}", arg_name);
    }
}

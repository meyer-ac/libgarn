use std::ffi::{CStr, CString, c_char};

#[macro_export]
macro_rules! ffi_partial_error {
    ($error_type:ident) => {
        $crate::interface::error_handling::PartialError::new(garn_proc_macros::prefix_error_type!(
            $error_type
        ))
    };
}

#[macro_export]
macro_rules! ffi_partial_error_with_details {
    ($error_type:ident, $details:expr) => {
        $crate::interface::error_handling::PartialError::with_details(
            garn_proc_macros::prefix_error_type!($error_type),
            $details,
        )
    };
}

#[macro_export]
macro_rules! ffi_error {
    ($error_type:ident) => {
        ::std::boxed::Box::into_raw(::std::boxed::Box::new(
            $crate::interface::error_handling::Error::new(
                garn_proc_macros::prefix_error_type!($error_type),
                garn_proc_macros::fn_name_const_identifier!(),
            ),
        ))
    };
}

#[macro_export]
macro_rules! ffi_error_with_arg {
    ($error_type:ident, $arg:ident) => {
        ::std::boxed::Box::into_raw(::std::boxed::Box::new(
            $crate::interface::error_handling::Error::with_arg(
                garn_proc_macros::prefix_error_type!($error_type),
                garn_proc_macros::fn_name_const_identifier!(),
                garn_proc_macros::arg_name_const_identifier!($arg),
            ),
        ))
    };
}

#[macro_export]
macro_rules! ffi_error_from_partial {
    ($partial:expr) => {
        ::std::boxed::Box::into_raw(::std::boxed::Box::new(
            $crate::interface::error_handling::Error::from_partial(
                $partial,
                garn_proc_macros::fn_name_const_identifier!(),
            ),
        ))
    };
}

#[macro_export]
macro_rules! ffi_error_from_partial_with_arg {
    ($partial:expr, $arg:ident) => {
        ::std::boxed::Box::into_raw(::std::boxed::Box::new(
            $crate::interface::error_handling::Error::from_partial_with_arg(
                $partial,
                garn_proc_macros::fn_name_const_identifier!(),
                garn_proc_macros::arg_name_const_identifier!($arg),
            ),
        ))
    };
}

#[macro_export]
macro_rules! ffi_no_error {
    () => {
        ::std::ptr::null_mut()
    };
}

pub use {
    ffi_error, ffi_error_from_partial, ffi_error_from_partial_with_arg, ffi_error_with_arg,
    ffi_no_error, ffi_partial_error_with_details,
};

pub fn raise_unrecoverable_error(message: &str) -> ! {
    eprintln!("garn: FATAL! {message}");
    std::process::abort();
}

#[repr(usize)]
#[derive(Copy, Clone, PartialEq)]
pub enum ErrorType {
    NonNullReferenceViolation = 1,
    InvalidString = 2,
    MutexAlreadyOpened = 3,
    ThreadOwnershipViolation = 4,
    ServiceCommunicationFailed = 5,
    NameTooLong = 6,
    SharedMemoryError = 7,
    GetPageSizeFailed = 8,
}

impl ErrorType {
    const fn get_c_error_message(self) -> *const c_char {
        match self {
            Self::NonNullReferenceViolation => c"A null-reference was provided to a garn-function expecting a non-null-reference.",
            Self::InvalidString => c"An invalid UTF8-string was provided to a garn-function.",
            Self::MutexAlreadyOpened => c"Tried to open the same mutex twice.",
            Self::ThreadOwnershipViolation => c"A thread different from that which created the resource tried to modify or destroy it.",
            Self::ServiceCommunicationFailed => c"Communication with the garnd service failed.",
            Self::NameTooLong => c"The provided name was too long.",
            Self::SharedMemoryError => c"Failed to map the service's shared memory into the local address space.",
            Self::GetPageSizeFailed => c"Could not determine the system's page size.",
        }.as_ptr()
    }
}

pub struct PartialError {
    error_type: ErrorType,
    details: Option<String>,
}

impl PartialError {
    pub fn new(error_type: ErrorType) -> Self {
        Self {
            error_type,
            details: None,
        }
    }

    pub fn with_details(error_type: ErrorType, details: String) -> Self {
        Self {
            error_type,
            details: Some(details),
        }
    }
}

pub struct Error {
    #[allow(clippy::struct_field_names)] // `type` is not a valid identifier
    error_type: ErrorType,
    message: *const c_char,
    fn_name: *const c_char,
    arg_name: *const c_char,
    details: Option<CString>,
}

impl Error {
    pub fn new(error_type: ErrorType, fn_name: *const c_char) -> Self {
        Self {
            error_type,
            message: error_type.get_c_error_message(),
            fn_name,
            arg_name: std::ptr::null(),
            details: None,
        }
    }

    pub fn with_arg(
        error_type: ErrorType,
        fn_name: *const c_char,
        arg_name: *const c_char,
    ) -> Self {
        Self {
            error_type,
            message: error_type.get_c_error_message(),
            fn_name,
            arg_name,
            details: None,
        }
    }

    pub fn from_partial(partial_error: PartialError, fn_name: *const c_char) -> Self {
        let details_c = partial_error.details.map(|details| {
            CString::new(details).unwrap_or_else(|_| {
            raise_unrecoverable_error(
                "Error object is in an invalid state: details contains an internal null character.",
            )
        })
        });
        Self {
            error_type: partial_error.error_type,
            message: partial_error.error_type.get_c_error_message(),
            fn_name,
            arg_name: std::ptr::null(),
            details: details_c,
        }
    }

    pub fn from_partial_with_arg(
        partial_error: PartialError,
        fn_name: *const c_char,
        arg_name: *const c_char,
    ) -> Self {
        let details_c = partial_error.details.map(|details| {
            CString::new(details).unwrap_or_else(|_| {
            raise_unrecoverable_error(
                "Error object is in an invalid state: details contains an internal null character.",
            )
        })
        });
        Self {
            error_type: partial_error.error_type,
            message: partial_error.error_type.get_c_error_message(),
            fn_name,
            arg_name,
            details: details_c,
        }
    }

    unsafe fn get_message(&self) -> &str {
        if self.message.is_null() {
            raise_unrecoverable_error(
                "Error object is in an invalid state: error_message is null.",
            );
        }
        unsafe { CStr::from_ptr(self.message) }
            .to_str()
            .unwrap_or_else(|_| {
                raise_unrecoverable_error(
                    "Error object is in an invalid state: error_message is not a valid string.",
                )
            })
    }

    unsafe fn get_fn_name(&self) -> &str {
        if self.fn_name.is_null() {
            raise_unrecoverable_error("Error object is in an invalid state: fn_name is null.");
        }
        unsafe { CStr::from_ptr(self.fn_name) }
            .to_str()
            .unwrap_or_else(|_| {
                raise_unrecoverable_error(
                    "Error object is in an invalid state: fn_name is not a valid string.",
                )
            })
    }

    unsafe fn get_arg_name(&self) -> Option<&str> {
        if self.arg_name.is_null() {
            return None;
        }
        Some(
            unsafe { CStr::from_ptr(self.arg_name) }
                .to_str()
                .unwrap_or_else(|_| {
                    raise_unrecoverable_error(
                        "Error object is in an invalid state: arg_name is not a valid string.",
                    )
                }),
        )
    }

    unsafe fn get_details(&self) -> Option<&str> {
        self.details.as_deref().map(|details| {
            details.to_str().unwrap_or_else(|_| {
                raise_unrecoverable_error(
                    "Error object is in an invalid state: details is not a valid string.",
                )
            })
        })
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn garn_error_get_code(error: *const Error) -> usize {
    unsafe { &*error }.error_type as usize
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn garn_error_get_message(error: *const Error) -> *const c_char {
    unsafe { &*error }.message
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn garn_error_get_function(error: *const Error) -> *const c_char {
    unsafe { &*error }.fn_name
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn garn_error_get_argument(error: *const Error) -> *const c_char {
    unsafe { &*error }.arg_name
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn garn_error_get_details(error: *const Error) -> *const c_char {
    match unsafe { &*error }.details.as_ref() {
        Some(details) => details.as_ptr(),
        None => std::ptr::null(),
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn garn_error_handled(error: *mut Error) {
    if let Some(error_details) = unsafe { Box::from_raw(error) }.details {
        drop(error_details);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn garn_error_print(error: *const Error) {
    let error = unsafe { &*error };
    eprintln!("garn: {}", unsafe { error.get_message() });
    eprintln!("      In function: {}", unsafe { error.get_fn_name() });
    if let Some(arg_name) = unsafe { error.get_arg_name() } {
        eprintln!("      Responsible argument: {arg_name}");
    }
    if let Some(details) = unsafe { error.get_details() } {
        eprintln!("      Additional details: {details}");
    }
}

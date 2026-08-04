#[doc(hidden)]
pub mod __macro_support {
    pub use crate::interface::error_handling::ErrorType;
}

mod interface;

cfg_if::cfg_if! {
    if #[cfg(target_os="linux")] {
        mod linux;
        pub use linux::environment;
        pub use linux::mutex;
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}

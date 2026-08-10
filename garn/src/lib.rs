#![warn(
    clippy::all,
    //clippy::restriction,
    clippy::pedantic,
    //clippy::nursery,
    //clippy::cargo
)]

#[doc(hidden)]
pub mod __macro_support {
    pub use crate::interface::error_handling::ErrorType;
}

mod interface;
mod platform_traits;

cfg_if::cfg_if! {
    if #[cfg(target_os="linux")] {
        mod linux;
        use linux::environment;
        use linux::mutex;
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}

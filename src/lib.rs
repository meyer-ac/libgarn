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
    fn it_works() {

    }
}

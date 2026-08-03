fn main() {
    const CONFIG_FILE: &str = "cbindgen.toml";

    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();

    let config = cbindgen::Config::from_file(CONFIG_FILE)
        .unwrap_or_else(|_| panic!("Unable to open {}", CONFIG_FILE));

    cbindgen::Builder::new()
        .with_crate(crate_dir)
        .with_config(config)
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file("include/libgarn.h");
}
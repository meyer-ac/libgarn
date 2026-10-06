use std::env;
use std::path::PathBuf;

fn main() {
    const CONFIG_FILE: &str = "cbindgen.toml";
    const GARN_LIB_SRC_RELATIVE: &str = "../garn/src/lib.rs";

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let garn_lib_src = PathBuf::from(&manifest_dir).join(GARN_LIB_SRC_RELATIVE);

    let config = cbindgen::Config::from_file(CONFIG_FILE)
        .unwrap_or_else(|_| panic!("Unable to open {}", CONFIG_FILE));

    cbindgen::Builder::new()
        .with_src(garn_lib_src)
        .with_config(config)
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file("../include/libgarn.h");
}

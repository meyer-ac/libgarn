use std::env;
use std::path::{Path, PathBuf};

fn main() {
    const C_INCLUDE_RELATIVE: &str = "../garn/c/include";
    const C_SRC_RELATIVE: &str = "../garn/c/src";
    const C_SRC_FILES: &[&str] = &["tls.c"];

    let lto_mode = env::var("LIBGARN_LTO_MODE").unwrap_or_else(|_| "universal".to_owned());

    let mut tls = cc::Build::new();
    tls.opt_level(3);
    tls.include(C_INCLUDE_RELATIVE);
    tls.define("LIBGARN_TARGET_DYNAMIC", None);

    let c_src = PathBuf::from(C_SRC_RELATIVE);
    for src_file in C_SRC_FILES {
        tls.file(c_src.join(Path::new(src_file)));
    }

    #[allow(clippy::single_match)]
    match lto_mode.as_str() {
        "llvm-lto" => {
            tls.flag("-flto=full");
            tls.flag("-Xclang");
            tls.flag("-fembed-bitcode=all");
        }
        _ => {}
    }

    tls.compile("tls");
}

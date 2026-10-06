use std::path::{Path, PathBuf};

fn main() {
    const C_INCLUDE_RELATIVE: &str = "../garn/c/include";
    const C_SRC_RELATIVE: &str = "../garn/c/src";
    const C_SRC_FILES: [&str; 1] = ["tls.c"];

    let mut tls = cc::Build::new();
    tls.include(C_INCLUDE_RELATIVE);
    tls.define("LIBGARN_TARGET_DYNAMIC", None);

    let c_src = PathBuf::from(C_SRC_RELATIVE);
    for src_file in C_SRC_FILES {
        tls.file(c_src.join(Path::new(src_file)));
    }

    tls.compile("tls");
}

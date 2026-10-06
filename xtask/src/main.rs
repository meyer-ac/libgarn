use std::env::consts::OS;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const HEADERS_PACKAGE_NAME: &str = "garnheaders";
const DIST_DIR: &str = "dist";
const GENERATED_HEADERS_PATH: &str = "include";
const HEADER_NAMES: &[&str] = &["libgarn.h"];
const RESULTING_LIB_NAME: &str = "libgarn";
const STATIC_LIB_NAME: &str = "garnstatic";
const DYNAMIC_LIB_NAME: &str = "garndynamic";

#[derive(Copy, Clone)]
enum LibFormat {
    Static,
    Dynamic,
}

impl LibFormat {
    fn crate_name(&self) -> &'static str {
        match self {
            LibFormat::Static => STATIC_LIB_NAME,
            LibFormat::Dynamic => DYNAMIC_LIB_NAME,
        }
    }
}

#[derive(Copy, Clone)]
enum LtoMode {
    Universal,
    Llvm,
}

impl LtoMode {
    fn lto_name(&self) -> &'static str {
        match self {
            LtoMode::Universal => "universal",
            LtoMode::Llvm => "llvm-lto",
        }
    }
}

#[derive(Copy, Clone)]
struct MArchTarget {
    pub target: &'static str,
    pub march: &'static str,
    pub target_cpu: &'static str,
    pub lto_mode: LtoMode,
}

impl MArchTarget {
    pub fn full_name(&self) -> String {
        format!(
            "{}-{}-{}",
            self.target,
            self.march,
            self.lto_mode.lto_name()
        )
    }
}

macro_rules! march_target {
    ($target:expr, $march:expr, $target_cpu:expr, $lto_mode:expr) => {
        MArchTarget {
            target: $target,
            march: $march,
            target_cpu: $target_cpu,
            lto_mode: $lto_mode,
        }
    };
}

const TARGETS: &[MArchTarget] = &[
    march_target!(
        "x86_64-unknown-linux-gnu",
        "generic",
        "x86-64",
        LtoMode::Universal
    ),
    march_target!(
        "x86_64-unknown-linux-gnu",
        "v2",
        "x86-64-v2",
        LtoMode::Universal
    ),
    march_target!(
        "x86_64-unknown-linux-gnu",
        "v3",
        "x86-64-v3",
        LtoMode::Universal
    ),
    march_target!(
        "x86_64-unknown-linux-gnu",
        "v4",
        "x86-64-v4",
        LtoMode::Universal
    ),
    march_target!(
        "x86_64-unknown-linux-gnu",
        "generic",
        "x86-64",
        LtoMode::Llvm
    ),
    march_target!("x86_64-unknown-linux-gnu", "v2", "x86-64-v2", LtoMode::Llvm),
    march_target!("x86_64-unknown-linux-gnu", "v3", "x86-64-v3", LtoMode::Llvm),
    march_target!("x86_64-unknown-linux-gnu", "v4", "x86-64-v4", LtoMode::Llvm),
    //
    march_target!(
        "aarch64-unknown-linux-gnu",
        "generic",
        "generic",
        LtoMode::Universal
    ),
    march_target!(
        "aarch64-unknown-linux-gnu",
        "neoverse-n1",
        "neoverse-n1",
        LtoMode::Universal
    ),
    march_target!(
        "aarch64-unknown-linux-gnu",
        "neoverse-v1",
        "neoverse-v1",
        LtoMode::Universal
    ),
    march_target!(
        "aarch64-unknown-linux-gnu",
        "generic",
        "generic",
        LtoMode::Llvm
    ),
    march_target!(
        "aarch64-unknown-linux-gnu",
        "neoverse-n1",
        "neoverse-n1",
        LtoMode::Llvm
    ),
    march_target!(
        "aarch64-unknown-linux-gnu",
        "neoverse-v1",
        "neoverse-v1",
        LtoMode::Llvm
    ),
    //
    march_target!(
        "x86_64-unknown-linux-musl",
        "generic",
        "x86-64",
        LtoMode::Universal
    ),
    march_target!(
        "x86_64-unknown-linux-musl",
        "v2",
        "x86-64-v2",
        LtoMode::Universal
    ),
    march_target!(
        "x86_64-unknown-linux-musl",
        "v3",
        "x86-64-v3",
        LtoMode::Universal
    ),
    march_target!(
        "x86_64-unknown-linux-musl",
        "v4",
        "x86-64-v4",
        LtoMode::Universal
    ),
    march_target!(
        "x86_64-unknown-linux-musl",
        "generic",
        "x86-64",
        LtoMode::Llvm
    ),
    march_target!(
        "x86_64-unknown-linux-musl",
        "v2",
        "x86-64-v2",
        LtoMode::Llvm
    ),
    march_target!(
        "x86_64-unknown-linux-musl",
        "v3",
        "x86-64-v3",
        LtoMode::Llvm
    ),
    march_target!(
        "x86_64-unknown-linux-musl",
        "v4",
        "x86-64-v4",
        LtoMode::Llvm
    ),
    //
    march_target!(
        "aarch64-unknown-linux-musl",
        "generic",
        "generic",
        LtoMode::Universal
    ),
    march_target!(
        "aarch64-unknown-linux-musl",
        "neoverse-n1",
        "neoverse-n1",
        LtoMode::Universal
    ),
    march_target!(
        "aarch64-unknown-linux-musl",
        "neoverse-v1",
        "neoverse-v1",
        LtoMode::Universal
    ),
    march_target!(
        "aarch64-unknown-linux-musl",
        "generic",
        "generic",
        LtoMode::Llvm
    ),
    march_target!(
        "aarch64-unknown-linux-musl",
        "neoverse-n1",
        "neoverse-n1",
        LtoMode::Llvm
    ),
    march_target!(
        "aarch64-unknown-linux-musl",
        "neoverse-v1",
        "neoverse-v1",
        LtoMode::Llvm
    ),
    //
    //march_target!("x86_64-pc-windows-msvc", "generic", "x86-64", LtoMode::Universal),
    //march_target!("x86_64-pc-windows-msvc", "v2", "x86-64-v2", LtoMode::Universal),
    //march_target!("x86_64-pc-windows-msvc", "v3", "x86-64-v3", LtoMode::Universal),
    //march_target!("x86_64-pc-windows-msvc", "v4", "x86-64-v4", LtoMode::Universal),
    //
    //march_target!("aarch64-pc-windows-msvc", "generic", "generic", LtoMode::Universal),
    //march_target!("aarch64-pc-windows-msvc", "neoverse-n1", "neoverse-n1", LtoMode::Universal),
    //march_target!("aarch64-pc-windows-msvc", "neoverse-v1", "neoverse-v1", LtoMode::Universal),
    //
    //march_target!("x86_64-pc-windows-gnu", "generic", "x86-64", LtoMode::Universal),
    //march_target!("x86_64-pc-windows-gnu", "v2", "x86-64-v2", LtoMode::Universal),
    //march_target!("x86_64-pc-windows-gnu", "v3", "x86-64-v3", LtoMode::Universal),
    //march_target!("x86_64-pc-windows-gnu", "v4", "x86-64-v4", LtoMode::Universal),
    //march_target!("x86_64-pc-windows-gnu", "generic", "x86-64", LtoMode::Llvm),
    //march_target!("x86_64-pc-windows-gnu", "v2", "x86-64-v2", LtoMode::Llvm),
    //march_target!("x86_64-pc-windows-gnu", "v3", "x86-64-v3", LtoMode::Llvm),
    //march_target!("x86_64-pc-windows-gnu", "v4", "x86-64-v4", LtoMode::Llvm),
    //
    //march_target!("x86_64-pc-apple-darwin", "generic", "x86-64", LtoMode::Universal),
    //march_target!("x86_64-pc-apple-darwin", "v2", "x86-64-v2", LtoMode::Universal),
    //march_target!("x86_64-pc-apple-darwin", "v3", "x86-64-v3", LtoMode::Universal),
    //march_target!("x86_64-pc-apple-darwin", "v4", "x86-64-v4", LtoMode::Universal),
    //march_target!("x86_64-pc-apple-darwin", "generic", "x86-64", LtoMode::Llvm),
    //march_target!("x86_64-pc-apple-darwin", "v2", "x86-64-v2", LtoMode::Llvm),
    //march_target!("x86_64-pc-apple-darwin", "v3", "x86-64-v3", LtoMode::Llvm),
    //march_target!("x86_64-pc-apple-darwin", "v4", "x86-64-v4", LtoMode::Llvm),
    //
    //march_target!("aarch64-apple-darwin", "apple-m1", "apple-m1", LtoMode::Universal),
    //march_target!("aarch64-apple-darwin", "apple-m1", "apple-m1", LtoMode::Llvm),
];

fn main() {
    println!("Cleaning dist directory...");
    let dist_dir = Path::new(DIST_DIR);
    let _ = fs::remove_dir_all(dist_dir);
    fs::create_dir_all(dist_dir).unwrap();
    println!("dist directory cleaned.");

    ensure_zigbuild_installed();

    println!("Generating headers...");
    Command::new("cargo")
        .args(["build", "--release", "--package", HEADERS_PACKAGE_NAME])
        .status()
        .expect("Failed to generate headers.");
    println!("Headers generated.");

    println!("Copying headers...");
    for header in HEADER_NAMES {
        fs::copy(
            Path::new(GENERATED_HEADERS_PATH).join(header),
            dist_dir.join(header),
        )
        .ok();
    }
    println!("Headers copied.");

    println!("Building static libraries...");
    for target in TARGETS {
        build_target(*target, LibFormat::Static, dist_dir);
    }
    println!("Static libraries built.");

    println!("Building dynamic libraries...");
    for target in TARGETS {
        build_target(*target, LibFormat::Dynamic, dist_dir);
    }
    println!("Dynamic libraries built.");
    println!("Everything done.");
}

fn ensure_zigbuild_installed() {
    println!("Checking for presence of Zig...");
    let output = Command::new("zig").args(["version"]).output();

    let is_installed = match output {
        Ok(out) => out.status.success(),
        Err(_) => false,
    };

    if is_installed {
        println!("Zig is installed.");
    } else {
        panic!("Zig is not installed.");
    }

    println!("Checking for presence of zigbuild");
    let output = Command::new("cargo")
        .args(["zigbuild", "--version"])
        .output();

    let is_installed = match output {
        Ok(out) => out.status.success(),
        Err(_) => false,
    };

    if is_installed {
        println!("zigbuild is already installed.");
    } else {
        println!("Installing zigbuild...");
        Command::new("cargo")
            .args(["install", "cargo-zigbuild"])
            .status()
            .expect("Failed to install zigbuild.");
        println!("zigbuild installed.");
    }
}

fn build_target(target: MArchTarget, format: LibFormat, dist_dir: &Path) {
    println!(
        "Building {} library for target {}...",
        match format {
            LibFormat::Static => "static",
            LibFormat::Dynamic => "dynamic",
        },
        target.full_name()
    );

    let host_os = OS;

    let is_apple = target.target.contains("apple");
    let is_msvc = target.target.contains("msvc");

    if is_apple && host_os != "macos" {
        println!(
            "Skipping {}: Apple targets must be built on macOS.",
            target.full_name()
        );
        return;
    }

    if is_msvc && host_os != "windows" {
        println!(
            "Skipping {}: MSVC targets must be built on Windows.",
            target.full_name()
        );
        return;
    }

    let use_zigbuild = !is_apple && !is_msvc;

    Command::new("rustup")
        .args(["target", "add", target.target])
        .status()
        .ok();

    let cargo_cmd = if use_zigbuild { "zigbuild" } else { "build" };

    let mut rustflags = format!("-C target-cpu={}", target.target_cpu);
    rustflags.push_str(" -C opt-level=3");
    match target.lto_mode {
        LtoMode::Universal => {
            rustflags.push_str(" -C lto=off");
            rustflags.push_str(" -C embed-bitcode=no");
        }
        LtoMode::Llvm => {
            rustflags.push_str(" -C lto=fat");
            rustflags.push_str(" -C linker-plugin-lto");
            rustflags.push_str(" -C embed-bitcode=yes");
        }
    }

    if target.target.contains("musl") && matches!(format, LibFormat::Dynamic) {
        rustflags.push_str(" -C target-feature=-crt-static");
    }

    Command::new("cargo")
        .args([
            cargo_cmd,
            "--release",
            "--package",
            format.crate_name(),
            "--target",
            target.target,
        ])
        .env("RUSTFLAGS", rustflags)
        .env("LIBGARN_LTO_MODE", target.lto_mode.lto_name())
        .status()
        .unwrap_or_else(|_| {
            panic!(
                "Failed to build {} for {}",
                format.crate_name(),
                target.full_name()
            )
        });

    stage_artifact(target, format, dist_dir);
}

fn stage_artifact(target: MArchTarget, format: LibFormat, dist_dir: &Path) {
    let target_dir = dist_dir.join(target.full_name());
    fs::create_dir_all(&target_dir).unwrap();

    let is_windows = target.target.contains("windows");
    let is_macos = target.target.contains("apple");

    let ext = match format {
        LibFormat::Static => {
            if is_windows {
                "lib"
            } else {
                "a"
            }
        }
        LibFormat::Dynamic => {
            if is_windows {
                "dll"
            } else if is_macos {
                "dylib"
            } else {
                "so"
            }
        }
    };

    let filename = format!("{RESULTING_LIB_NAME}.{ext}");
    let artifact_path: PathBuf = ["target", target.target, "release", &filename]
        .iter()
        .collect();
    let dest_path = target_dir.join(&filename);

    if artifact_path.exists() {
        fs::copy(&artifact_path, &dest_path).expect("Failed to copy artifact.");
    } else {
        panic!("Expected artifact not found at {}.", filename);
    }

    if is_windows && matches!(format, LibFormat::Dynamic) {
        let import_lib = if target.target.contains("msvc") {
            format!("{RESULTING_LIB_NAME}.dll.lib")
        } else {
            format!("{RESULTING_LIB_NAME}.dll.a")
        };
        let import_artifact: PathBuf = ["target", target.target, "release", &import_lib]
            .iter()
            .collect();
        if import_artifact.exists() {
            fs::copy(&import_artifact, target_dir.join(import_lib))
                .expect("Failed to copy artifact.");
        }
    }
    println!("Build successful.");
}

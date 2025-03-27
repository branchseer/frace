use std::{path::Path, process::Command};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    match target_os.as_str() {
        "macos" => {
            println!("cargo::rerun-if-env-changed=PATH");
            println!("cargo::rerun-if-changed=src/macos/preload.c");
            let clang_path  = which::which("clang").expect("clang is not found in PATH");
            let out_path = Path::new(&std::env::var_os("OUT_DIR").unwrap()).join("frace_preload.dylib");
            let status = Command::new(clang_path)
                .args(&[
                    "-O3",
                    "-arch", "arm64",
                    "-arch", "x86_64",
                    "-dynamiclib",
                    "-Werror",
                    "-Wall",
                    "-std=c++11", "-lc++",
                    "-mmacosx-version-min=10.8",
                    "-o", out_path.to_str().unwrap(),
                    "src/macos/preload.cc"
                ])
                .status()
                .expect("failed to compile frace_preload.dylib");
            assert!(status.success());
        },
        _ => {},
    }
}

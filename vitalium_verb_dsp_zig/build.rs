use std::{os::unix::process::CommandExt, path::PathBuf, process::Command};

const ZIG_FILE_MAIN: &str = "main.zig";

fn main() {
    println!("cargo::rerun-if-changed=src");

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let src_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let lib_name = std::env::var("CARGO_PKG_NAME").unwrap();

    // set cargo search directory
    println!("cargo:rustc-link-search=native={}", out_dir);

    // set cargo lib name
    println!("cargo:rustc-link-lib={}={}", "static", lib_name);

    let mut file = PathBuf::from(src_dir);
    file.push("src");
    file.push(ZIG_FILE_MAIN);

    let target = {
        let split_target = std::env::var("TARGET")
            .expect("TARGET to be set by cargo")
            .split('-')
            .map(String::from)
            .collect::<Vec<_>>();

        format!(
            "{}-{}-{}",
            split_target[0], split_target[2], split_target[3]
        )
    };

    let profile = match std::env::var("PROFILE").unwrap().as_str() {
        "release" => "ReleaseSafe".to_string(),
        "debug" => "Debug".to_string(),
        _ => unreachable!("Invalid cargo PROFILE env"),
    };

    let mut emit_name = PathBuf::from(&out_dir);
    emit_name.push(format!("lib{}.a", lib_name));

    let mut command = Command::new("zig");

    command.arg("build-lib");
    // build as a static library
    command.arg("-static");
    // change the default SONAME to the specified lib name
    command.arg("--name");
    command.arg(lib_name);
    // set the output directory of the build cache
    command.arg("--cache-dir");
    command.arg(&out_dir);
    // set the location and name of the output binary
    command.arg(format!("-femit-bin={}", emit_name.display()));
    // set the platform target
    command.arg("-target");
    command.arg(target);
    // set the optimization level
    command.arg("-O");
    command.arg(&profile);

    // fixes linking issues
    command.arg("-fPIC");

    if &profile == "Debug" {
        // statically include the compiler safety checks when in debug mode
        command.arg("-fcompiler-rt");
    }

    // set the file to be compiled and linked
    command.arg(file);

    command.exec();
}

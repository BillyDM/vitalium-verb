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

    let mut commands = Command::new("zig");

    commands.arg("build-lib");
    commands.arg("-static");
    // changes the default SONAME to the specified lib name
    commands.arg("--name");
    commands.arg(lib_name);
    // set the output directory of the build cache
    commands.arg("--cache-dir");
    commands.arg(&out_dir);
    // set the output directy of the binary
    commands.arg(format!("-femit-bin={}", emit_name.display()));
    // set the target directory
    commands.arg("-target");
    commands.arg(target);
    // set the compilation mode
    commands.arg("-O");
    commands.arg(&profile);

    // fixes linking issues
    commands.arg("-fPIC");

    if &profile == "Debug" {
        // statically include the compiler safety checks when in debug mode
        commands.arg("-fcompiler-rt");
    }

    // set the file to be compiled and linked
    commands.arg(file);

    commands.exec();
}

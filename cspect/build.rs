use std::env;
use std::path::{Path, PathBuf};

fn main() {
    // ==== Find svdpi.h =======================================================

    // Ensure svdpi.h file exists
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    let svdpi_path = PathBuf::from(crate_dir.clone()).join(Path::new("svdpi.h"));
    if !svdpi_path.exists() {
        panic!(
            "[cspect_dpi/build.rs]: Could not find svdpi.h header at {}.",
            svdpi_path.to_string_lossy()
        )
    }

    // Register svdpi.h as build dependency (although it really should
    // never change):
    println!("cargo:rerun-if-changed={}", svdpi_path.to_string_lossy());

    // ==== Generate rust types/bindings for svdpi.h ===========================

    // Generate rust bindings of types in `svdpi.h`:
    println!("cargo:rerun-if-changed=bindgen.rs");
    let bindings = bindgen::Builder::default()
        .header("bindgen.h")
        .clang_arg(format!("-I{}", crate_dir))
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    // ==== Generate C header file with DPI function signatures ================

    // Register re-generation if relevant files change:
    println!("cargo:rerun-if-changed=src/dpi.rs");
    println!("cargo:rerun-if-changed=cbindgen.toml");

    // Put output in `target/$BUILD_TYPE/cspect_dpi.h`
    // `$OUT_DIR` is typically: `target/$BUILD_TYPE/build/crate-name-hash/out`
    // Go up 3 levels to reach `target/$BUILD_TYPE`
    let mut output_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    for _ in 0..3 {
        output_path.pop();
    }
    output_path.push("cspect_dpi.h");
    eprintln!(
        "[cspect_dpi/build.rs]: Generating header file at {}.",
        output_path.to_string_lossy()
    );

    cbindgen::Builder::new()
        .with_crate(crate_dir)
        .with_config(
            cbindgen::Config::from_file("cbindgen.toml").expect("Failed to parse cbindgen.toml"),
        )
        .generate()
        .expect("Unable to generate C header.")
        .write_to_file(output_path);
}

use std::env;
use std::path::PathBuf;

fn main() {
    // ==== Generate C header file with DPI function signatures ================

    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

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

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

fn main() {
    // Put `link.x` in our output directory and ensure it's
    // on the linker search path.
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());

    println!("cargo:rustc-link-search={}", out.display());

    // Call another function to copy the linker script into the output directory.
    // This allows us to use a different linker script based on whether the
    // cmsis-pack-compat feature is enabled or not
    setup_linker_script(out);
}

#[cfg(not(feature = "cmsis-pack-compat"))]
fn setup_linker_script(out: &Path) {
    // rename the linker output to ensure we use the one in the
    // build output instead of the one in the project root
    File::create(out.join("link_generated.x"))
        .unwrap()
        .write_all(include_bytes!("link.x"))
        .unwrap();
    // By default, Cargo will re-run a build script whenever
    // any file in the project changes. By specifying `link.x`
    // here, we ensure the build script is only re-run when
    // `link.x` is changed.
    println!("cargo:rerun-if-changed=link.x");
}

#[cfg(feature = "cmsis-pack-compat")]
fn setup_linker_script(out: &Path) {
    File::create(out.join("link_generated.x"))
        .unwrap()
        .write_all(include_bytes!("link_metadata.x"))
        .unwrap();
    println!("cargo:rerun-if-changed=link_metadata.x");
}

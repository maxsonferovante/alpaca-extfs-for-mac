use std::env;
use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let vendor_dir = manifest_dir.join("vendor").join("e2fsprogs");
    let lib_dir = vendor_dir.join("lib");
    let include_dir = vendor_dir.join("include");

    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    println!("cargo:rustc-link-lib=static=ext2fs");
    println!("cargo:rustc-link-lib=static=com_err");
    println!("cargo:rustc-link-lib=static=e2p");
    println!("cargo:rustc-link-lib=static=uuid");

    println!("cargo:rerun-if-changed=build.rs");

    let bindings = bindgen::Builder::default()
        .header(include_dir.join("ext2fs").join("ext2fs.h").to_str().unwrap())
        .header(include_dir.join("com_err.h").to_str().unwrap())
        .clang_arg(format!("-I{}", include_dir.display()))
        .allowlist_function("ext2fs_.*")
        .allowlist_type("ext2_.*")
        .allowlist_type("ext2fs_.*")
        .allowlist_var("EXT2_.*")
        .allowlist_var("EXT3_.*")
        .allowlist_var("EXT4_.*")
        .allowlist_var(".*_io_manager")
        .generate_comments(false)

        .generate()
        .expect("Unable to generate libext2fs bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

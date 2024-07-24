use std::env;
use std::path::PathBuf;
use ultralight_build::UltralightBuild;

fn docsrs() -> PathBuf {
    PathBuf::from(env::var("MANIFEST_DIR").expect("MANIFEST_DIR not set")).join("Ultralight-API")
}

fn normal() -> PathBuf {
    let headers_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set")).join("headers");

    UltralightBuild::new()
        .download_headers()
        .with_headers_out_dir(headers_dir.clone())
        .build()
        .expect("Failed to download headers");

    headers_dir.clone()
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let headers_dir = if env::var("DOCS_RS").is_ok() {
        docsrs()
    } else {
        normal()
    };

    let bindings = bindgen::Builder::default()
        .header("src/wrapper.h")
        .clang_arg(format!("-I{}", headers_dir.display()))
        .impl_debug(true)
        .impl_partialeq(true)
        .generate_comments(true)
        .generate_inline_functions(true)
        .allowlist_var("^UL.*|JS.*|ul.*|WK.*|kJS.*")
        .allowlist_type("^UL.*|JS.*|ul.*|WK.*")
        .allowlist_function("^UL.*|JS.*|ul.*|WK.*")
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

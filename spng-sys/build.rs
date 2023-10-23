use std::env;

fn main() {
    let mut build = cc::Build::new();
    build.file("libspng/spng/spng.c");
    if let Some(libz_include) = env::var_os("DEP_Z_INCLUDE") {
        build.include(libz_include);
    }
    if cfg!(target_feature = "sse4.1") {
        build.define("SPNG_SSE", Some("4"));
    } else if cfg!(target_feature = "ssse3") {
        build.define("SPNG_SSE", Some("3"));
    }
    build.compile("spng");

    // DEP_SPNG_INCLUDE for other crates
    println!("cargo:include=libspng/spng");

    println!("cargo:rustc-link-lib=static={}", libname());
}

#[cfg(not(feature = "zlib-ng"))]
fn libname() -> &'static str {
    "z"
}

#[cfg(feature = "zlib-ng")]
fn libname() -> &'static str {
    let target = env::var("TARGET").unwrap();
    // Derived from: https://github.com/rust-lang/libz-sys/blob/36b3071331d9a87712c9d23fd7aea79208425c73/build.rs#L167
    if target.contains("windows") {
        if target.contains("msvc") && env::var("OPT_LEVEL").unwrap() == "0" {
            "zlibstaticd"
        } else {
            "zlibstatic"
        }
    } else {
        "z"
    }
}

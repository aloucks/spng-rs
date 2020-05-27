use std::env;

fn main() {
    let mut build = cc::Build::new();
    build.file("libspng/spng.c");
    if let Some(libz_include) = env::var_os("DEP_Z_INCLUDE") {
        build.include(libz_include);
    }
    if cfg!(target_feature = "ssse3") {
        build.define("SPNG_SSE", Some("3"));
    }
    build.compile("spng");

    // DEP_SPNG_INCLUDE
    println!("include=libspng");

    println!("cargo:rustc-link-lib=z");
}

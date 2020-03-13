use std::env;

fn main() {
    let libz_include = env::var_os("DEP_Z_INCLUDE").expect("DEP_Z_INCLUDE");
    cc::Build::new()
        .file("libspng/spng.c")
        .include(libz_include)
        .compile("spng");

    // DEP_SPNG_INCLUDE
    println!("include=libspng");
}

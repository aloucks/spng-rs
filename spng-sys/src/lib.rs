//! Native bindings to [libspng](https://libspng.org).

#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(clippy::upper_case_acronyms)]
mod ffi;

pub use ffi::*;

// Declaring this crate as extern is needed so that the Rust compiler thinks libz
// is used, and thus passes the expected parameters to get libz linked in. See:
// https://github.com/dtolnay/link-cplusplus/blob/75a186c35babbb7b39d0e5c544e1dfc9cc704800/README.md?plain=1#L54-L62
extern crate libz_sys;

#[test]
fn create_context() {
    use std::ptr;
    unsafe {
        let ctx = spng_ctx_new(0);
        assert_ne!(ptr::null_mut(), ctx);
        spng_ctx_free(ctx);
    }
}

//! Native bindings to [libspng](https://libspng.org).

#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
mod ffi;

pub use ffi::*;

#[test]
fn create_context() {
    use std::ptr;
    unsafe {
        let ctx = spng_ctx_new(0);
        assert_ne!(ptr::null_mut(), ctx);
        spng_ctx_free(ctx);
    }
}

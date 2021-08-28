VERSION=$(bindgen --version)
bindgen -o spng-sys/src/ffi.rs \
 --raw-line "/* ${VERSION} */" \
 --raw-line "#![allow(deref_nullptr)] /* https://github.com/rust-lang/rust-bindgen/pull/2055 */" \
 --whitelist-type "spng_.*" \
 --whitelist-var "SPNG_.*" \
 --whitelist-function "spng_.*" \
 --ctypes-prefix libc \
 --use-core \
 --impl-debug \
 --impl-partialeq \
 --size_t-is-usize \
 --opaque-type FILE \
 spng-sys/libspng/spng/spng.h

VERSION=$(bindgen --version)
bindgen -o spng-sys/src/ffi.rs \
 --raw-line "/* ${VERSION} */" \
 --allowlist-type "spng_.*" \
 --allowlist-var "SPNG_.*" \
 --allowlist-function "spng_.*" \
 --ctypes-prefix libc \
 --use-core \
 --impl-debug \
 --impl-partialeq \
 --opaque-type FILE \
 spng-sys/libspng/spng/spng.h

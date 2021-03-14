bindgen -o spng-sys/src/ffi.rs \
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

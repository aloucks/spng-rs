bindgen -o spng-sys/src/ffi.rs \
 --whitelist-type "spng_.*" \
 --whitelist-var "SPNG_.*" \
 --whitelist-function "spng_.*" \
 --ctypes-prefix libc \
 --use-core \
 --impl-debug \
 --impl-partialeq \
 spng-sys/libspng/spng.h

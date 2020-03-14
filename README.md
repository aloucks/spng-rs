# spng-rs

[![crates.io](https://img.shields.io/crates/v/spng.svg)](https://crates.io/crates/spng)
[![docs.rs](https://docs.rs/spng/badge.svg)](https://docs.rs/spng)
[![build status](https://dev.azure.com/aloucks/aloucks/_apis/build/status/aloucks.spng-rs?branchName=master)](https://dev.azure.com/aloucks/aloucks/_build/latest?definitionId=5&branchName=master)

Rust bindings to [libspng](https://libspng.org).

## Example

```rust
let cursor = std::io::Cursor::new(TEST_PNG);
let decoder = spng::Decoder::new(cursor);
let (out_info, mut reader) = decoder.read_info()?;
let output_buffer_size = reader.output_buffer_size();
assert_eq!(300, out_info.width);
assert_eq!(300, out_info.height);
assert_eq!(8, out_info.bit_depth);
assert_eq!(4, out_info.color_type.samples());
assert_eq!(out_info.buffer_size, output_buffer_size);
let mut out = vec![0; output_buffer_size];
reader.next_frame(&mut out)?;
```
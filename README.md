# spng-rs

[![crates.io](https://img.shields.io/crates/v/spng.svg)](https://crates.io/crates/spng)
[![docs.rs](https://docs.rs/spng/badge.svg)](https://docs.rs/spng)
[![build status](https://dev.azure.com/aloucks/aloucks/_apis/build/status/aloucks.spng-rs?branchName=master)](https://dev.azure.com/aloucks/aloucks/_build/latest?definitionId=5&branchName=master)

Rust bindings to [libspng].

## Version

| crate    | spng-rs  | libspng |
| -------- | -------- | ------- |
| spng     |  `0.1.0` | `master` ([rev]) |
| spng-sys |  `0.1.0` | `master` ([rev]) |

## Performance

This [test image] is decoded ~ 3-5x faster than with the [png] crate.

```
png_decode              time:   [2.1378 ms 2.1410 ms 2.1446 ms]
spng_decode             time:   [778.51 us 780.36 us 782.33 us]
spng_decode             time:   [420.45 us 421.26 us 422.12 us] (--features=zlib-ng)
```

## Examples

A one-liner for simple use cases:

```rust
let file = File::open("image.png")?;
let (out_info, data) = spng::decode(file, spng::Format::Rgba8)?;

assert_eq!(300, out_info.width);
assert_eq!(300, out_info.height);
assert_eq!(8, out_info.bit_depth);
assert_eq!(4, out_info.color_type.samples());
assert_eq!(out_info.buffer_size, output_buffer_size);
```

The `Decoder` interface is modeled after the [png] crate:

```rust
let file = File::open("image.png")?;
let decoder = spng::Decoder::new(file)
    .with_output_format(spng::Format::Rgba8);
let (out_info, mut reader) = decoder.read_info()?;
let out_buffer_size = reader.output_buffer_size();
let mut data = vec![0; out_buffer_size];
reader.next_frame(&mut data)?;

assert_eq!(300, out_info.width);
assert_eq!(300, out_info.height);
assert_eq!(8, out_info.bit_depth);
assert_eq!(4, out_info.color_type.samples());
assert_eq!(out_info.buffer_size, out_buffer_size);
```

The `RawContext` interface is a safe and minimal wrapper over the full [libspng] `C` API.

```rust
let file = File::open("image.png")?;
let out_format = spng::Format::Rgba8;
let mut ctx = spng::raw::RawContext::new()?;
ctx.set_png_stream(file)?;
let ihdr = ctx.get_ihdr()?;
let out_buffer_size = ctx.decoded_image_size(out_format)?;
let mut data = vec![0; out_buffer_size];
ctx.decode_image(&mut data, out_format, spng::DecodeFlags::empty())?;

assert_eq!(300, ihdr.width);
assert_eq!(300, ihdr.height);
assert_eq!(8, ihdr.bit_depth);
assert_eq!(4, spng::ColorType::try_from(ihdr.color_type)?.samples());
```

[png]: https://crates.io/crates/png
[libspng]: https://libspng.org
[rev]: https://github.com/randy408/libspng/tree/264476a1521bcb1d526c05ece0ed68b855fcfc4c
[test image]: spng/tests/test-002.png

# spng-rs

[![crates.io](https://img.shields.io/crates/v/spng.svg)](https://crates.io/crates/spng)
[![docs.rs](https://docs.rs/spng/badge.svg)](https://docs.rs/spng)
[![tests](https://github.com/aloucks/spng-rs/actions/workflows/tests.yml/badge.svg)](https://github.com/aloucks/spng-rs/actions/workflows/tests.yml)

Rust bindings to [libspng].

## Version

| spng-rs         | libspng                                                                                     |
|-----------------|---------------------------------------------------------------------------------------------|
| Unreleased      | [`0.7.4`](https://github.com/randy408/libspng/tree/v0.7.4)                                   |
| `0.2.0-alpha.2` | [`0.7.0-rc2`](https://github.com/randy408/libspng/tree/v0.7.0-rc2)                   |
| `0.2.0-alpha.1` | [`0.7.0-rc2`](https://github.com/randy408/libspng/tree/v0.7.0-rc2)                   |
| `0.1.0`         | [`0.6.3`](https://github.com/randy408/libspng/tree/264476a1521bcb1d526c05ece0ed68b855fcfc4c) |

## Performance

This [test image] is decoded ~ 3-5x faster than with the [png] crate.

```
png_decode              time:   [1.7354 ms 1.7372 ms 1.7392 ms]
spng_decode             time:   [569.27 µs 570.86 µs 572.45 µs]
spng_decode             time:   [311.84 µs 312.45 µs 313.13 µs] (--features=zlib-ng)
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
[test image]: spng/tests/test-002.png

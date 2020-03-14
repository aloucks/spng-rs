//! PNG image decoding
//!
//! Rust bindings to [libspng](https://libspng.org).
//!
//! # Examples
//!
//! ```
//! # static TEST_PNG: &[u8] = include_bytes!("../test.png");
//! let cursor = std::io::Cursor::new(TEST_PNG);
//! let decoder = spng::Decoder::new(cursor);
//! let (out_info, mut reader) = decoder.read_info()?;
//! let output_buffer_size = reader.output_buffer_size();
//! assert_eq!(300, out_info.width);
//! assert_eq!(300, out_info.height);
//! assert_eq!(8, out_info.bit_depth);
//! assert_eq!(4, out_info.color_type.samples());
//! assert_eq!(out_info.buffer_size, output_buffer_size);
//! let mut out = vec![0; output_buffer_size];
//! reader.next_frame(&mut out)?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use std::convert::TryFrom;
use std::{io, mem, ptr, slice};

use spng_sys as sys;

mod error;

use error::check_err;
pub use error::Error;

#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Format {
    Rgba8 = sys::spng_format_SPNG_FMT_RGBA8,
    Rgba16 = sys::spng_format_SPNG_FMT_RGBA16,
}

#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ColorType {
    Grayscale = sys::spng_color_type_SPNG_COLOR_TYPE_GRAYSCALE,
    /// RGB
    Truecolor = sys::spng_color_type_SPNG_COLOR_TYPE_TRUECOLOR,
    Indexed = sys::spng_color_type_SPNG_COLOR_TYPE_INDEXED,
    GrayscaleAlpha = sys::spng_color_type_SPNG_COLOR_TYPE_GRAYSCALE_ALPHA,
    /// RGBA
    TruecolorAlpha = sys::spng_color_type_SPNG_COLOR_TYPE_TRUECOLOR_ALPHA,
}

impl TryFrom<u8> for ColorType {
    type Error = Error;
    fn try_from(c: u8) -> Result<ColorType, Error> {
        use ColorType::*;
        let c = c as i32;
        match c {
            sys::spng_color_type_SPNG_COLOR_TYPE_GRAYSCALE => Ok(Grayscale),
            sys::spng_color_type_SPNG_COLOR_TYPE_TRUECOLOR => Ok(Truecolor),
            sys::spng_color_type_SPNG_COLOR_TYPE_INDEXED => Ok(Indexed),
            sys::spng_color_type_SPNG_COLOR_TYPE_GRAYSCALE_ALPHA => Ok(GrayscaleAlpha),
            sys::spng_color_type_SPNG_COLOR_TYPE_TRUECOLOR_ALPHA => Ok(TruecolorAlpha),
            _ => Err(Error::ColorType),
        }
    }
}

impl ColorType {
    /// Returns the number of samples per pixel
    pub fn samples(&self) -> usize {
        use ColorType::*;
        match self {
            Grayscale | Indexed => 1,
            GrayscaleAlpha => 2,
            Truecolor => 3,
            TruecolorAlpha => 4,
        }
    }
}

bitflags::bitflags! {
    /// Decoding flags
    pub struct DecodeFlags: i32 {
        /// Apply transparency
        const TRANSPARENCY = sys::spng_decode_flags_SPNG_DECODE_USE_TRNS;
        /// Apply gamma correction
        const GAMMA = sys::spng_decode_flags_SPNG_DECODE_USE_GAMA;
        #[doc(hidden)]
        const SIGNIFICANT_BIT = sys::spng_decode_flags_SPNG_DECODE_USE_SBIT;
    }
}

/// Decoding limits
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Limits {
    /// Maximum image width
    pub max_width: u32,
    /// Maximum image height
    pub max_height: u32,
}

impl Default for Limits {
    fn default() -> Limits {
        Limits {
            max_width: std::u32::MAX / 2 - 1,
            max_height: std::u32::MAX / 2 - 1,
        }
    }
}

/// PNG decoder
#[derive(Debug)]
pub struct Decoder<R> {
    reader: R,
    limits: Limits,
    decode_flags: DecodeFlags,
    output_format: Option<Format>,
}

/// Decoded output image information
#[derive(Debug)]
pub struct OutputInfo {
    /// The image width in pixels
    pub width: u32,
    /// The image height in pixels
    pub height: u32,
    /// The color channels
    pub color_type: ColorType,
    /// The per component bit depth
    pub bit_depth: u8,
    /// The minimum buffer size required for the decoded pixel output
    pub buffer_size: usize,
}

/// PNG image information
#[derive(Debug)]
pub struct Info {
    /// The image width in pixels
    pub width: u32,
    /// The image height in pixels
    pub height: u32,
    /// The color channels
    pub color_type: ColorType,
    /// The per component bit depth
    pub bit_depth: u8,
}

#[derive(Debug)]
/// PNG reader
pub struct Reader<R> {
    ctx: Context,
    out_format: Format,
    info: Info,
    #[allow(unused)]
    inner: Box<R>,
    decode_flags: DecodeFlags,
    output_buffer_size: usize,
}

unsafe extern "C" fn read_fn<R: io::Read>(
    _: *mut sys::spng_ctx,
    user: *mut libc::c_void,
    dest: *mut libc::c_void,
    len: usize,
) -> libc::c_int {
    let reader: &mut R = &mut *(user as *mut R as *mut _);
    let dest = slice::from_raw_parts_mut(dest as *mut u8, len);
    match reader.read(dest) {
        Ok(0) => sys::spng_errno_SPNG_IO_EOF,
        Ok(_) => sys::spng_errno_SPNG_OK,
        Err(_) => sys::spng_errno_SPNG_IO_ERROR,
    }
}

#[derive(Debug)]
struct Context {
    raw: *mut sys::spng_ctx,
}

impl Drop for Context {
    fn drop(&mut self) {
        if self.raw != ptr::null_mut() {
            unsafe {
                sys::spng_ctx_free(self.raw);
            }
        }
    }
}

impl Context {
    fn new(flags: i32) -> Result<Context, Error> {
        unsafe {
            let raw = sys::spng_ctx_new(flags);
            if raw == ptr::null_mut() {
                return Err(Error::Mem);
            } else {
                Ok(Context { raw })
            }
        }
    }

    fn decoded_image_size(&self, out_format: Format) -> Result<usize, Error> {
        let mut len = 0;
        unsafe {
            check_err(sys::spng_decoded_image_size(
                self.raw,
                out_format as _,
                &mut len,
            ))?;
        }
        Ok(len)
    }

    fn set_image_limits(&mut self, max_width: u32, max_height: u32) -> Result<(), Error> {
        unsafe { check_err(sys::spng_set_image_limits(self.raw, max_width, max_height)) }
    }

    fn set_png_stream<R>(
        &mut self,
        read_fn: sys::spng_read_fn,
        reader: *mut R,
    ) -> Result<(), Error> {
        unsafe {
            check_err(sys::spng_set_png_stream(
                self.raw,
                read_fn,
                reader as *mut _,
            ))
        }
    }

    fn get_ihdr(&self) -> Result<sys::spng_ihdr, Error> {
        unsafe {
            let mut header = mem::zeroed();
            check_err(sys::spng_get_ihdr(self.raw, &mut header))?;
            Ok(header)
        }
    }

    fn decode_image(
        &mut self,
        output: &mut [u8],
        out_format: Format,
        flags: DecodeFlags,
    ) -> Result<(), Error> {
        unsafe {
            check_err(sys::spng_decode_image(
                self.raw,
                output.as_mut_ptr() as _,
                output.len(),
                out_format as _,
                flags.bits,
            ))
        }
    }
}

impl<R: io::Read> Decoder<R> {
    /// Create a new `png` decoder with the default limits
    pub fn new(r: R) -> Decoder<R> {
        Decoder::with_limits(r, Limits::default())
    }

    /// Create a new `png` decoder with the given limits
    pub fn with_limits(r: R, limits: Limits) -> Decoder<R> {
        let decode_flags = DecodeFlags::empty();
        Decoder {
            reader: r,
            limits,
            decode_flags,
            output_format: None,
        }
    }

    /// Set the limits
    pub fn set_limits(&mut self, limits: Limits) {
        self.limits = limits;
    }

    /// Set the decoding flags
    pub fn set_decode_flags(&mut self, decode_flags: DecodeFlags) {
        self.decode_flags = decode_flags;
    }

    /// Set the output image format
    pub fn set_output_format(&mut self, format: Format) {
        self.output_format = Some(format);
    }

    /// Read the `png` header and initialize decoding.
    pub fn read_info(self) -> Result<(OutputInfo, Reader<R>), Error> {
        let mut inner = Box::new(self.reader);
        let mut ctx = Context::new(self.decode_flags.bits)?;
        ctx.set_image_limits(self.limits.max_width, self.limits.max_height)?;
        ctx.set_png_stream(Some(read_fn::<R>), inner.as_mut() as *mut R as *mut _)?;
        let header = ctx.get_ihdr()?;

        let info = Info {
            bit_depth: header.bit_depth,
            color_type: ColorType::try_from(header.color_type)?,
            width: header.width,
            height: header.height,
        };
        let out_format =
            self.output_format
                .unwrap_or_else(|| match (info.bit_depth, info.color_type) {
                    (16, _) => Format::Rgba16,
                    (_, _) => Format::Rgba8,
                });
        let buffer_size = ctx.decoded_image_size(out_format)?;

        let (out_bit_depth, out_color_type) = match out_format {
            Format::Rgba8 => (8, ColorType::TruecolorAlpha),
            Format::Rgba16 => (16, ColorType::TruecolorAlpha),
        };
        let out_info = OutputInfo {
            width: info.width,
            height: info.height,
            bit_depth: out_bit_depth,
            color_type: out_color_type,
            buffer_size,
        };
        let reader = Reader {
            ctx,
            out_format,
            info,
            decode_flags: self.decode_flags,
            inner,
            output_buffer_size: buffer_size,
        };

        Ok((out_info, reader))
    }
}

impl<R> Reader<R> {
    /// Returns input information
    pub fn info(&self) -> &Info {
        &self.info
    }

    /// Returns the minimum buffer size required for `next_frame`
    pub fn output_buffer_size(&self) -> usize {
        self.output_buffer_size
    }

    /// Decodes the next frame of the `png`. This currently may only be called once.
    pub fn next_frame(&mut self, output: &mut [u8]) -> Result<(), Error> {
        self.ctx
            .decode_image(output, self.out_format, self.decode_flags)
    }
}

#[cfg(test)]
static TEST_PNG: &[u8] = include_bytes!("../test.png");

#[test]
fn decode() -> Result<(), Box<dyn std::error::Error>> {
    use std::io::Cursor;
    let cursor = Cursor::new(TEST_PNG);
    let decoder = Decoder::new(cursor);
    let (out_info, mut reader) = decoder.read_info()?;
    let output_buffer_size = reader.output_buffer_size();
    assert_eq!(300, out_info.width);
    assert_eq!(300, out_info.height);
    assert_eq!(8, out_info.bit_depth);
    assert_eq!(4, out_info.color_type.samples());
    assert_eq!(out_info.buffer_size, output_buffer_size);
    let mut out = vec![0; output_buffer_size];
    reader.next_frame(&mut out)?;
    Ok(())
}

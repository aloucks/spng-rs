//! PNG image decoding
//!
//! Rust bindings to [libspng](https://libspng.org).
//!
//! # Examples
//!
//! ```
//! # static TEST_PNG: &[u8] = include_bytes!("../tests/test-001.png");
//! let cursor = std::io::Cursor::new(TEST_PNG);
//! let decoder = spng::Decoder::new(cursor);
//! let mut reader = decoder.read_info()?;
//! let info = reader.info();
//! let output_buffer_size = reader.output_buffer_size();
//! assert_eq!(300, info.width);
//! assert_eq!(300, info.height);
//! assert_eq!(8, info.bit_depth as u8);
//! assert_eq!(4, info.color_type.samples());
//! let mut out = vec![0; output_buffer_size];
//! let out_info = reader.next_frame(&mut out)?;
//! assert_eq!(output_buffer_size, out_info.buffer_size());
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use std::convert::TryFrom;
use std::io;

use spng_sys as sys;

mod error;
pub mod raw;

pub use error::Error;

use raw::RawContext;

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum CrcAction {
    /// Default
    Error = sys::spng_crc_action_SPNG_CRC_ERROR,
    /// Discard chunk, invalid for critical chunks
    Discard = sys::spng_crc_action_SPNG_CRC_DISCARD,
    /// Ignore and don't calculate checksum
    Use = sys::spng_crc_action_SPNG_CRC_USE,
}

/// PNG output format
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Format {
    Rgba8 = sys::spng_format_SPNG_FMT_RGBA8,
    Rgba16 = sys::spng_format_SPNG_FMT_RGBA16,
    Rgb8 = sys::spng_format_SPNG_FMT_RGB8,
    G8 = sys::spng_format_SPNG_FMT_G8,
    Ga8 = sys::spng_format_SPNG_FMT_GA8,
    Ga16 = sys::spng_format_SPNG_FMT_GA16,
    /// The PNG's format in host-endian
    Png = sys::spng_format_SPNG_FMT_PNG,
    /// The PNG's format in big-endian
    Raw = sys::spng_format_SPNG_FMT_RAW,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ColorType {
    Grayscale = sys::spng_color_type_SPNG_COLOR_TYPE_GRAYSCALE as u8,
    /// RGB
    Truecolor = sys::spng_color_type_SPNG_COLOR_TYPE_TRUECOLOR as u8,
    Indexed = sys::spng_color_type_SPNG_COLOR_TYPE_INDEXED as u8,
    GrayscaleAlpha = sys::spng_color_type_SPNG_COLOR_TYPE_GRAYSCALE_ALPHA as u8,
    /// RGBA
    TruecolorAlpha = sys::spng_color_type_SPNG_COLOR_TYPE_TRUECOLOR_ALPHA as u8,
}

impl ColorType {
    /// Alias for `Truecolor`
    pub const RGB: ColorType = ColorType::Truecolor;
    /// Alias for `TruecolorAlpha`
    pub const RGBA: ColorType = ColorType::TruecolorAlpha;
    /// Alias for `Grayscale`
    pub const G: ColorType = ColorType::Grayscale;
    /// Alias for `GrayscaleAlpha`
    pub const GA: ColorType = ColorType::GrayscaleAlpha;
}

impl TryFrom<u8> for ColorType {
    type Error = Error;
    fn try_from(value: u8) -> Result<ColorType, Error> {
        use ColorType::*;
        match value as u32 {
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
    pub fn samples(self) -> usize {
        use ColorType::*;
        match self {
            Grayscale | Indexed => 1,
            GrayscaleAlpha => 2,
            Truecolor => 3,
            TruecolorAlpha => 4,
        }
    }
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum BitDepth {
    One = 1,
    Two = 2,
    Four = 4,
    Eight = 8,
    Sixteen = 16,
}

impl TryFrom<u8> for BitDepth {
    type Error = Error;
    fn try_from(value: u8) -> Result<BitDepth, Error> {
        use BitDepth::*;
        match value as i32 {
            1 => Ok(One),
            2 => Ok(Two),
            4 => Ok(Four),
            8 => Ok(Eight),
            16 => Ok(Sixteen),
            _ => Err(Error::BitDepth),
        }
    }
}

bitflags::bitflags! {
    /// Decoding flags
    pub struct DecodeFlags: u32 {
        /// Apply transparency
        const TRANSPARENCY = sys::spng_decode_flags_SPNG_DECODE_TRNS;
        /// Apply gamma correction
        const GAMMA = sys::spng_decode_flags_SPNG_DECODE_GAMMA;
        /// Initialize for progressive reads
        const PROGRESSIVE = sys::spng_decode_flags_SPNG_DECODE_PROGRESSIVE;
        #[doc(hidden)]
        const SIGNIFICANT_BIT = sys::spng_decode_flags_SPNG_DECODE_USE_SBIT;
    }
}

bitflags::bitflags! {
    pub struct ContextFlags: u32 {
        /// Ignore checksum in `DEFLATE` streams
        const IGNORE_ADLER32 = sys::spng_ctx_flags_SPNG_CTX_IGNORE_ADLER32;
    }
}

/// Decoding limits
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Limits {
    /// Maximum image width
    pub max_width: u32,
    /// Maximum image height
    pub max_height: u32,
}

const PNG_U32_MAX: u32 = std::u32::MAX / 2 - 1;

impl Default for Limits {
    fn default() -> Limits {
        Limits {
            max_width: PNG_U32_MAX,
            max_height: PNG_U32_MAX,
        }
    }
}

/// PNG decoder
#[derive(Debug)]
pub struct Decoder<R> {
    reader: R,
    limits: Limits,
    context_flags: ContextFlags,
    decode_flags: DecodeFlags,
    output_format: Format,
}

/// Decoded output image information
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct OutputInfo {
    /// The image width in pixels
    pub width: u32,
    /// The image height in pixels
    pub height: u32,
    /// The color channels
    pub color_type: ColorType,
    /// The per-component bit depth
    pub bit_depth: BitDepth,
    /// The minimum buffer size required for the decoded pixel output
    pub buffer_size: usize,
}

impl OutputInfo {
    /// The width of each row or scanline
    pub fn line_size(&self) -> usize {
        self.buffer_size / self.height as usize
    }

    pub fn buffer_size(&self) -> usize {
        self.buffer_size
    }
}

impl OutputInfo {
    fn from_ihdr_format_buffer_size(
        ihdr: &sys::spng_ihdr,
        output_format: Format,
        output_buffer_size: usize,
    ) -> Result<OutputInfo, Error> {
        let bit_depth = match output_format {
            Format::Png | Format::Raw => BitDepth::try_from(ihdr.bit_depth)?,
            Format::Rgb8 | Format::Rgba8 | Format::G8 | Format::Ga8 => BitDepth::Eight,
            Format::Rgba16 | Format::Ga16 => BitDepth::Sixteen,
        };
        let color_type = match output_format {
            Format::Png | Format::Raw => ColorType::try_from(ihdr.color_type)?,
            Format::Rgb8 => ColorType::Truecolor,
            Format::Rgba8 => ColorType::TruecolorAlpha,
            Format::Rgba16 => ColorType::TruecolorAlpha,
            Format::G8 => ColorType::Grayscale,
            Format::Ga8 | Format::Ga16 => ColorType::GrayscaleAlpha,
        };
        Ok(OutputInfo {
            bit_depth,
            color_type,
            width: ihdr.width,
            height: ihdr.height,
            buffer_size: output_buffer_size,
        })
    }
}

/// PNG image information
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Info {
    /// The image width in pixels
    pub width: u32,
    /// The image height in pixels
    pub height: u32,
    /// The color channels
    pub color_type: ColorType,
    /// The per-component bit depth
    pub bit_depth: BitDepth,
}

impl Info {
    fn from_ihdr(header: &sys::spng_ihdr) -> Result<Info, Error> {
        Ok(Info {
            width: header.width,
            height: header.height,
            bit_depth: BitDepth::try_from(header.bit_depth)?,
            color_type: ColorType::try_from(header.color_type)?,
        })
    }
}

#[derive(Debug)]
/// PNG reader
pub struct Reader<R> {
    ctx: RawContext<R>,
    ihdr: sys::spng_ihdr,
    output_buffer_size: usize,
    output_format: Format,
    decode_flags: DecodeFlags,
}

impl<R> Decoder<R> {
    /// Create a new `png` decoder with the default limits
    pub fn new(reader: R) -> Decoder<R> {
        let decode_flags = DecodeFlags::empty();
        let context_flags = ContextFlags::empty();
        let output_format = Format::Png;
        let limits = Limits::default();
        Decoder {
            reader,
            limits,
            context_flags,
            decode_flags,
            output_format,
        }
    }

    pub fn with_limits(mut self, limits: Limits) -> Decoder<R> {
        self.limits = limits;
        self
    }

    pub fn with_context_flags(mut self, context_flags: ContextFlags) -> Decoder<R> {
        self.context_flags = context_flags;
        self
    }

    pub fn with_decode_flags(mut self, decode_flags: DecodeFlags) -> Decoder<R> {
        self.decode_flags = decode_flags;
        self
    }

    pub fn with_output_format(mut self, output_format: Format) -> Decoder<R> {
        self.output_format = output_format;
        self
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
    pub fn set_output_format(&mut self, output_format: Format) {
        self.output_format = output_format;
    }

    pub fn set_context_flags(&mut self, context_flags: ContextFlags) {
        self.context_flags = context_flags;
    }

    /// Read the `png` header and initialize decoding.
    pub fn read_info(self) -> Result<Reader<R>, Error>
    where
        R: io::Read,
    {
        let mut ctx = RawContext::with_flags(self.context_flags)?;
        ctx.set_image_limits(self.limits.max_width, self.limits.max_height)?;
        ctx.set_png_stream(self.reader)?;
        let ihdr = ctx.get_ihdr()?;
        let output_buffer_size = ctx.decoded_image_size(self.output_format)?;
        let reader = Reader {
            ctx,
            ihdr,
            output_format: self.output_format,
            decode_flags: self.decode_flags,
            output_buffer_size,
        };

        Ok(reader)
    }
}

impl<R> Reader<R> {
    /// Returns input information
    pub fn info(&self) -> Info {
        Info::from_ihdr(&self.ihdr).expect("invalid ihdr")
    }

    /// Returns the minimum buffer size required for `next_frame`
    #[inline]
    pub fn output_buffer_size(&self) -> usize {
        self.output_buffer_size
    }

    /// Decodes the next frame of the `png`. This currently may only be called once.
    pub fn next_frame(&mut self, output: &mut [u8]) -> Result<OutputInfo, Error> {
        self.ctx
            .decode_image(output, self.output_format, self.decode_flags)?;
        let ihdr = self.ctx.get_ihdr()?;
        let output_info = OutputInfo::from_ihdr_format_buffer_size(
            &ihdr,
            self.output_format,
            self.output_buffer_size(),
        )?;
        Ok(output_info)
    }

    /// Returns a reference to the `RawContext`.
    pub fn raw_context(&self) -> &RawContext<R> {
        &self.ctx
    }
}

/// Decode `png` data.
pub fn decode<R>(reader: R, output_format: Format) -> Result<(OutputInfo, Vec<u8>), Error>
where
    R: io::Read,
{
    let decoder = Decoder::new(reader).with_output_format(output_format);
    let mut reader = decoder.read_info()?;
    let mut out = Vec::new();
    out.reserve_exact(reader.output_buffer_size());
    unsafe {
        out.set_len(reader.output_buffer_size());
    }
    let out_info = reader.next_frame(&mut out)?;
    Ok((out_info, out))
}

/// Returns the `libspng` version: `(major, minor, patch)`
pub fn version() -> (u32, u32, u32) {
    (
        spng_sys::SPNG_VERSION_MAJOR,
        spng_sys::SPNG_VERSION_MINOR,
        spng_sys::SPNG_VERSION_PATCH,
    )
}

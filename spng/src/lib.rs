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
//! let (out_info, mut reader) = decoder.read_info()?;
//! let output_buffer_size = reader.output_buffer_size();
//! assert_eq!(300, out_info.width);
//! assert_eq!(300, out_info.height);
//! assert_eq!(8, out_info.bit_depth as u8);
//! assert_eq!(4, out_info.color_type.samples());
//! assert_eq!(out_info.buffer_size, output_buffer_size);
//! let mut out = vec![0; output_buffer_size];
//! reader.next_frame(&mut out)?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use std::convert::TryFrom;
use std::{io, mem, mem::MaybeUninit, slice};

use spng_sys as sys;

mod error;

use error::check_err;
pub use error::Error;

#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Format {
    Rgba8 = sys::spng_format_SPNG_FMT_RGBA8,
    Rgba16 = sys::spng_format_SPNG_FMT_RGBA16,
    Rgb8 = sys::spng_format_SPNG_FMT_RGB8,
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

impl TryFrom<u8> for ColorType {
    type Error = Error;
    fn try_from(value: u8) -> Result<ColorType, Error> {
        use ColorType::*;
        match value as i32 {
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
    pub struct DecodeFlags: i32 {
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
    pub struct ContextFlags: i32 {
        /// Ignore checksum in `DEFLATE` streams
        const IGNORE_ADLER32 = sys::spng_ctx_flags_SPNG_CTX_IGNORE_ADLER32;
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
#[derive(Debug)]
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
    /// The per-component bit depth
    pub bit_depth: BitDepth,
}

impl Info {
    fn from_header(header: &sys::spng_ihdr) -> Result<Info, Error> {
        Ok(Info {
            width: header.width,
            height: header.height,
            bit_depth: BitDepth::try_from(header.bit_depth)?,
            color_type: ColorType::try_from(header.color_type)?,
        })
    }

    fn output_info(
        &self,
        output_format: Format,
        output_buffer_size: usize,
    ) -> Result<OutputInfo, Error> {
        let bit_depth = match output_format {
            Format::Png | Format::Raw => self.bit_depth,
            Format::Rgb8 | Format::Rgba8 => BitDepth::Eight,
            Format::Rgba16 => BitDepth::Sixteen,
        };
        let color_type = match output_format {
            Format::Png | Format::Raw => self.color_type,
            Format::Rgb8 => ColorType::Truecolor,
            Format::Rgba8 => ColorType::TruecolorAlpha,
            Format::Rgba16 => ColorType::TruecolorAlpha,
        };
        Ok(OutputInfo {
            bit_depth,
            color_type,
            width: self.width,
            height: self.height,
            buffer_size: output_buffer_size,
        })
    }
}

#[derive(Debug)]
/// PNG reader
pub struct Reader<R> {
    ctx: RawContext<R>,
    out_format: Format,
    info: Info,
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
    let mut offset = 0;
    while offset < len {
        let buf = &mut dest[offset..];
        let ret = reader.read(buf);
        match ret {
            Ok(0) => return sys::spng_errno_SPNG_IO_EOF,
            Ok(n) => offset += n,
            Err(_) => return sys::spng_errno_SPNG_IO_ERROR,
        }
    }
    sys::spng_errno_SPNG_OK
}

/// The raw decoding context.
///
/// <http://www.libpng.org/pub/png/spec/1.1/PNG-Chunks.html>
#[derive(Debug)]
pub struct RawContext<R> {
    raw: *mut sys::spng_ctx,
    reader: Option<Box<R>>,
}

impl<R> Drop for RawContext<R> {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe {
                sys::spng_ctx_free(self.raw);
            }
        }
    }
}

impl<R> RawContext<R> {
    pub fn new(flags: ContextFlags) -> Result<RawContext<R>, Error> {
        unsafe {
            let raw = sys::spng_ctx_new(flags.bits());
            if raw.is_null() {
                Err(Error::Mem)
            } else {
                Ok(RawContext { raw, reader: None })
            }
        }
    }

    pub fn decoded_image_size(&self, out_format: Format) -> Result<usize, Error> {
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

    pub fn set_image_limits(&mut self, max_width: u32, max_height: u32) -> Result<(), Error> {
        unsafe { check_err(sys::spng_set_image_limits(self.raw, max_width, max_height)) }
    }

    /// Returns the image limits: `(width, height)`
    pub fn get_image_limits(&self) -> Result<(u32, u32), Error> {
        let mut width = 0;
        let mut height = 0;
        unsafe {
            check_err(sys::spng_get_image_limits(
                self.raw,
                &mut width,
                &mut height,
            ))?;
            Ok((width, height))
        }
    }

    pub fn get_ihdr(&self) -> Result<sys::spng_ihdr, Error> {
        unsafe {
            let mut ihdr = MaybeUninit::uninit();
            check_err(sys::spng_get_ihdr(self.raw, ihdr.as_mut_ptr()))?;
            Ok(ihdr.assume_init())
        }
    }

    pub fn get_plte(&self) -> Result<sys::spng_plte, Error> {
        unsafe {
            let mut plte = MaybeUninit::uninit();
            check_err(sys::spng_get_plte(self.raw, plte.as_mut_ptr()))?;
            Ok(plte.assume_init())
        }
    }

    pub fn get_trns(&self) -> Result<sys::spng_trns, Error> {
        unsafe {
            let mut trns = MaybeUninit::uninit();
            check_err(sys::spng_get_trns(self.raw, trns.as_mut_ptr()))?;
            Ok(trns.assume_init())
        }
    }

    pub fn get_chrm(&self) -> Result<sys::spng_chrm, Error> {
        unsafe {
            let mut chrm = MaybeUninit::uninit();
            check_err(sys::spng_get_chrm(self.raw, chrm.as_mut_ptr()))?;
            Ok(chrm.assume_init())
        }
    }

    pub fn get_chrm_int(&self) -> Result<sys::spng_chrm_int, Error> {
        unsafe {
            let mut spng_chrm_int = MaybeUninit::uninit();
            check_err(sys::spng_get_chrm_int(self.raw, spng_chrm_int.as_mut_ptr()))?;
            Ok(spng_chrm_int.assume_init())
        }
    }

    pub fn get_gama(&self) -> Result<f64, Error> {
        unsafe {
            let mut gama = MaybeUninit::uninit();
            check_err(sys::spng_get_gama(self.raw, gama.as_mut_ptr()))?;
            Ok(gama.assume_init())
        }
    }

    pub fn get_iccp(&self) -> Result<sys::spng_iccp, Error> {
        unsafe {
            let mut iccp = MaybeUninit::uninit();
            check_err(sys::spng_get_iccp(self.raw, iccp.as_mut_ptr()))?;
            Ok(iccp.assume_init())
        }
    }

    pub fn get_sbit(&self) -> Result<sys::spng_sbit, Error> {
        unsafe {
            let mut sbit = MaybeUninit::uninit();
            check_err(sys::spng_get_sbit(self.raw, sbit.as_mut_ptr()))?;
            Ok(sbit.assume_init())
        }
    }

    /// Returns the `sRGB` rendering intent or `Err(Chunkavil)` for non-`sRGB` images.
    pub fn get_srgb(&self) -> Result<u8, Error> {
        unsafe {
            let mut rendering_intent = 0;
            check_err(sys::spng_get_srgb(self.raw, &mut rendering_intent))?;
            Ok(rendering_intent)
        }
    }

    /// Returns text information
    ///
    /// Note that the referenced text data pointers are freed when the context is dropped.
    pub fn get_text(&self) -> Result<Vec<sys::spng_text>, Error> {
        unsafe {
            use std::ptr;
            let mut len = 0;
            check_err(sys::spng_get_text(self.raw, ptr::null_mut(), &mut len))?;
            let mut text =
                vec![MaybeUninit::<sys::spng_text>::uninit().assume_init(); len as usize];
            check_err(sys::spng_get_text(self.raw, text.as_mut_ptr(), &mut len))?;
            Ok(mem::transmute(text))
        }
    }

    pub fn get_bkgd(&self) -> Result<sys::spng_bkgd, Error> {
        unsafe {
            let mut bkgd = MaybeUninit::uninit();
            check_err(sys::spng_get_bkgd(self.raw, bkgd.as_mut_ptr()))?;
            Ok(bkgd.assume_init())
        }
    }

    /// Return the physical pixel dimensions
    pub fn get_phys(&self) -> Result<sys::spng_phys, Error> {
        unsafe {
            let mut phys = MaybeUninit::uninit();
            check_err(sys::spng_get_phys(self.raw, phys.as_mut_ptr()))?;
            Ok(phys.assume_init())
        }
    }

    /// Returns suggested palettes
    ///
    /// Note that the referenced suggested palette pointers are freed when the context is dropped.
    pub fn get_splt(&self) -> Result<Vec<sys::spng_splt>, Error> {
        unsafe {
            use std::ptr;
            let mut len = 0;
            check_err(sys::spng_get_splt(self.raw, ptr::null_mut(), &mut len))?;
            let mut splt =
                vec![MaybeUninit::<sys::spng_splt>::uninit().assume_init(); len as usize];
            check_err(sys::spng_get_splt(self.raw, splt.as_mut_ptr(), &mut len))?;
            Ok(mem::transmute(splt))
        }
    }

    pub fn get_time(&self) -> Result<sys::spng_time, Error> {
        unsafe {
            let mut time = MaybeUninit::uninit();
            check_err(sys::spng_get_time(self.raw, time.as_mut_ptr()))?;
            Ok(time.assume_init())
        }
    }

    /// Return the image offset
    pub fn get_offs(&self) -> Result<sys::spng_offs, Error> {
        unsafe {
            let mut offs = MaybeUninit::uninit();
            check_err(sys::spng_get_offs(self.raw, offs.as_mut_ptr()))?;
            Ok(offs.assume_init())
        }
    }

    /// Return `EXIF` data.
    ///
    /// Note that `exif.data` is freed when the context is destroyed.
    pub fn get_exif(&self) -> Result<sys::spng_exif, Error> {
        unsafe {
            let mut exif = MaybeUninit::uninit();
            check_err(sys::spng_get_exif(self.raw, exif.as_mut_ptr()))?;
            Ok(exif.assume_init())
        }
    }

    pub fn get_row_info(&self) -> Result<sys::spng_row_info, Error> {
        unsafe {
            let mut row_info = MaybeUninit::uninit();
            check_err(sys::spng_get_row_info(self.raw, row_info.as_mut_ptr()))?;
            Ok(row_info.assume_init())
        }
    }

    pub fn decode_image(
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

    pub fn decode_row(&mut self, output: &mut [u8]) -> Result<(), Error> {
        unsafe {
            check_err(sys::spng_decode_row(
                self.raw,
                output.as_mut_ptr() as _,
                output.len(),
            ))
        }
    }

    pub fn decode_scanline(&mut self, output: &mut [u8]) -> Result<(), Error> {
        unsafe {
            check_err(sys::spng_decode_scanline(
                self.raw,
                output.as_mut_ptr() as _,
                output.len(),
            ))
        }
    }
}

impl<R: io::Read> RawContext<R> {
    /// Set the input `png` stream reader. The input buffer or stream may only be set once per context.
    pub fn set_png_stream(&mut self, reader: R) -> Result<(), Error> {
        let mut boxed = Box::new(reader);
        let user = boxed.as_mut() as *mut R as *mut _;
        self.reader = Some(boxed);
        let read_fn: sys::spng_read_fn = Some(read_fn::<R>);
        unsafe { check_err(sys::spng_set_png_stream(self.raw, read_fn, user)) }
    }
}

impl<'a> RawContext<&'a [u8]> {
    /// Set the input `png` buffer. The input buffer or stream may only be set once per context.
    pub fn set_png_buffer(&mut self, buf: &'a [u8]) -> Result<(), Error> {
        unsafe {
            check_err(sys::spng_set_png_buffer(
                self.raw,
                buf.as_ptr() as *const _,
                buf.len(),
            ))
        }
    }
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
            decode_flags,
            context_flags,
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
    pub fn read_info(self) -> Result<(OutputInfo, Reader<R>), Error>
    where
        R: io::Read,
    {
        let mut ctx = RawContext::new(self.context_flags)?;
        ctx.set_image_limits(self.limits.max_width, self.limits.max_height)?;
        ctx.set_png_stream(self.reader)?;
        let header = ctx.get_ihdr()?;
        let output_buffer_size = ctx.decoded_image_size(self.output_format)?;
        let info = Info::from_header(&header)?;
        let out_info = info.output_info(self.output_format, output_buffer_size)?;
        let reader = Reader {
            ctx,
            out_format: self.output_format,
            info,
            decode_flags: self.decode_flags,
            output_buffer_size,
        };

        Ok((out_info, reader))
    }
}

impl<'a> Decoder<&'a [u8]> {
    /// Read the `png` header and initialize decoding.
    ///
    /// Like [`read_info`] but prevents extra copies when the `png` data is already in memory.
    ///
    /// [`read_info`]: method@Decoder::read_info
    pub fn read_info_from_slice(self) -> Result<(OutputInfo, Reader<&'a [u8]>), Error> {
        let mut ctx = RawContext::new(self.context_flags)?;
        ctx.set_image_limits(self.limits.max_width, self.limits.max_height)?;
        ctx.set_png_buffer(self.reader)?;
        let header = ctx.get_ihdr()?;
        let output_buffer_size = ctx.decoded_image_size(self.output_format)?;
        let info = Info::from_header(&header)?;
        let out_info = info.output_info(self.output_format, output_buffer_size)?;
        let reader = Reader {
            ctx,
            out_format: self.output_format,
            info,
            decode_flags: self.decode_flags,
            output_buffer_size,
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

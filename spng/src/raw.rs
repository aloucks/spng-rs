use crate::{
    error::{check_err, Error},
    ContextFlags, CrcAction, DecodeFlags, Format,
};

use spng_sys as sys;
use std::{io, mem, mem::MaybeUninit, slice};

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

/// Helper trait for converting optional ancillary chunks into `Option<T>`.
///
/// <http://www.libpng.org/pub/png/spec/1.1/PNG-Chunks.html#C.Ancillary-chunks>
///
/// ## Ancilary chunks
///
/// * BKGD
/// * CHRM
/// * GAMA
/// * HIST
/// * ICCP
/// * PHYS
/// * SBIT
/// * SPLT
/// * SRGB
/// * TEXT
/// * TIME
/// * TRNS
/// * ZTXT
pub trait IfPresent<T> {
    /// Converts `Err(Error::Chunkavail)` into `Ok(None)`.
    fn if_present(self) -> Result<Option<T>, Error>;
}

impl<T> IfPresent<T> for Result<T, Error> {
    fn if_present(self) -> Result<Option<T>, Error> {
        match self {
            Ok(value) => Ok(Some(value)),
            Err(Error::Chunkavail) => Ok(None),
            Err(error) => Err(error),
        }
    }
}

/// The raw decoding context.
///
/// * <https://libspng.org/>
/// * <http://www.libpng.org/pub/png/spec/1.1/PNG-Contents.html>
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
    pub fn new() -> Result<RawContext<R>, Error> {
        RawContext::with_flags(ContextFlags::empty())
    }

    pub fn with_flags(flags: ContextFlags) -> Result<RawContext<R>, Error> {
        unsafe {
            let raw = sys::spng_ctx_new(flags.bits());
            if raw.is_null() {
                Err(Error::Mem)
            } else {
                Ok(RawContext { raw, reader: None })
            }
        }
    }

    /// Set how chunk CRC errors should be handled for critical and ancillary chunks.
    /// ### Note
    /// Partially implemented, `SPNG_CRC_DISCARD` has no effect.
    pub fn set_crc_action(
        &mut self,
        critical: CrcAction,
        ancillary: CrcAction,
    ) -> Result<(), Error> {
        unsafe {
            check_err(sys::spng_set_crc_action(
                self.raw,
                critical as i32,
                ancillary as i32,
            ))
        }
    }

    /// Get image width and height limits.
    ///
    /// Returns `(width, height)`
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

    /// Set image width and height limits, these may not be larger than `(2^31)-1`.
    pub fn set_image_limits(&mut self, max_width: u32, max_height: u32) -> Result<(), Error> {
        unsafe { check_err(sys::spng_set_image_limits(self.raw, max_width, max_height)) }
    }

    /// Get chunk size and chunk cache limits.
    ///
    /// Returns `(chunk_size, cache_size)`
    pub fn get_chunk_limits(&self) -> Result<(usize, usize), Error> {
        let mut chunk_size = 0;
        let mut cache_size = 0;
        unsafe {
            check_err(sys::spng_get_chunk_limits(
                self.raw,
                &mut chunk_size,
                &mut cache_size,
            ))?;
            Ok((chunk_size, cache_size))
        }
    }

    /// Set chunk size and chunk cache limits, the default chunk size limit is `(2^31)-1`, the default
    /// chunk cache limit is `SIZE_MAX`.
    pub fn set_chunk_limits(&mut self, chunk_size: usize, cache_size: usize) -> Result<(), Error> {
        unsafe { check_err(sys::spng_set_chunk_limits(self.raw, chunk_size, cache_size)) }
    }

    /// Get the image header.
    pub fn get_ihdr(&self) -> Result<sys::spng_ihdr, Error> {
        unsafe {
            let mut chunk = MaybeUninit::uninit();
            check_err(sys::spng_get_ihdr(self.raw, chunk.as_mut_ptr()))?;
            Ok(chunk.assume_init())
        }
    }

    /// Get the image palette.
    pub fn get_plte(&self) -> Result<sys::spng_plte, Error> {
        unsafe {
            let mut chunk = MaybeUninit::uninit();
            check_err(sys::spng_get_plte(self.raw, chunk.as_mut_ptr()))?;
            Ok(chunk.assume_init())
        }
    }

    /// Get the image transparency.
    pub fn get_trns(&self) -> Result<sys::spng_trns, Error> {
        unsafe {
            let mut chunk = MaybeUninit::uninit();
            check_err(sys::spng_get_trns(self.raw, chunk.as_mut_ptr()))?;
            Ok(chunk.assume_init())
        }
    }

    /// Get primary chromacities and white point as floating point numbers.
    pub fn get_chrm(&self) -> Result<sys::spng_chrm, Error> {
        unsafe {
            let mut chunk = MaybeUninit::uninit();
            check_err(sys::spng_get_chrm(self.raw, chunk.as_mut_ptr()))?;
            Ok(chunk.assume_init())
        }
    }

    /// Get primary chromacities and white point in the PNG's internal representation.
    pub fn get_chrm_int(&self) -> Result<sys::spng_chrm_int, Error> {
        unsafe {
            let mut chunk = MaybeUninit::uninit();
            check_err(sys::spng_get_chrm_int(self.raw, chunk.as_mut_ptr()))?;
            Ok(chunk.assume_init())
        }
    }

    /// Get the image gamma.
    pub fn get_gama(&self) -> Result<f64, Error> {
        unsafe {
            let mut chunk = MaybeUninit::uninit();
            check_err(sys::spng_get_gama(self.raw, chunk.as_mut_ptr()))?;
            Ok(chunk.assume_init())
        }
    }

    /// Get the ICC profile.
    ///
    /// ### Note
    /// ICC profiles are not validated.
    pub fn get_iccp(&self) -> Result<sys::spng_iccp, Error> {
        unsafe {
            let mut chunk = MaybeUninit::uninit();
            check_err(sys::spng_get_iccp(self.raw, chunk.as_mut_ptr()))?;
            Ok(chunk.assume_init())
        }
    }

    /// Get the significant bits.
    pub fn get_sbit(&self) -> Result<sys::spng_sbit, Error> {
        unsafe {
            let mut chunk = MaybeUninit::uninit();
            check_err(sys::spng_get_sbit(self.raw, chunk.as_mut_ptr()))?;
            Ok(chunk.assume_init())
        }
    }

    /// Get the `sRGB` rendering intent.
    pub fn get_srgb(&self) -> Result<u8, Error> {
        unsafe {
            let mut rendering_intent = 0;
            check_err(sys::spng_get_srgb(self.raw, &mut rendering_intent))?;
            Ok(rendering_intent)
        }
    }

    /// Get text information.
    ///
    /// ### Note
    /// Due to the structure of PNG files it is recommended to call this function after
    /// [`decode_image`] to retrieve all text chunks.
    ///
    /// ### Safety
    /// Text data is freed after the context is dropped.
    ///
    /// [`decode_image`]: method@RawContext::decode_image
    pub fn get_text(&self) -> Result<Vec<sys::spng_text>, Error> {
        unsafe {
            use std::ptr;
            let mut len = 0;
            check_err(sys::spng_get_text(self.raw, ptr::null_mut(), &mut len))?;
            let mut chunk =
                vec![MaybeUninit::<sys::spng_text>::uninit().assume_init(); len as usize];
            check_err(sys::spng_get_text(self.raw, chunk.as_mut_ptr(), &mut len))?;
            Ok(chunk)
        }
    }

    /// Get the image background color.
    pub fn get_bkgd(&self) -> Result<sys::spng_bkgd, Error> {
        unsafe {
            let mut chunk = MaybeUninit::uninit();
            check_err(sys::spng_get_bkgd(self.raw, chunk.as_mut_ptr()))?;
            Ok(chunk.assume_init())
        }
    }

    /// Get physical pixel dimensions.
    pub fn get_phys(&self) -> Result<sys::spng_phys, Error> {
        unsafe {
            let mut chunk = MaybeUninit::uninit();
            check_err(sys::spng_get_phys(self.raw, chunk.as_mut_ptr()))?;
            Ok(chunk.assume_init())
        }
    }

    /// Get the suggested palettes.
    ///
    /// ### Safety
    /// Suggested palettes are freed when the context is dropped.
    ///
    /// [`decode_image`]: method@RawContext::decode_image
    pub fn get_splt(&self) -> Result<Vec<sys::spng_splt>, Error> {
        unsafe {
            use std::ptr;
            let mut len = 0;
            check_err(sys::spng_get_splt(self.raw, ptr::null_mut(), &mut len))?;
            let mut chunk =
                vec![MaybeUninit::<sys::spng_splt>::uninit().assume_init(); len as usize];
            check_err(sys::spng_get_splt(self.raw, chunk.as_mut_ptr(), &mut len))?;
            Ok(mem::transmute(chunk))
        }
    }

    /// Get the modification time.
    ///
    /// ### Note
    /// Due to the structure of PNG files it is recommended to call this function after [`decode_image`].
    ///
    /// [`decode_image`]: method@RawContext::decode_image
    pub fn get_time(&self) -> Result<sys::spng_time, Error> {
        unsafe {
            let mut chunk = MaybeUninit::uninit();
            check_err(sys::spng_get_time(self.raw, chunk.as_mut_ptr()))?;
            Ok(chunk.assume_init())
        }
    }

    /// Get the image offset.
    pub fn get_offs(&self) -> Result<sys::spng_offs, Error> {
        unsafe {
            let mut chunk = MaybeUninit::uninit();
            check_err(sys::spng_get_offs(self.raw, chunk.as_mut_ptr()))?;
            Ok(chunk.assume_init())
        }
    }

    /// Get the `EXIF` data.
    ///
    /// ### Note
    /// Due to the structure of PNG files it is recommended to call this function after [`decode_image`].
    ///
    ///
    /// ### Safety
    /// `exif.data` is freed when the context is dropped.
    ///
    /// [`decode_image`]: method@RawContext::decode_image
    pub fn get_exif(&self) -> Result<sys::spng_exif, Error> {
        unsafe {
            let mut chunk = MaybeUninit::uninit();
            check_err(sys::spng_get_exif(self.raw, chunk.as_mut_ptr()))?;
            Ok(chunk.assume_init())
        }
    }

    /// Get the current, to-be-decoded row's information.
    pub fn get_row_info(&self) -> Result<sys::spng_row_info, Error> {
        unsafe {
            let mut chunk = MaybeUninit::uninit();
            check_err(sys::spng_get_row_info(self.raw, chunk.as_mut_ptr()))?;
            Ok(chunk.assume_init())
        }
    }

    /// Calculates decoded image buffer size for the given output format.
    ///
    /// PNG data must have been set prior with [`set_png_stream`] or [`set_png_buffer`].
    ///
    /// [`set_png_stream`]: method@RawContext::set_png_stream
    /// [`set_png_buffer`]: method@RawContext::set_png_buffer
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

    /// Decodes the PNG file and writes the image to `out`. The image is converted from any PNG format to the
    /// destination format `out_format`. Interlaced images are deinterlaced and `16-bit` images are converted to
    /// host-endian.
    ///
    /// The `out` buffer must have a length greater or equal to the size returned by [`decoded_image_size`] with
    /// the same `out_format`.
    ///
    /// If the `SPNG_DECODE_PROGRESSIVE` flag is set, the context will be initialied with `out_format` for
    /// progressive decoding. The image is not immediately decoded and the `out` buffer is ignored.
    ///
    /// The `SPNG_DECODE_TRNS` flag is ignored if the PNG has an alpha channel or does not contain a `TRNS`
    /// chunk. It is also ignored for gray `1/2/4`-bit images.
    ///
    /// The function may only be called **once** per context.
    ///
    /// [`decode_image`]: method@RawContext::decode_image
    /// [`decoded_image_size`]: method@RawContext::decoded_image_size
    pub fn decode_image(
        &mut self,
        out: &mut [u8],
        out_format: Format,
        flags: DecodeFlags,
    ) -> Result<(), Error> {
        unsafe {
            check_err(sys::spng_decode_image(
                self.raw,
                out.as_mut_ptr() as _,
                out.len(),
                out_format as _,
                flags.bits,
            ))
        }
    }

    /// Decodes and deinterlaces a scanline to `out`.
    ///
    /// This function requires the decoder to be initialized by calling [`decode_image`] with the
    /// `SPNG_DECODE_PROGRESSIVE` flag set.
    ///
    /// The widest scanline is the decoded image size divided by `ihdr.height`.
    ///
    /// For the last scanline and subsequent calls the return value is `SPNG_EOI`.
    ///
    /// If the image is not interlaced this function's behavior is identical to [`decode_scanline`].
    ///
    /// [`decode_image`]: method@RawContext::decode_image
    /// [`decode_scanline`]: method@RawContext::decode_scanline
    pub fn decode_row(&mut self, out: &mut [u8]) -> Result<(), Error> {
        unsafe {
            check_err(sys::spng_decode_row(
                self.raw,
                out.as_mut_ptr() as _,
                out.len(),
            ))
        }
    }

    /// Decodes a scanline to `out`.
    ///
    /// This function requires the decoder to be initialized by calling [`decode_image`] with the
    /// `SPNG_DECODE_PROGRESSIVE` flag set.
    ///
    /// The widest scanline is the decoded image size divided by `ihdr.height`.
    ///
    /// For the last scanline and subsequent calls the return value is `SPNG_EOI`.
    ///
    /// [`decode_image`]: method@RawContext::decode_image
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

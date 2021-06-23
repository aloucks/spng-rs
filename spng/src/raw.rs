//! Raw decoding context

use crate::{
    error::{check_err, Error},
    ContextFlags, CrcAction, DecodeFlags, Format,
};

use self::chunk::*;

use spng_sys as sys;
use std::{io, marker::PhantomData, mem, mem::MaybeUninit, slice};

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
pub trait ChunkAvail<T> {
    /// Converts `Err(Error::Chunkavail)` into `Ok(None)`.
    fn chunk_avail(self) -> Result<Option<T>, Error>;
}

impl<T> ChunkAvail<T> for Result<T, Error> {
    fn chunk_avail(self) -> Result<Option<T>, Error> {
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
            let raw = sys::spng_ctx_new(flags.bits() as _);
            if raw.is_null() {
                Err(Error::Mem)
            } else {
                Ok(RawContext { raw, reader: None })
            }
        }
    }

    /// Set how chunk CRC errors should be handled for critical and ancillary chunks.
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
    pub fn get_ihdr(&self) -> Result<Ihdr, Error> {
        unsafe {
            let mut chunk = MaybeUninit::uninit();
            check_err(sys::spng_get_ihdr(self.raw, chunk.as_mut_ptr()))?;
            Ok(chunk.assume_init())
        }
    }

    /// Get the image palette.
    pub fn get_plte(&self) -> Result<Ref<Plte>, Error> {
        unsafe {
            let mut chunk = MaybeUninit::uninit();
            check_err(sys::spng_get_plte(self.raw, chunk.as_mut_ptr()))?;
            Ok(Ref::from(Plte(chunk.assume_init())))
        }
    }

    /// Get the image transparency.
    pub fn get_trns(&self) -> Result<Trns, Error> {
        unsafe {
            let mut chunk = MaybeUninit::uninit();
            check_err(sys::spng_get_trns(self.raw, chunk.as_mut_ptr()))?;
            Ok(chunk.assume_init())
        }
    }

    /// Get primary chromacities and white point as floating point numbers.
    pub fn get_chrm(&self) -> Result<Chrm, Error> {
        unsafe {
            let mut chunk = MaybeUninit::uninit();
            check_err(sys::spng_get_chrm(self.raw, chunk.as_mut_ptr()))?;
            Ok(chunk.assume_init())
        }
    }

    /// Get primary chromacities and white point in the PNG's internal representation.
    pub fn get_chrm_int(&self) -> Result<ChrmInt, Error> {
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
    pub fn get_iccp(&self) -> Result<Ref<Iccp>, Error> {
        unsafe {
            let mut chunk = MaybeUninit::uninit();
            check_err(sys::spng_get_iccp(self.raw, chunk.as_mut_ptr()))?;
            let chunk: Iccp = mem::transmute(chunk.assume_init());
            Ok(Ref::from(chunk))
        }
    }

    /// Get the significant bits.
    pub fn get_sbit(&self) -> Result<Sbit, Error> {
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
    pub fn get_text(&self) -> Result<Ref<Vec<Text>>, Error> {
        unsafe {
            use std::ptr;
            let mut len = 0;
            check_err(sys::spng_get_text(self.raw, ptr::null_mut(), &mut len))?;
            let mut vec = Vec::<Text>::new();
            vec.reserve_exact(len as usize);
            vec.set_len(len as usize);
            let text_ptr = vec.as_mut_ptr() as *mut sys::spng_text;
            check_err(sys::spng_get_text(self.raw, text_ptr, &mut len))?;
            Ok(Ref::from(vec))
        }
    }

    /// Get the image background color.
    pub fn get_bkgd(&self) -> Result<Bkgd, Error> {
        unsafe {
            let mut chunk = MaybeUninit::uninit();
            check_err(sys::spng_get_bkgd(self.raw, chunk.as_mut_ptr()))?;
            Ok(chunk.assume_init())
        }
    }

    /// Get the image histogram.
    pub fn get_hist(&self) -> Result<Hist, Error> {
        unsafe {
            let mut chunk = MaybeUninit::uninit();
            check_err(sys::spng_get_hist(self.raw, chunk.as_mut_ptr()))?;
            Ok(chunk.assume_init())
        }
    }

    /// Get physical pixel dimensions.
    pub fn get_phys(&self) -> Result<Phys, Error> {
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
    pub fn get_splt(&self) -> Result<Ref<Vec<Splt>>, Error> {
        unsafe {
            use std::ptr;
            let mut len = 0;
            check_err(sys::spng_get_splt(self.raw, ptr::null_mut(), &mut len))?;
            let mut vec = Vec::<Splt>::new();
            vec.reserve_exact(len as usize);
            vec.set_len(len as usize);
            let splt_ptr = vec.as_mut_ptr() as *mut sys::spng_splt;
            check_err(sys::spng_get_splt(self.raw, splt_ptr, &mut len))?;
            Ok(Ref::from(vec))
        }
    }

    /// Get the modification time.
    ///
    /// ### Note
    /// Due to the structure of PNG files it is recommended to call this function after [`decode_image`].
    ///
    /// [`decode_image`]: method@RawContext::decode_image
    pub fn get_time(&self) -> Result<Time, Error> {
        unsafe {
            let mut chunk = MaybeUninit::uninit();
            check_err(sys::spng_get_time(self.raw, chunk.as_mut_ptr()))?;
            Ok(chunk.assume_init())
        }
    }

    /// Get the image offset.
    pub fn get_offs(&self) -> Result<Offs, Error> {
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
    pub fn get_exif(&self) -> Result<Ref<Exif>, Error> {
        unsafe {
            let mut chunk = MaybeUninit::uninit();
            check_err(sys::spng_get_exif(self.raw, chunk.as_mut_ptr()))?;
            let chunk: Exif = mem::transmute(chunk.assume_init());
            Ok(Ref::from(chunk))
        }
    }

    /// Get the current, to-be-decoded row's information.
    pub fn get_row_info(&self) -> Result<RowInfo, Error> {
        unsafe {
            let mut chunk = MaybeUninit::uninit();
            check_err(sys::spng_get_row_info(self.raw, chunk.as_mut_ptr()))?;
            Ok(chunk.assume_init())
        }
    }

    pub fn get_unknown_chunks(&self) -> Result<Ref<Vec<UnknownChunk>>, Error> {
        unsafe {
            use std::ptr;
            let mut len = 0;
            check_err(sys::spng_get_unknown_chunks(
                self.raw,
                ptr::null_mut(),
                &mut len,
            ))?;
            let mut vec = Vec::<UnknownChunk>::new();
            vec.reserve_exact(len as usize);
            vec.set_len(len as usize);
            let chunk_ptr = vec.as_mut_ptr() as *mut sys::spng_unknown_chunk;
            check_err(sys::spng_get_unknown_chunks(self.raw, chunk_ptr, &mut len))?;
            Ok(Ref::from(vec))
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
                flags.bits as _,
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

/// An owned reference.
///
/// Attaches lifetime `'a` to `T`.
pub struct Ref<'a, T: 'a> {
    data: T,
    _p: PhantomData<&'a ()>,
}

impl<'a, T: 'a> std::ops::Deref for Ref<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<'a, T: 'a> From<T> for Ref<'a, T> {
    fn from(t: T) -> Ref<'a, T> {
        Ref {
            data: t,
            _p: PhantomData,
        }
    }
}

/// `PNG` chunk data
pub mod chunk {
    use spng_sys as sys;
    use std::{ffi::CStr, slice};

    /// Safe wrapper for [`spng_sys::spng_splt`]
    #[repr(C)]
    pub struct Splt(pub(crate) sys::spng_splt);

    impl Splt {
        pub fn name(&self) -> Result<&str, std::str::Utf8Error> {
            unsafe { CStr::from_ptr(self.0.name.as_ptr() as _).to_str() }
        }

        pub fn sample_depth(&self) -> u8 {
            self.0.sample_depth
        }

        pub fn entries(&self) -> &[sys::spng_splt_entry] {
            unsafe { slice::from_raw_parts(self.0.entries, self.0.n_entries as usize) }
        }
    }

    /// Safe wrapper for [`spng_sys::spng_plte`]
    #[repr(C)]
    pub struct Plte(pub(crate) sys::spng_plte);

    impl Plte {
        pub fn entries(&self) -> &[PlteEntry] {
            unsafe { slice::from_raw_parts(self.0.entries.as_ptr(), self.0.n_entries as usize) }
        }
    }

    /// Safe wrapper for [`spng_sys::spng_exif`]
    #[repr(C)]
    pub struct Exif(pub(crate) sys::spng_exif);

    impl Exif {
        pub fn data(&self) -> &[u8] {
            unsafe { slice::from_raw_parts(self.0.data as _, self.0.length as usize) }
        }
    }

    /// Safe wrapper for [`spng_sys::spng_text`]
    #[repr(C)]
    pub struct Text(pub(crate) sys::spng_text);

    impl Text {
        pub fn keyword(&self) -> Result<&str, std::str::Utf8Error> {
            unsafe { CStr::from_ptr(self.0.keyword.as_ptr() as _).to_str() }
        }

        pub fn type_(&self) -> i32 {
            self.0.type_
        }

        pub fn length(&self) -> usize {
            self.0.length
        }

        pub fn text(&self) -> Result<&str, std::str::Utf8Error> {
            unsafe { CStr::from_ptr(self.0.text).to_str() }
        }

        pub fn compression_flag(&self) -> u8 {
            self.0.compression_flag
        }

        pub fn compression_method(&self) -> u8 {
            self.0.compression_method
        }

        pub fn language_tag(&self) -> Result<&str, std::str::Utf8Error> {
            unsafe { CStr::from_ptr(self.0.language_tag).to_str() }
        }

        pub fn translated_keyword(&self) -> Result<&str, std::str::Utf8Error> {
            unsafe { CStr::from_ptr(self.0.translated_keyword).to_str() }
        }
    }

    /// Safe wrapper for [`spng_sys::spng_iccp`]
    #[repr(C)]
    pub struct Iccp(pub(crate) sys::spng_iccp);

    impl Iccp {
        pub fn profile_name(&self) -> Result<&str, std::str::Utf8Error> {
            unsafe { CStr::from_ptr(self.0.profile_name.as_ptr()).to_str() }
        }

        pub fn profile(&self) -> &[u8] {
            unsafe { slice::from_raw_parts(self.0.profile as _, self.0.profile_len as usize) }
        }
    }

    #[repr(C)]
    pub struct UnknownChunk(pub(crate) spng_sys::spng_unknown_chunk);

    impl UnknownChunk {
        /// Returns the chunk type or `None` if it could not be parsed as valid `utf-8`.
        pub fn type_(&self) -> Option<&str> {
            std::str::from_utf8(&self.0.type_).ok()
        }

        /// Returns the chunk data.
        pub fn data(&self) -> &[u8] {
            unsafe { slice::from_raw_parts(self.0.data as _, self.0.length as usize) }
        }
    }

    /// Image header
    pub type Ihdr = sys::spng_ihdr;
    /// Transparency
    pub type Trns = sys::spng_trns;
    /// Primary chromacities and white point as floating point numbers
    pub type Chrm = sys::spng_chrm;
    /// Primary chromacities and white point in the PNG's internal representation
    pub type ChrmInt = sys::spng_chrm_int;
    /// Significant bits
    pub type Sbit = sys::spng_sbit;
    /// Background color
    pub type Bkgd = sys::spng_bkgd;
    /// Histogram
    pub type Hist = sys::spng_hist;
    /// Physical pixel dimensions
    pub type Phys = sys::spng_phys;
    /// Modification time
    pub type Time = sys::spng_time;
    /// Offset
    pub type Offs = sys::spng_offs;
    /// To-be-decoded row information
    pub type RowInfo = sys::spng_row_info;
    /// Palette entry
    pub type PlteEntry = spng_sys::spng_plte_entry;
}

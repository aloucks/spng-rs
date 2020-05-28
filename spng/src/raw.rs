use crate::{
    error::{check_err, Error},
    ContextFlags, DecodeFlags, Format,
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
            let mut chunk = MaybeUninit::uninit();
            check_err(sys::spng_get_ihdr(self.raw, chunk.as_mut_ptr()))?;
            Ok(chunk.assume_init())
        }
    }

    pub fn get_plte(&self) -> Result<sys::spng_plte, Error> {
        unsafe {
            let mut chunk = MaybeUninit::uninit();
            check_err(sys::spng_get_plte(self.raw, chunk.as_mut_ptr()))?;
            Ok(chunk.assume_init())
        }
    }

    pub fn get_trns(&self) -> Result<sys::spng_trns, Error> {
        unsafe {
            let mut chunk = MaybeUninit::uninit();
            check_err(sys::spng_get_trns(self.raw, chunk.as_mut_ptr()))?;
            Ok(chunk.assume_init())
        }
    }

    pub fn get_chrm(&self) -> Result<sys::spng_chrm, Error> {
        unsafe {
            let mut chunk = MaybeUninit::uninit();
            check_err(sys::spng_get_chrm(self.raw, chunk.as_mut_ptr()))?;
            Ok(chunk.assume_init())
        }
    }

    pub fn get_chrm_int(&self) -> Result<sys::spng_chrm_int, Error> {
        unsafe {
            let mut chunk = MaybeUninit::uninit();
            check_err(sys::spng_get_chrm_int(self.raw, chunk.as_mut_ptr()))?;
            Ok(chunk.assume_init())
        }
    }

    pub fn get_gama(&self) -> Result<f64, Error> {
        unsafe {
            let mut chunk = MaybeUninit::uninit();
            check_err(sys::spng_get_gama(self.raw, chunk.as_mut_ptr()))?;
            Ok(chunk.assume_init())
        }
    }

    pub fn get_iccp(&self) -> Result<sys::spng_iccp, Error> {
        unsafe {
            let mut chunk = MaybeUninit::uninit();
            check_err(sys::spng_get_iccp(self.raw, chunk.as_mut_ptr()))?;
            Ok(chunk.assume_init())
        }
    }

    pub fn get_sbit(&self) -> Result<sys::spng_sbit, Error> {
        unsafe {
            let mut chunk = MaybeUninit::uninit();
            check_err(sys::spng_get_sbit(self.raw, chunk.as_mut_ptr()))?;
            Ok(chunk.assume_init())
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
            let mut chunk =
                vec![MaybeUninit::<sys::spng_text>::uninit().assume_init(); len as usize];
            check_err(sys::spng_get_text(self.raw, chunk.as_mut_ptr(), &mut len))?;
            Ok(mem::transmute(chunk))
        }
    }

    pub fn get_bkgd(&self) -> Result<sys::spng_bkgd, Error> {
        unsafe {
            let mut chunk = MaybeUninit::uninit();
            check_err(sys::spng_get_bkgd(self.raw, chunk.as_mut_ptr()))?;
            Ok(chunk.assume_init())
        }
    }

    /// Return the physical pixel dimensions
    pub fn get_phys(&self) -> Result<sys::spng_phys, Error> {
        unsafe {
            let mut chunk = MaybeUninit::uninit();
            check_err(sys::spng_get_phys(self.raw, chunk.as_mut_ptr()))?;
            Ok(chunk.assume_init())
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
            let mut chunk =
                vec![MaybeUninit::<sys::spng_splt>::uninit().assume_init(); len as usize];
            check_err(sys::spng_get_splt(self.raw, chunk.as_mut_ptr(), &mut len))?;
            Ok(mem::transmute(chunk))
        }
    }

    pub fn get_time(&self) -> Result<sys::spng_time, Error> {
        unsafe {
            let mut chunk = MaybeUninit::uninit();
            check_err(sys::spng_get_time(self.raw, chunk.as_mut_ptr()))?;
            Ok(chunk.assume_init())
        }
    }

    /// Return the image offset
    pub fn get_offs(&self) -> Result<sys::spng_offs, Error> {
        unsafe {
            let mut chunk = MaybeUninit::uninit();
            check_err(sys::spng_get_offs(self.raw, chunk.as_mut_ptr()))?;
            Ok(chunk.assume_init())
        }
    }

    /// Return `EXIF` data.
    ///
    /// Note that `exif.data` is freed when the context is destroyed.
    pub fn get_exif(&self) -> Result<sys::spng_exif, Error> {
        unsafe {
            let mut chunk = MaybeUninit::uninit();
            check_err(sys::spng_get_exif(self.raw, chunk.as_mut_ptr()))?;
            Ok(chunk.assume_init())
        }
    }

    pub fn get_row_info(&self) -> Result<sys::spng_row_info, Error> {
        unsafe {
            let mut chunk = MaybeUninit::uninit();
            check_err(sys::spng_get_row_info(self.raw, chunk.as_mut_ptr()))?;
            Ok(chunk.assume_init())
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

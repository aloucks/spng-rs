use std::error::Error as StdError;
use std::fmt;

use spng_sys as sys;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum Error {
    IoError = sys::spng_errno_SPNG_IO_ERROR,
    IoEof = sys::spng_errno_SPNG_IO_EOF,
    // Ok = sys::spng_errno_SPNG_OK,
    Inval = sys::spng_errno_SPNG_EINVAL,
    Mem = sys::spng_errno_SPNG_EMEM,
    Overflow = sys::spng_errno_SPNG_EOVERFLOW,
    Signature = sys::spng_errno_SPNG_ESIGNATURE,
    Width = sys::spng_errno_SPNG_EWIDTH,
    Height = sys::spng_errno_SPNG_EHEIGHT,
    UserWidth = sys::spng_errno_SPNG_EUSER_WIDTH,
    UserHeight = sys::spng_errno_SPNG_EUSER_HEIGHT,
    BitDepth = sys::spng_errno_SPNG_EBIT_DEPTH,
    ColorType = sys::spng_errno_SPNG_ECOLOR_TYPE,
    CompressionMethod = sys::spng_errno_SPNG_ECOMPRESSION_METHOD,
    FilterMethod = sys::spng_errno_SPNG_EFILTER_METHOD,
    InterlaceMethod = sys::spng_errno_SPNG_EINTERLACE_METHOD,
    IhdrSize = sys::spng_errno_SPNG_EIHDR_SIZE,
    Noihdr = sys::spng_errno_SPNG_ENOIHDR,
    ChunkPos = sys::spng_errno_SPNG_ECHUNK_POS,
    ChunkSize = sys::spng_errno_SPNG_ECHUNK_SIZE,
    ChunkCrc = sys::spng_errno_SPNG_ECHUNK_CRC,
    ChunkType = sys::spng_errno_SPNG_ECHUNK_TYPE,
    ChunkUnknownCritical = sys::spng_errno_SPNG_ECHUNK_UNKNOWN_CRITICAL,
    DupPlte = sys::spng_errno_SPNG_EDUP_PLTE,
    DupChrm = sys::spng_errno_SPNG_EDUP_CHRM,
    DupGama = sys::spng_errno_SPNG_EDUP_GAMA,
    DupIccp = sys::spng_errno_SPNG_EDUP_ICCP,
    DupSbit = sys::spng_errno_SPNG_EDUP_SBIT,
    DupSrgb = sys::spng_errno_SPNG_EDUP_SRGB,
    DupBkgd = sys::spng_errno_SPNG_EDUP_BKGD,
    DupHist = sys::spng_errno_SPNG_EDUP_HIST,
    DupTrns = sys::spng_errno_SPNG_EDUP_TRNS,
    DupPhys = sys::spng_errno_SPNG_EDUP_PHYS,
    DupTime = sys::spng_errno_SPNG_EDUP_TIME,
    DupOffs = sys::spng_errno_SPNG_EDUP_OFFS,
    DupExif = sys::spng_errno_SPNG_EDUP_EXIF,
    Chrm = sys::spng_errno_SPNG_ECHRM,
    PlteIdx = sys::spng_errno_SPNG_EPLTE_IDX,
    TrnsColorType = sys::spng_errno_SPNG_ETRNS_COLOR_TYPE,
    TrnsNoPlte = sys::spng_errno_SPNG_ETRNS_NO_PLTE,
    Gama = sys::spng_errno_SPNG_EGAMA,
    IccpName = sys::spng_errno_SPNG_EICCP_NAME,
    IccpCompressionMethod = sys::spng_errno_SPNG_EICCP_COMPRESSION_METHOD,
    Sbit = sys::spng_errno_SPNG_ESBIT,
    Srgb = sys::spng_errno_SPNG_ESRGB,
    Text = sys::spng_errno_SPNG_ETEXT,
    TextKeyword = sys::spng_errno_SPNG_ETEXT_KEYWORD,
    Ztxt = sys::spng_errno_SPNG_EZTXT,
    ZtxtCompressionMethod = sys::spng_errno_SPNG_EZTXT_COMPRESSION_METHOD,
    Itxt = sys::spng_errno_SPNG_EITXT,
    ItxtCompressionFlag = sys::spng_errno_SPNG_EITXT_COMPRESSION_FLAG,
    ItxtCompressionMethod = sys::spng_errno_SPNG_EITXT_COMPRESSION_METHOD,
    ItxtLangTag = sys::spng_errno_SPNG_EITXT_LANG_TAG,
    ItxtTranslatedKey = sys::spng_errno_SPNG_EITXT_TRANSLATED_KEY,
    BkgdNoPlte = sys::spng_errno_SPNG_EBKGD_NO_PLTE,
    BkgdPlteIdx = sys::spng_errno_SPNG_EBKGD_PLTE_IDX,
    HistNoPlte = sys::spng_errno_SPNG_EHIST_NO_PLTE,
    Phys = sys::spng_errno_SPNG_EPHYS,
    SpltName = sys::spng_errno_SPNG_ESPLT_NAME,
    SpltDupName = sys::spng_errno_SPNG_ESPLT_DUP_NAME,
    SpltDepth = sys::spng_errno_SPNG_ESPLT_DEPTH,
    Time = sys::spng_errno_SPNG_ETIME,
    Offs = sys::spng_errno_SPNG_EOFFS,
    Exif = sys::spng_errno_SPNG_EEXIF,
    IdatTooShort = sys::spng_errno_SPNG_EIDAT_TOO_SHORT,
    IdatStream = sys::spng_errno_SPNG_EIDAT_STREAM,
    Zlib = sys::spng_errno_SPNG_EZLIB,
    Filter = sys::spng_errno_SPNG_EFILTER,
    Bufsiz = sys::spng_errno_SPNG_EBUFSIZ,
    Io = sys::spng_errno_SPNG_EIO,
    Eof = sys::spng_errno_SPNG_EOF,
    BufSet = sys::spng_errno_SPNG_EBUF_SET,
    Badstate = sys::spng_errno_SPNG_EBADSTATE,
    Fmt = sys::spng_errno_SPNG_EFMT,
    Flags = sys::spng_errno_SPNG_EFLAGS,
    Chunkavail = sys::spng_errno_SPNG_ECHUNKAVAIL,
    NcodeOnly = sys::spng_errno_SPNG_ENCODE_ONLY,
    Oi = sys::spng_errno_SPNG_EOI,
    Noplte = sys::spng_errno_SPNG_ENOPLTE,
    ChunkLimits = sys::spng_errno_SPNG_ECHUNK_LIMITS,
    ZlibInit = sys::spng_errno_SPNG_EZLIB_INIT,
    ChunkStdlen = sys::spng_errno_SPNG_ECHUNK_STDLEN,
    Internal = sys::spng_errno_SPNG_EINTERNAL,
    CtxType = sys::spng_errno_SPNG_ECTXTYPE,
    NoSrc = sys::spng_errno_SPNG_ENOSRC,
    NoDst = sys::spng_errno_SPNG_ENODST,
    OpState = sys::spng_errno_SPNG_EOPSTATE,
    NotFinal = sys::spng_errno_SPNG_ENOTFINAL,
}

pub fn check_err(e: i32) -> Result<(), Error> {
    use Error::*;
    match e {
        sys::spng_errno_SPNG_IO_ERROR => Err(IoError),
        sys::spng_errno_SPNG_IO_EOF => Err(IoEof),
        sys::spng_errno_SPNG_OK => Ok(()),
        sys::spng_errno_SPNG_EINVAL => Err(Inval),
        sys::spng_errno_SPNG_EMEM => Err(Mem),
        sys::spng_errno_SPNG_EOVERFLOW => Err(Overflow),
        sys::spng_errno_SPNG_ESIGNATURE => Err(Signature),
        sys::spng_errno_SPNG_EWIDTH => Err(Width),
        sys::spng_errno_SPNG_EHEIGHT => Err(Height),
        sys::spng_errno_SPNG_EUSER_WIDTH => Err(UserWidth),
        sys::spng_errno_SPNG_EUSER_HEIGHT => Err(UserHeight),
        sys::spng_errno_SPNG_EBIT_DEPTH => Err(BitDepth),
        sys::spng_errno_SPNG_ECOLOR_TYPE => Err(ColorType),
        sys::spng_errno_SPNG_ECOMPRESSION_METHOD => Err(CompressionMethod),
        sys::spng_errno_SPNG_EFILTER_METHOD => Err(FilterMethod),
        sys::spng_errno_SPNG_EINTERLACE_METHOD => Err(InterlaceMethod),
        sys::spng_errno_SPNG_EIHDR_SIZE => Err(IhdrSize),
        sys::spng_errno_SPNG_ENOIHDR => Err(Noihdr),
        sys::spng_errno_SPNG_ECHUNK_POS => Err(ChunkPos),
        sys::spng_errno_SPNG_ECHUNK_SIZE => Err(ChunkSize),
        sys::spng_errno_SPNG_ECHUNK_CRC => Err(ChunkCrc),
        sys::spng_errno_SPNG_ECHUNK_TYPE => Err(ChunkType),
        sys::spng_errno_SPNG_ECHUNK_UNKNOWN_CRITICAL => Err(ChunkUnknownCritical),
        sys::spng_errno_SPNG_EDUP_PLTE => Err(DupPlte),
        sys::spng_errno_SPNG_EDUP_CHRM => Err(DupChrm),
        sys::spng_errno_SPNG_EDUP_GAMA => Err(DupGama),
        sys::spng_errno_SPNG_EDUP_ICCP => Err(DupIccp),
        sys::spng_errno_SPNG_EDUP_SBIT => Err(DupSbit),
        sys::spng_errno_SPNG_EDUP_SRGB => Err(DupSrgb),
        sys::spng_errno_SPNG_EDUP_BKGD => Err(DupBkgd),
        sys::spng_errno_SPNG_EDUP_HIST => Err(DupHist),
        sys::spng_errno_SPNG_EDUP_TRNS => Err(DupTrns),
        sys::spng_errno_SPNG_EDUP_PHYS => Err(DupPhys),
        sys::spng_errno_SPNG_EDUP_TIME => Err(DupTime),
        sys::spng_errno_SPNG_EDUP_OFFS => Err(DupOffs),
        sys::spng_errno_SPNG_EDUP_EXIF => Err(DupExif),
        sys::spng_errno_SPNG_ECHRM => Err(Chrm),
        sys::spng_errno_SPNG_EPLTE_IDX => Err(PlteIdx),
        sys::spng_errno_SPNG_ETRNS_COLOR_TYPE => Err(TrnsColorType),
        sys::spng_errno_SPNG_ETRNS_NO_PLTE => Err(TrnsNoPlte),
        sys::spng_errno_SPNG_EGAMA => Err(Gama),
        sys::spng_errno_SPNG_EICCP_NAME => Err(IccpName),
        sys::spng_errno_SPNG_EICCP_COMPRESSION_METHOD => Err(IccpCompressionMethod),
        sys::spng_errno_SPNG_ESBIT => Err(Sbit),
        sys::spng_errno_SPNG_ESRGB => Err(Srgb),
        sys::spng_errno_SPNG_ETEXT => Err(Text),
        sys::spng_errno_SPNG_ETEXT_KEYWORD => Err(TextKeyword),
        sys::spng_errno_SPNG_EZTXT => Err(Ztxt),
        sys::spng_errno_SPNG_EZTXT_COMPRESSION_METHOD => Err(ZtxtCompressionMethod),
        sys::spng_errno_SPNG_EITXT => Err(Itxt),
        sys::spng_errno_SPNG_EITXT_COMPRESSION_FLAG => Err(ItxtCompressionFlag),
        sys::spng_errno_SPNG_EITXT_COMPRESSION_METHOD => Err(ItxtCompressionMethod),
        sys::spng_errno_SPNG_EITXT_LANG_TAG => Err(ItxtLangTag),
        sys::spng_errno_SPNG_EITXT_TRANSLATED_KEY => Err(ItxtTranslatedKey),
        sys::spng_errno_SPNG_EBKGD_NO_PLTE => Err(BkgdNoPlte),
        sys::spng_errno_SPNG_EBKGD_PLTE_IDX => Err(BkgdPlteIdx),
        sys::spng_errno_SPNG_EHIST_NO_PLTE => Err(HistNoPlte),
        sys::spng_errno_SPNG_EPHYS => Err(Phys),
        sys::spng_errno_SPNG_ESPLT_NAME => Err(SpltName),
        sys::spng_errno_SPNG_ESPLT_DUP_NAME => Err(SpltDupName),
        sys::spng_errno_SPNG_ESPLT_DEPTH => Err(SpltDepth),
        sys::spng_errno_SPNG_ETIME => Err(Time),
        sys::spng_errno_SPNG_EOFFS => Err(Offs),
        sys::spng_errno_SPNG_EEXIF => Err(Exif),
        sys::spng_errno_SPNG_EIDAT_TOO_SHORT => Err(IdatTooShort),
        sys::spng_errno_SPNG_EIDAT_STREAM => Err(IdatStream),
        sys::spng_errno_SPNG_EZLIB => Err(Zlib),
        sys::spng_errno_SPNG_EFILTER => Err(Filter),
        sys::spng_errno_SPNG_EBUFSIZ => Err(Bufsiz),
        sys::spng_errno_SPNG_EIO => Err(Io),
        sys::spng_errno_SPNG_EOF => Err(Eof),
        sys::spng_errno_SPNG_EBUF_SET => Err(BufSet),
        sys::spng_errno_SPNG_EBADSTATE => Err(Badstate),
        sys::spng_errno_SPNG_EFMT => Err(Fmt),
        sys::spng_errno_SPNG_EFLAGS => Err(Flags),
        sys::spng_errno_SPNG_ECHUNKAVAIL => Err(Chunkavail),
        sys::spng_errno_SPNG_ENCODE_ONLY => Err(NcodeOnly),
        sys::spng_errno_SPNG_EOI => Err(Oi),
        sys::spng_errno_SPNG_ENOPLTE => Err(Noplte),
        sys::spng_errno_SPNG_ECHUNK_LIMITS => Err(ChunkLimits),
        sys::spng_errno_SPNG_EZLIB_INIT => Err(ZlibInit),
        sys::spng_errno_SPNG_ECHUNK_STDLEN => Err(ChunkStdlen),
        sys::spng_errno_SPNG_EINTERNAL => Err(Internal),
        sys::spng_errno_SPNG_ECTXTYPE => Err(CtxType),
        sys::spng_errno_SPNG_ENOSRC => Err(NoSrc),
        sys::spng_errno_SPNG_ENODST => Err(NoDst),
        sys::spng_errno_SPNG_EOPSTATE => Err(OpState),
        sys::spng_errno_SPNG_ENOTFINAL => Err(NotFinal),
        _ => {
            eprintln!("unknown spng error code: {}", e);
            Err(Inval)
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let errno = *self as i32;
        unsafe {
            let ptr = sys::spng_strerror(errno);
            let s = std::ffi::CStr::from_ptr(ptr);
            write!(f, "{}", s.to_string_lossy())
        }
    }
}

impl StdError for Error {}

use spng::{
    raw::{chunk::Ihdr, ChunkAvail, RawContext},
    BitDepth, ColorType, ContextFlags, Decoder, EncodeFlags,
};
use std::io::{BufReader, Cursor, Read};

static TEST_PNG_001: &[u8] = include_bytes!("test-001.png");
static TEST_PNG_002: &[u8] = include_bytes!("test-002.png");

fn check_decoder<R: Read>(
    decoder: Decoder<R>,
    width: u32,
    height: u32,
    bit_depth: BitDepth,
    color_type: ColorType,
) {
    let mut reader = decoder.read_info().expect("read_info failed");
    let info = reader.info();
    let output_buffer_size = reader.output_buffer_size();
    assert_eq!(width, info.width);
    assert_eq!(height, info.height);
    assert_eq!(bit_depth, info.bit_depth);
    assert_eq!(color_type, info.color_type);
    let mut out = vec![0; output_buffer_size];
    let out_info = reader.next_frame(&mut out).expect("next_frame failed");
    assert_eq!(output_buffer_size, out_info.buffer_size());
    assert_eq!(info.width, out_info.width);
    assert_eq!(info.height, out_info.height);
    assert_eq!(info.bit_depth, out_info.bit_depth);
    assert_eq!(info.color_type, out_info.color_type);
}

#[test]
fn decode_001_cursor() {
    let cursor = Cursor::new(TEST_PNG_001);
    let decoder = Decoder::new(cursor);

    check_decoder(decoder, 300, 300, BitDepth::Eight, ColorType::RGBA);
}

#[test]
fn decode_001_cursor_buffered() {
    let cursor = Cursor::new(TEST_PNG_001);
    let cursor_buffered = BufReader::new(cursor);
    let decoder = Decoder::new(cursor_buffered);

    check_decoder(decoder, 300, 300, BitDepth::Eight, ColorType::RGBA);
}

#[test]
fn decode_001_slice() {
    let decoder = Decoder::new(TEST_PNG_001);

    check_decoder(decoder, 300, 300, BitDepth::Eight, ColorType::RGBA);
}

#[test]
fn decode_002_cursor() {
    let cursor = Cursor::new(TEST_PNG_002);
    let decoder = Decoder::new(cursor);

    check_decoder(decoder, 380, 287, BitDepth::Eight, ColorType::RGBA);
}

#[test]
fn decode_002_cursor_buffered() {
    let cursor = Cursor::new(TEST_PNG_002);
    let cursor_buffered = BufReader::new(cursor);
    let decoder = Decoder::new(cursor_buffered);

    check_decoder(decoder, 380, 287, BitDepth::Eight, ColorType::RGBA);
}

#[test]
fn decode_002_slice() {
    let decoder = Decoder::new(TEST_PNG_002);

    check_decoder(decoder, 380, 287, BitDepth::Eight, ColorType::RGBA);
}

#[test]
fn decode() -> Result<(), Box<dyn std::error::Error>> {
    let (out_info, out) = spng::decode(TEST_PNG_001, spng::Format::Png)?;
    assert_eq!(300, out_info.width);
    assert_eq!(300, out_info.height);
    assert_eq!(8, out_info.bit_depth as u8);
    assert_eq!(4, out_info.color_type.samples());
    assert_eq!(out_info.buffer_size, out.len());
    Ok(())
}

#[test]
fn decode_001_raw_context() -> Result<(), Box<dyn std::error::Error>> {
    use std::convert::TryFrom;
    let out_format = spng::Format::Rgba8;
    let mut ctx = spng::raw::RawContext::new()?;
    ctx.set_png_stream_reader(TEST_PNG_001)?;
    let ihdr = ctx.get_ihdr()?;
    assert_eq!(300, ihdr.width);
    assert_eq!(300, ihdr.height);
    assert_eq!(8, ihdr.bit_depth);
    assert_eq!(4, spng::ColorType::try_from(ihdr.color_type)?.samples());
    let buffer_size = ctx.decoded_image_size(out_format)?;
    let mut data = vec![0; buffer_size];
    ctx.decode_image(&mut data, out_format, spng::DecodeFlags::empty())?;
    let text = ctx
        .get_text()
        .chunk_avail()?
        .expect("text chunk in test image");
    let text_str = text[0].text()?;
    assert_eq!("Created with GIMP", text_str);
    Ok(())
}

#[test]
fn encode_001_raw_context() -> Result<(), Box<dyn std::error::Error>> {
    let fmt = spng::Format::Rgba8;
    let (out_info, data) = spng::decode(Cursor::new(TEST_PNG_001), fmt)?;
    let mut ctx = RawContext::with_flags(ContextFlags::ENCODER)?;
    let out_file = std::fs::File::create("target/out.png")?;
    ctx.set_ihdr(Ihdr {
        width: out_info.width,
        height: out_info.height,
        bit_depth: out_info.bit_depth as _,
        color_type: out_info.color_type as _,
        compression_method: 0,
        filter_method: 0,
        interlace_method: 0,
    })?;
    ctx.set_png_stream_writer(out_file)?;
    ctx.encode_image(&data, spng::Format::Png, EncodeFlags::empty())?;
    Ok(())
}

#[test]
fn version() {
    println!("{:?}", spng::version());
}

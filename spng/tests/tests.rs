use spng::Decoder;
use std::io::Cursor;

static TEST_PNG_001: &[u8] = include_bytes!("test-001.png");
static TEST_PNG_002: &[u8] = include_bytes!("test-002.png");

#[test]
fn decode_001_cursor() -> Result<(), Box<dyn std::error::Error>> {
    let cursor = Cursor::new(TEST_PNG_001);
    let decoder = Decoder::new(cursor);
    let (out_info, mut reader) = decoder.read_info()?;
    let output_buffer_size = reader.output_buffer_size();
    assert_eq!(300, out_info.width);
    assert_eq!(300, out_info.height);
    assert_eq!(8, out_info.bit_depth as u8);
    assert_eq!(4, out_info.color_type.samples());
    assert_eq!(out_info.buffer_size, output_buffer_size);
    let mut out = vec![0; output_buffer_size];
    reader.next_frame(&mut out)?;
    Ok(())
}

#[test]
fn decode_001_cursor_buffered() -> Result<(), Box<dyn std::error::Error>> {
    use std::io;

    let cursor = io::BufReader::new(Cursor::new(TEST_PNG_001));
    let decoder = Decoder::new(cursor);
    let (out_info, mut reader) = decoder.read_info()?;
    let output_buffer_size = reader.output_buffer_size();
    assert_eq!(300, out_info.width);
    assert_eq!(300, out_info.height);
    assert_eq!(8, out_info.bit_depth as u8);
    assert_eq!(4, out_info.color_type.samples());
    assert_eq!(out_info.buffer_size, output_buffer_size);
    let mut out = vec![0; output_buffer_size];
    reader.next_frame(&mut out)?;
    Ok(())
}

#[test]
fn decode_001_slice() -> Result<(), Box<dyn std::error::Error>> {
    let decoder = Decoder::new(TEST_PNG_001);
    let (out_info, mut reader) = decoder.read_info()?;
    let output_buffer_size = reader.output_buffer_size();
    assert_eq!(300, out_info.width);
    assert_eq!(300, out_info.height);
    assert_eq!(8, out_info.bit_depth as u8);
    assert_eq!(4, out_info.color_type.samples());
    assert_eq!(out_info.buffer_size, output_buffer_size);
    let mut out = vec![0; output_buffer_size];
    reader.next_frame(&mut out)?;
    Ok(())
}

#[test]
fn decode_002_cursor() -> Result<(), Box<dyn std::error::Error>> {
    let cursor = Cursor::new(TEST_PNG_002);
    let decoder = Decoder::new(cursor);
    let (out_info, mut reader) = decoder.read_info()?;
    let output_buffer_size = reader.output_buffer_size();
    assert_eq!(380, out_info.width);
    assert_eq!(287, out_info.height);
    assert_eq!(8, out_info.bit_depth as u8);
    assert_eq!(4, out_info.color_type.samples());
    assert_eq!(out_info.buffer_size, output_buffer_size);
    let mut out = vec![0; output_buffer_size];
    reader.next_frame(&mut out)?;
    Ok(())
}

#[test]
fn decode_002_cursor_buffered() -> Result<(), Box<dyn std::error::Error>> {
    use std::io;

    let cursor = io::BufReader::new(Cursor::new(TEST_PNG_002));
    let decoder = Decoder::new(cursor);
    let (out_info, mut reader) = decoder.read_info()?;
    let output_buffer_size = reader.output_buffer_size();
    assert_eq!(380, out_info.width);
    assert_eq!(287, out_info.height);
    assert_eq!(8, out_info.bit_depth as u8);
    assert_eq!(4, out_info.color_type.samples());
    assert_eq!(out_info.buffer_size, output_buffer_size);
    let mut out = vec![0; output_buffer_size];
    reader.next_frame(&mut out)?;
    Ok(())
}

#[test]
fn decode_002_slice() -> Result<(), Box<dyn std::error::Error>> {
    let decoder = Decoder::new(TEST_PNG_002);
    let (out_info, mut reader) = decoder.read_info()?;
    let output_buffer_size = reader.output_buffer_size();
    assert_eq!(380, out_info.width);
    assert_eq!(287, out_info.height);
    assert_eq!(8, out_info.bit_depth as u8);
    assert_eq!(4, out_info.color_type.samples());
    assert_eq!(out_info.buffer_size, output_buffer_size);
    let mut out = vec![0; output_buffer_size];
    reader.next_frame(&mut out)?;
    Ok(())
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
    ctx.set_png_stream(TEST_PNG_001)?;
    let ihdr = ctx.get_ihdr()?;
    assert_eq!(300, ihdr.width);
    assert_eq!(300, ihdr.height);
    assert_eq!(8, ihdr.bit_depth);
    assert_eq!(4, spng::ColorType::try_from(ihdr.color_type)?.samples());
    let buffer_size = ctx.decoded_image_size(out_format)?;
    let mut data = vec![0; buffer_size];
    ctx.decode_image(&mut data, out_format, spng::DecodeFlags::empty())?;
    Ok(())
}

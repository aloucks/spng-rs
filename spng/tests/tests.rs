use spng::Decoder;
use std::io::Cursor;

#[test]
fn decode_001() -> Result<(), Box<dyn std::error::Error>> {
    static TEST_PNG: &[u8] = include_bytes!("test-001.png");

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

#[test]
fn decode_002() -> Result<(), Box<dyn std::error::Error>> {
    static TEST_PNG: &[u8] = include_bytes!("test-002.png");

    let cursor = Cursor::new(TEST_PNG);
    let decoder = Decoder::new(cursor);
    let (out_info, mut reader) = decoder.read_info()?;
    let output_buffer_size = reader.output_buffer_size();
    assert_eq!(380, out_info.width);
    assert_eq!(287, out_info.height);
    assert_eq!(8, out_info.bit_depth);
    assert_eq!(4, out_info.color_type.samples());
    assert_eq!(out_info.buffer_size, output_buffer_size);
    let mut out = vec![0; output_buffer_size];
    reader.next_frame(&mut out)?;
    Ok(())
}

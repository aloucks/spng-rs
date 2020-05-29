pub static TEST_PNG_002: &[u8] = include_bytes!("../../spng/tests/test-002.png");

pub fn reserve(buf: &mut Vec<u8>, capacity: usize) {
    let cap = buf.capacity();
    if cap < capacity {
        let additional = capacity - cap;
        buf.reserve(additional);
        unsafe {
            buf.set_len(capacity);
        }
    }
}

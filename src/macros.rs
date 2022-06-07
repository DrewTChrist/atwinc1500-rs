
/// Takes an array of bytes
/// combines it into one u32
///
/// If the size of the buffer
/// is greater than 4, the
/// remaining bytes will still
/// be shifted in
macro_rules! combine_bytes {
    ($buffer:expr) => {{
        let mut value: u32 = 0;
        for i in 0..$buffer.len() {
            value <<= 8_u32;
            value |= $buffer[i] as u32;
        }
        value
    }};
}


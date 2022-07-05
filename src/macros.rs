/// Takes an array of bytes
/// combines it into one u32
///
/// If the size of the buffer
/// is greater than 4, the
/// remaining bytes will still
/// be shifted in
#[allow(unused_macros)]
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

/// Combine a a byte array
/// into a u32 little endian
macro_rules! combine_bytes_lsb {
    ($buffer:expr) => {{
        let mut value: u32 = 0;
        for i in (0..$buffer.len()).rev() {
            value <<= 8_u32;
            value |= $buffer[i] as u32;
        }
        value
    }};
}

macro_rules! retry_while {
    ($condition:expr, retries=$num_retries:literal, $expression:expr) => {
        let mut r = $num_retries;
        while $condition && r > 0 {
            $expression;
            r -= 1;
        }
    };
}

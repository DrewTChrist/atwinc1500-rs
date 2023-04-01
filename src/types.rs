//! Public type implementations
use core::fmt;
use defmt::{write as defmt_write, Format, Formatter};

/// Firmware version of 3 bytes in the format x.x.x
#[derive(Copy, Clone)]
pub struct FirmwareVersion(pub [u8; 3]);

/// Mac address of 6 bytes in the format x:x:x:x:x:x
#[derive(Copy, Clone)]
pub struct MacAddress(pub [u8; 6]);

impl Format for FirmwareVersion {
    fn format(&self, fmt: Formatter) {
        defmt_write!(fmt, "{}.{}.{}", self.0[0], self.0[1], self.0[2]);
    }
}

impl fmt::Display for FirmwareVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.0[0], self.0[1], self.0[2])
    }
}

impl Format for MacAddress {
    fn format(&self, fmt: Formatter) {
        defmt_write!(
            fmt,
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            self.0[0],
            self.0[1],
            self.0[2],
            self.0[3],
            self.0[4],
            self.0[5]
        )
    }
}

impl fmt::Display for MacAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5]
        )
    }
}

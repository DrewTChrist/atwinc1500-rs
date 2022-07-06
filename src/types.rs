//! Public type implementations
use core::fmt;
#[cfg(target_os = "none")]
use defmt::{write as defmt_write, Format, Formatter};

/// Firmware version of 3 bytes in the format x.x.x
pub struct FirmwareVersion(pub [u8; 3]);
/// Mac address of 6 bytes in the format x:x:x:x:x:x
pub struct MacAddress(pub [u8; 6]);

#[cfg(target_os = "none")]
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

#[cfg(target_os = "none")]
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

enum SecurityType {
    /// Invalid security type
    Invalid = 0,
    /// Wi-Fi network is not secured
    Open = 1,
    /// Wi-Fi network is secured with WPA/WPA2 personal(PSK)
    WpaPsk = 2,
    /// Security type WEP (40 or 104) OPEN OR SHARED
    Wep = 3,
    /// Wi-Fi network is secured with WPA/WPA2 Enterprise.IEEE802.1x user-name/password authentication
    Sec8021x = 4,
}

enum Channel {
    Ch1 = 1,
    Ch2 = 2,
    Ch3 = 3,
    Ch4 = 4,
    Ch5 = 5,
    Ch6 = 6,
    Ch7 = 7,
    Ch8 = 8,
    Ch9 = 9,
    Ch10 = 10,
    Ch11 = 11,
    Ch12 = 12,
    Ch13 = 13,
    Ch14 = 14,
    Ch15 = 15,
    Ch16 = 16,
    ChAll = 255,
}

pub struct TcpSocket {}

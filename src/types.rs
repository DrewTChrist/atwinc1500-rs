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

pub enum SecurityType {
    /// Wi-Fi network is not secured
    Open = 1,
    /// Wi-Fi network is secured with WPA/WPA2 personal(PSK)
    WpaPsk = 2,
    /// Security type WEP (40 or 104) OPEN OR SHARED
    Wep = 3,
    /// Wi-Fi network is secured with WPA/WPA2 Enterprise.IEEE802.1x user-name/password authentication
    Sec8021x = 4,
}

pub enum Channel {
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

const MAX_SSID_LEN: usize = 33;
const MAX_PSK_LEN: usize = 65;
const MIN_PSK_LEN: usize = 9;
const USER_NAME_MAX: usize = 21;
const PASSWORD_MAX: usize = 41;
const WEP_40_KEY_STRING_SIZE: usize = 10;
const WEP_104_KEY_STRING_SIZE: usize = 26;
const WEP_KEY_MAX_INDEX: usize = 4;

struct WepSecurity {
    key_index: u8,
    key_size: u8,
    key: [u8; WEP_104_KEY_STRING_SIZE + 1],
}

struct WpaEnterpriseSecurity {
    username: [u8; USER_NAME_MAX],
    password: [u8; PASSWORD_MAX],
}

pub struct SecurityParameters {
    sec_type: SecurityType,
    wep: Option<WepSecurity>,
    wpa_psk: Option<[u8; MAX_PSK_LEN]>,
    wpa_enterprise: Option<WpaEnterpriseSecurity>,
}

impl SecurityParameters {
    fn new(
        sec_type: SecurityType,
        username: Option<&[u8]>,
        password: Option<&[u8]>,
        key_index: Option<u8>,
        key_size: Option<u8>,
        key: Option<&[u8]>,
    ) -> Self {
        let mut s = Self {
            sec_type,
            wep: None,
            wpa_psk: None,
            wpa_enterprise: None,
        };
        match s.sec_type {
            SecurityType::Open => {}
            SecurityType::WpaPsk => {
                s.wpa_psk = Some([0; MAX_PSK_LEN]);
                if let Some(pword) = password {
                    if let Some(mut psk) = s.wpa_psk {
                        psk[..pword.len()].copy_from_slice(pword);
                    }
                }
            }
            SecurityType::Wep => {
                let mut wep = WepSecurity {
                    key_index: 0,
                    key_size: 0,
                    key: [0; WEP_104_KEY_STRING_SIZE + 1],
                };
                if let Some(k_index) = key_index {
                    wep.key_index = k_index;
                }
                if let Some(k_size) = key_size {
                    wep.key_size = k_size;
                }
                if let Some(k) = key {
                    wep.key[..k.len()].copy_from_slice(k);
                }
                s.wep = Some(wep)
            }
            SecurityType::Sec8021x => {
                s.wpa_enterprise = Some(WpaEnterpriseSecurity {
                    username: [0; USER_NAME_MAX],
                    password: [0; PASSWORD_MAX],
                });
                if let Some(user) = username {
                    if let Some(ref mut wpa) = s.wpa_enterprise {
                        wpa.username[..user.len()].copy_from_slice(user);
                    }
                }
                if let Some(pword) = password {
                    if let Some(ref mut wpa) = s.wpa_enterprise {
                        wpa.password[..pword.len()].copy_from_slice(pword);
                    }
                }
            }
        }
        s
    }
}

pub struct ConnectionParameters {
    security: SecurityParameters,
    channel: Channel,
    ssid: [u8; MAX_SSID_LEN],
    save_creds: u8,
}

impl ConnectionParameters {
    fn new(
        security: SecurityParameters,
        channel: Channel,
        ssid: [u8; MAX_SSID_LEN],
        save_creds: u8,
    ) -> Self {
        Self {
            security,
            channel,
            ssid,
            save_creds,
        }
    }
}

pub struct TcpSocket {}

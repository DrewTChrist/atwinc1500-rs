//! Wifi connection items

// constants
const MAX_SSID_LEN: usize = 33;
const MAX_PSK_LEN: usize = 65;
const _MIN_PSK_LEN: usize = 9;
const USER_NAME_MAX: usize = 21;
const PASSWORD_MAX: usize = 41;
const _WEP_40_KEY_STRING_SIZE: usize = 10;
const WEP_104_KEY_STRING_SIZE: usize = 26;
const _WEP_KEY_MAX_INDEX: usize = 4;

/// This represents the type
/// of security a network uses
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

/// Wireless channels
///
/// The default channel is any
#[derive(Default)]
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
    #[default]
    Any = 255,
}

/// Security parameters for connecting
/// to a WEP protected network
pub struct WepSecurity {
    key_index: u8,
    key_size: u8,
    key: [u8; WEP_104_KEY_STRING_SIZE + 1],
}

/// Security parameters for connecting
/// to an enterprise Wpa network
pub struct WpaEnterpriseSecurity {
    username: [u8; USER_NAME_MAX],
    password: [u8; PASSWORD_MAX],
}

/// Security parameters for connecting
/// to a wifi network
pub struct SecurityParameters {
    pub sec_type: SecurityType,
    pub wep: Option<WepSecurity>,
    pub wpa_psk: Option<[u8; MAX_PSK_LEN]>,
    pub wpa_enterprise: Option<WpaEnterpriseSecurity>,
}

impl SecurityParameters {
    pub fn new(
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
                let mut psk: [u8; MAX_PSK_LEN] = [0; MAX_PSK_LEN];
                if let Some(pword) = password {
                    psk[..pword.len()].copy_from_slice(pword);
                }
                s.wpa_psk = Some(psk);
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

/// These are the parameters used
/// to connect to a wifi network
pub struct ConnectionParameters {
    pub security: SecurityParameters,
    pub channel: Channel,
    pub ssid: [u8; MAX_SSID_LEN],
    pub save_creds: u8,
}

impl ConnectionParameters {
    pub fn new(
        security: SecurityParameters,
        channel: Channel,
        ssid: &[u8],
        save_creds: u8,
    ) -> Self {
        let mut s = Self {
            security,
            channel,
            ssid: [0; MAX_SSID_LEN],
            save_creds,
        };
        s.ssid[..ssid.len()].copy_from_slice(ssid);
        s
    }
}

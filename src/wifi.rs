//! Wifi connection items

// constants
const MAX_SSID_LEN: usize = 33;
const MAX_PSK_LEN: usize = 65;
const _MIN_PSK_LEN: usize = 9;
const _USER_NAME_MAX: usize = 21;
const _PASSWORD_MAX: usize = 41;
const _WEP_40_KEY_STRING_SIZE: usize = 10;
const _WEP_104_KEY_STRING_SIZE: usize = 26;
const _WEP_KEY_MAX_INDEX: usize = 4;

/// Connection for older firmware
pub type OldConnection = [u8; 106];
/// Connection for newer firmware
pub type NewConnection = ([u8; 48], [u8; 108]);

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

pub struct ConnectionOptions {
    sec_type: SecurityType,
    save_creds: u8,
    channel: Channel,
}

pub enum ConnectionParameters {
    _Wep(),
    WpaPsk([u8; MAX_SSID_LEN], [u8; MAX_PSK_LEN], ConnectionOptions),
    _WpaEnterprise(),
}

impl ConnectionParameters {
    pub fn _wep() -> Self {
        todo!()
    }

    pub fn wpa_psk(ssid: &[u8], wpa_psk: &[u8], channel: Channel, save_creds: u8) -> Self {
        let mut ssid_arr = [0; MAX_SSID_LEN];
        let mut wpa_psk_arr = [0; MAX_PSK_LEN];
        ssid_arr[..ssid.len()].copy_from_slice(ssid);
        wpa_psk_arr[..wpa_psk.len()].copy_from_slice(wpa_psk);
        let options = ConnectionOptions {
            sec_type: SecurityType::WpaPsk,
            save_creds,
            channel,
        };
        ConnectionParameters::WpaPsk(ssid_arr, wpa_psk_arr, options)
    }

    pub fn _wpa_enterprise() -> Self {
        todo!()
    }
}

impl From<ConnectionParameters> for OldConnection {
    fn from(connection: ConnectionParameters) -> Self {
        let mut conn_header: OldConnection = [0; 106];
        match connection {
            ConnectionParameters::WpaPsk(ssid, pass, opts) => {
                conn_header[0..MAX_PSK_LEN].copy_from_slice(&pass);
                conn_header[65] = opts.sec_type as u8;
                conn_header[66] = 0;
                conn_header[67] = 0;
                conn_header[68] = opts.channel as u8;
                conn_header[69] = 0;
                conn_header[70..103].copy_from_slice(&ssid);
                conn_header[103] = opts.save_creds;
                conn_header[104] = 0;
                conn_header[105] = 0;
            }
            ConnectionParameters::_Wep() => {}
            ConnectionParameters::_WpaEnterprise() => {}
        }
        conn_header
    }
}

impl From<ConnectionParameters> for NewConnection {
    fn from(connection: ConnectionParameters) -> Self {
        let mut _conn_header: NewConnection = ([0; 48], [0; 108]);
        match connection {
            ConnectionParameters::WpaPsk(_ssid, _pass, _opts) => {}
            ConnectionParameters::_Wep() => {
                /* This is an error, WEP was deprecated for
                 * the new connection model */
            }
            ConnectionParameters::_WpaEnterprise() => {}
        }
        _conn_header
    }
}

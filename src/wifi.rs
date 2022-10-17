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

/// Connection format for older firmware
pub(crate) type OldConnection = [u8; 106];
/// Connection format for newer firmware
pub(crate) type NewConnection = ([u8; 48], [u8; 108]);

/// This represents the type
/// of security a network uses
enum SecurityType {
    /// Wi-Fi network is not secured
    Open = 1,
    /// Wi-Fi network is secured with WPA/WPA2 personal(PSK)
    WpaPsk = 2,
    /// Security type WEP (40 or 104) OPEN OR SHARED
    _Wep = 3,
    /// Wi-Fi network is secured with WPA/WPA2 Enterprise.IEEE802.1x user-name/password authentication
    _Sec8021x = 4,
}

/// Wireless RF channels
///
/// The default channel is any
#[derive(Default)]
pub enum Channel {
    /// Channel 1
    Ch1 = 1,
    /// Channel 2
    Ch2 = 2,
    /// Channel 3
    Ch3 = 3,
    /// Channel 4
    Ch4 = 4,
    /// Channel 5
    Ch5 = 5,
    /// Channel 6
    Ch6 = 6,
    /// Channel 7
    Ch7 = 7,
    /// Channel 8
    Ch8 = 8,
    /// Channel 9
    Ch9 = 9,
    /// Channel 10
    Ch10 = 10,
    /// Channel 11
    Ch11 = 11,
    /// Channel 12
    Ch12 = 12,
    /// Channel 13
    Ch13 = 13,
    /// Channel 14
    Ch14 = 14,
    #[default]
    /// Any channel (default)
    Any = 255,
}

/// Configurable options used for connecting to
/// a wireless nework
struct ConnectionOptions {
    sec_type: SecurityType,
    save_creds: u8,
    channel: Channel,
}

/// Parameters used to connect to a wireless network
enum ConnectionParameters {
    /// ConnectionParameters for an open network
    Open([u8; MAX_SSID_LEN], ConnectionOptions),
    /// ConnectionParameters for a WEP protected network
    _Wep(),
    /// ConnectionParameters for a WPA PSK protected network
    WpaPsk([u8; MAX_SSID_LEN], [u8; MAX_PSK_LEN], ConnectionOptions),
    /// ConnectionParameters for a WPA Enterprise protected network
    _WpaEnterprise(),
}

/// The Connection struct is used to give
/// the Atwinc the credentials of the station
/// to connect to
pub struct Connection {
    parameters: ConnectionParameters,
}

impl Connection {
    /// Creates a [Connection] to
    /// connect to an open wifi network
    pub fn open(ssid: &[u8], channel: Channel, save_creds: u8) -> Self {
        let mut ssid_arr = [0; MAX_SSID_LEN];
        ssid_arr[..ssid.len()].copy_from_slice(ssid);
        let options = ConnectionOptions {
            sec_type: SecurityType::Open,
            save_creds,
            channel,
        };
        Self {
            parameters: ConnectionParameters::Open(ssid_arr, options),
        }
    }

    /// Creates a [Connection] to connect
    /// to a WEP protected wifi network
    pub fn _wep() -> Self {
        todo!()
    }

    /// Creates a [Connection] to connect
    /// to a WPA PSK protected wifi network
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
        Self {
            parameters: ConnectionParameters::WpaPsk(ssid_arr, wpa_psk_arr, options),
        }
    }

    /// Creates a [Connection] to connect
    /// to a WPA Enterprise protected wifi network
    pub fn _wpa_enterprise() -> Self {
        todo!()
    }
}

impl From<Connection> for OldConnection {
    /// Easily convert a [Connection] to the old
    /// wifi connection format
    fn from(connection: Connection) -> Self {
        let mut conn_header: OldConnection = [0; 106];
        match connection.parameters {
            ConnectionParameters::Open(ssid, opts) => {
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

impl From<Connection> for NewConnection {
    /// Easily convert a [Connection] to the new
    /// wifi connection format
    fn from(connection: Connection) -> Self {
        let mut _conn_header: NewConnection = ([0; 48], [0; 108]);
        match connection.parameters {
            ConnectionParameters::Open(_ssid, _opts) => {}
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

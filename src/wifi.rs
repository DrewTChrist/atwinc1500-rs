//! Wifi connection items
use crate::types::MacAddress;

// constants
pub(crate) const MAX_SSID_LEN: usize = 33;
const MAX_PSK_LEN: usize = 65;
const _MIN_PSK_LEN: usize = 9;
const USER_NAME_MAX: usize = 21;
const PASSWORD_MAX: usize = 41;
const _WEP_40_KEY_STRING_SIZE: usize = 10;
const _WEP_104_KEY_STRING_SIZE: usize = 26;
const _WEP_KEY_MAX_INDEX: usize = 4;
const CONN_HEADER_LEN: usize = 108;

/// WifiCommand variants represent
/// valid Atwinc1500 wifi commands
/// and responses
#[repr(u8)]
#[derive(from_u8_derive::FromByte, Debug)]
pub enum WifiCommand {
    /// Request to restart the MAC layer
    ReqRestart = 1,
    /// Request to set the mac address
    ReqSetMacAddress = 2,
    /// Request the current connection rssi
    ReqCurrentRssi = 3,
    /// Response for current connection rssi
    RespCurrentRssi = 4,
    /// Request current connection info
    ReqGetConnInfo = 5,
    /// Response for current connection info
    RespConnInfo = 6,
    /// Request to set device name
    ReqSetDeviceName = 7,
    /// Request to start provision mode
    ReqStartProvisionMode = 8,
    /// Responses for provision info
    RespProvisionInfo = 9,
    /// Request to stop provision mode
    ReqStopProvisionMode = 10,
    /// Request to set the system time
    ReqSetSysTime = 11,
    /// Request to enable SNTP client
    ReqEnableSntpClient = 12,
    /// Request to disable SNTP client
    ReqDisableSntpClient = 13,
    /// Add custom element to beacon management frame
    ReqCustInfoElement = 15,
    /// Request a network scan
    ReqScan = 16,
    /// Response to network scan
    RespScanDone = 17,
    /// Request a network scan result
    ReqScanResult = 18,
    /// Response to a network scan result
    RespScanResult = 19,
    /// Request to set scan options
    ReqSetScanOption = 20,
    /// Request to set scan region
    ReqSetScanRegion = 21,
    /// Request to set the power profile
    ReqSetPowerProfile = 22,
    /// Request to set transfer power
    ReqSetTxPower = 23,
    /// Request to set battery voltage
    ReqSetBatteryVoltage = 24,
    /// Request to enable logs
    ReqSetEnableLogs = 25,
    /// Request to get system time
    ReqGetSysTime = 26,
    /// Response to get system time
    RespGetSysTime = 27,
    /// Request to send ethernet packet in bypass mode
    ReqSendEthernetPacket = 28,
    /// Response to sending ethernet packet
    RespEthernetRxPacket = 29,
    /// Request to set multicast filters
    ReqSetMacMcast = 30,
    /// Request to get prng
    ReqGetPrng = 31,
    /// Response to prng
    RespGetPrng = 32,
    /// Request a list of ssids
    ReqScanSsidList = 33,
    /// Request to set the ppa gain
    ReqSetGains = 34,
    /// Request a passive scan
    ReqPassiveScan = 35,
    /// Maximum config value
    /// Probably not used
    MaxConfigAll = 36,
    /// Request to connect to network
    ReqConnect = 40,
    /// Request to connect to default network
    ReqDefaultConnect = 41,
    /// Response to connect to network
    RespConnect = 42,
    /// Request to disconnect from network
    ReqDisconnect = 43,
    /// Response to connection state changed
    RespConStateChanged = 44,
    /// Request ps mode
    ReqSleep = 45,
    /// Request a wps scan
    ReqWpsScan = 46,
    /// Request wps start
    ReqWps = 47,
    /// Request to disable wps
    ReqDisableWps = 49,
    /// Response Ip address was obtained
    ReqDhcpConf = 50,
    /// For internal use
    RespIpConfigured = 51,
    /// Response to ip conflicts
    RespIpConflict = 52,
    /// Request to enable monitor mode
    ReqEnableMonitoring = 53,
    /// Request to disable monitor mode
    ReqDisableMonitoring = 54,
    /// Response to send wifi packet
    RespWifiRxPacket = 55,
    /// Request to send wifi packet
    ReqSendWifiPacket = 56,
    /// Request wifi listen interval
    ReqLsnInt = 57,
    /// Request the Atwinc1500 to sleep in ps mode
    ReqDoze = 58,
    /// Not a valid command or response
    Invalid,
}

/// Connection format for older firmware
pub(crate) type OldConnection = [u8; CONN_HEADER_LEN];
/// Connection format for newer firmware
pub(crate) type NewConnection = ([u8; 48], [u8; CONN_HEADER_LEN]);

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
    Sec8021x = 4,
}

/// Wireless RF channels
///
/// The default channel is any
#[derive(Default)]
#[repr(u8)]
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
    WpaEnterprise(
        [u8; MAX_SSID_LEN],
        [u8; USER_NAME_MAX],
        [u8; PASSWORD_MAX],
        ConnectionOptions,
    ),
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
    pub fn wpa_enterprise(
        ssid: &[u8],
        user: &[u8],
        password: &[u8],
        channel: Channel,
        save_creds: u8,
    ) -> Self {
        let mut ssid_arr = [0; MAX_SSID_LEN];
        let mut user_arr = [0; USER_NAME_MAX];
        let mut password_arr = [0; PASSWORD_MAX];
        ssid_arr[..ssid.len()].copy_from_slice(ssid);
        user_arr[..user.len()].copy_from_slice(user);
        password_arr[..password.len()].copy_from_slice(password);
        let options = ConnectionOptions {
            sec_type: SecurityType::Sec8021x,
            save_creds,
            channel,
        };
        Self {
            parameters: ConnectionParameters::WpaEnterprise(
                ssid_arr,
                user_arr,
                password_arr,
                options,
            ),
        }
    }
}

impl From<Connection> for OldConnection {
    /// Easily convert a [Connection] to the old
    /// wifi connection format
    fn from(connection: Connection) -> Self {
        let mut conn_header: OldConnection = [0; CONN_HEADER_LEN];
        match connection.parameters {
            ConnectionParameters::Open(ssid, opts) => {
                conn_header[65] = opts.sec_type as u8;
                conn_header[68] = opts.channel as u8;
                conn_header[70..103].copy_from_slice(&ssid);
                conn_header[103] = opts.save_creds;
            }
            ConnectionParameters::WpaPsk(ssid, pass, opts) => {
                conn_header[0..MAX_PSK_LEN].copy_from_slice(&pass);
                conn_header[65] = opts.sec_type as u8;
                conn_header[68] = opts.channel as u8;
                conn_header[70..103].copy_from_slice(&ssid);
                conn_header[103] = opts.save_creds;
            }
            ConnectionParameters::_Wep() => {}
            ConnectionParameters::WpaEnterprise(ssid, user, pass, opts) => {
                conn_header[0..USER_NAME_MAX].copy_from_slice(&user);
                conn_header[USER_NAME_MAX..USER_NAME_MAX + PASSWORD_MAX].copy_from_slice(&pass);
                conn_header[65] = opts.sec_type as u8;
                conn_header[68] = opts.channel as u8;
                conn_header[70..103].copy_from_slice(&ssid);
                conn_header[103] = opts.save_creds;
            }
        }
        conn_header
    }
}

impl From<Connection> for NewConnection {
    /// Easily convert a [Connection] to the new
    /// wifi connection format
    fn from(connection: Connection) -> Self {
        let mut _conn_header: NewConnection = ([0; 48], [0; CONN_HEADER_LEN]);
        match connection.parameters {
            ConnectionParameters::Open(_ssid, _opts) => {}
            ConnectionParameters::WpaPsk(_ssid, _pass, _opts) => {}
            ConnectionParameters::_Wep() => {
                /* This is an error, WEP was deprecated for
                 * the new connection model */
            }
            ConnectionParameters::WpaEnterprise(_ssid, _user, _pass, _opts) => {}
        }
        _conn_header
    }
}

#[repr(u8)]
pub(crate) enum StateChangeErrorCode {
    ScanFail = 1,
    JoinFail = 2,
    AuthFail = 3,
    AssocFail = 4,
    ConnectionInProgress = 5,
    Invalid,
}

impl From<u8> for StateChangeErrorCode {
    fn from(code: u8) -> Self {
        match code {
            1 => StateChangeErrorCode::ScanFail,
            2 => StateChangeErrorCode::JoinFail,
            3 => StateChangeErrorCode::AuthFail,
            4 => StateChangeErrorCode::AssocFail,
            5 => StateChangeErrorCode::ConnectionInProgress,
            _ => StateChangeErrorCode::Invalid,
        }
    }
}

#[repr(u8)]
pub(crate) enum ConnectionState {
    Connected = 0,
    Disconnected = 1,
    Undefined = 0xff,
}

impl From<u8> for ConnectionState {
    fn from(code: u8) -> Self {
        match code {
            0 => ConnectionState::Connected,
            1 => ConnectionState::Disconnected,
            0xff => ConnectionState::Undefined,
            _ => ConnectionState::Undefined,
        }
    }
}

pub(crate) struct StateChange {
    pub current_state: ConnectionState,
    pub _error_code: StateChangeErrorCode,
}

impl From<[u8; 4]> for StateChange {
    fn from(data: [u8; 4]) -> Self {
        Self {
            current_state: ConnectionState::from(data[0]),
            _error_code: StateChangeErrorCode::from(data[1]),
        }
    }
}

pub(crate) struct ScanChannel {
    /// The channel to scan for networks
    pub channel: u8,
    /// Reserved for future use
    pub reserved: u8,
    /// Passive scan time if
    /// doing a passive scan
    pub passive_scan_time: u16,
}

impl ScanChannel {
    pub fn new(channel: Channel) -> Self {
        Self {
            channel: channel as u8,
            reserved: 0,
            passive_scan_time: 0,
        }
    }
    pub fn _new_passive(channel: Channel, passive_scan_time: u16) -> Self {
        Self {
            channel: channel as u8,
            reserved: 0,
            passive_scan_time,
        }
    }
}

impl From<ScanChannel> for [u8; 4] {
    fn from(scan_channel: ScanChannel) -> [u8; 4] {
        [
            scan_channel.channel,
            scan_channel.reserved,
            (scan_channel.passive_scan_time >> 4) as u8,
            (scan_channel.passive_scan_time & 0x0f) as u8,
        ]
    }
}

/// The ScanResultCount holds the
/// number of access points that were
/// found in the scan and the state of the scan
pub(crate) struct ScanResultCount {
    /// Number of access points
    /// found in the scan
    pub num_ap: u8,
    /// Scan state returned
    /// from the Atwinc1500
    pub _scan_state: i8,
}

impl From<[u8; 4]> for ScanResultCount {
    fn from(data: [u8; 4]) -> Self {
        Self {
            num_ap: data[0],
            _scan_state: data[1] as i8,
        }
    }
}

/// The ScanResultIndex is sent to the
/// Atwinc1500 to get the ScanResult back
pub(crate) struct ScanResultIndex(pub u8);

impl From<ScanResultIndex> for [u8; 4] {
    fn from(scan_index: ScanResultIndex) -> [u8; 4] {
        [scan_index.0, 0, 0, 0]
    }
}

/// The ScanResult struct holds information about an
/// access point found in a network scan
///
/// Network scans can be initiated by calling [request_network_scan](crate::Atwinc1500::request_network_scan).
#[derive(Clone)]
pub struct ScanResult {
    /// The index of the scan result
    pub index: u8,
    /// Rssi
    pub rssi: i8,
    /// Authorization type
    pub auth_type: u8,
    /// Wifi channel
    pub channel: u8,
    /// bssid, mac address?
    pub bssid: [u8; 6],
    /// Network name
    pub ssid: [u8; MAX_SSID_LEN],
}

impl From<[u8; 44]> for ScanResult {
    fn from(data: [u8; 44]) -> Self {
        let mut bssid = [0; 6];
        bssid.copy_from_slice(&data[4..10]);
        let mut ssid = [0; MAX_SSID_LEN];
        ssid.copy_from_slice(&data[10..43]);
        Self {
            index: data[0],
            rssi: data[1] as i8,
            auth_type: data[2],
            channel: data[3],
            bssid,
            ssid,
        }
    }
}

impl defmt::Format for ScanResult {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(
            f,
            "ScanResult {{ \
                index: {}, \
                rssi: {}, \
                auth_type: {}, \
                channel: {}, \
                bssid: {}, \
                rssi: {} \
            }}",
            self.index,
            self.rssi,
            self.auth_type,
            self.channel,
            self.bssid,
            core::str::from_utf8(&self.ssid)
                .unwrap()
                .trim_matches(char::from(0)),
        );
    }
}

/// The System time returned from the Atwinc1500 SNTP client
///
/// A request for the system time can be initiated by first connecting to a
/// wireless network and then calling [request_system_time](crate::Atwinc1500::request_system_time).
#[derive(Clone, defmt::Format)]
pub struct SystemTime {
    /// Year
    pub year: u16,
    /// Month
    pub month: u8,
    /// Day
    pub day: u8,
    /// Hour
    pub hour: u8,
    /// Minute
    pub minute: u8,
    /// Second
    pub second: u8,
}

impl From<[u8; 8]> for SystemTime {
    fn from(data: [u8; 8]) -> Self {
        Self {
            year: (((data[1] as u16) << 8) | data[0] as u16),
            month: data[2],
            day: data[3],
            hour: data[4],
            minute: data[5],
            second: data[6],
        }
    }
}

/// The ConnectionInfo struct holds information returned from the
/// Atwinc1500 regarding the current wireless connection.
///
/// This information can be requested by initiating a call to
/// [request_connection_info](crate::Atwinc1500::request_connection_info).
#[derive(defmt::Format, Debug)]
pub struct ConnectionInfo {
    /// SSID of the current connection
    pub ssid: [u8; MAX_SSID_LEN],
    /// Security type of the current connection
    pub security_type: u8,
    /// Local ip address of the atwinc1500
    pub ip_address: [u8; 4],
    /// Mac address of the AP
    pub mac_address: MacAddress,
    /// Current rssi
    pub rssi: i8,
}

impl From<&[u8]> for ConnectionInfo {
    fn from(slice: &[u8]) -> Self {
        let mut ssid: [u8; MAX_SSID_LEN] = [0; MAX_SSID_LEN];
        let mut ip: [u8; 4] = [0; 4];
        let mut mac: [u8; 6] = [0; 6];
        ssid[..MAX_SSID_LEN].copy_from_slice(&slice[..MAX_SSID_LEN]);
        ip[..4].copy_from_slice(&slice[MAX_SSID_LEN + 1..MAX_SSID_LEN + 5]);
        mac[..6].copy_from_slice(&slice[MAX_SSID_LEN + 5..MAX_SSID_LEN + 11]);
        Self {
            ssid,
            security_type: slice[MAX_SSID_LEN],
            ip_address: ip,
            mac_address: MacAddress(mac),
            rssi: slice[44] as i8,
        }
    }
}

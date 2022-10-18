#[cfg(test)]
mod connection_tests {
    use atwinc1500::wifi::{Channel, Connection};

    #[test]
    fn open_ssid() {
        let ssid = &"thisismyssid".as_bytes();
        let start: usize = 70;
        let end: usize = start + ssid.len();
        let connection = Connection::open(ssid, Channel::default(), 0);
        let arr: [u8; 108] = connection.into();
        assert_eq!(&&arr[start..end], ssid);
        assert_eq!(&&arr[65], &&1); // 1 = open network type
    }

    #[test]
    fn wpa_psk_ssid() {
        let ssid = &"thisismyssid".as_bytes();
        let pass = &"thisismypass".as_bytes();
        let start: usize = 70;
        let end: usize = start + ssid.len();
        let connection = Connection::wpa_psk(ssid, pass, Channel::default(), 0);
        let arr: [u8; 108] = connection.into();
        assert_eq!(&&arr[start..end], ssid);
        assert_eq!(&&arr[65], &&2); // 2 = wpa psk network type
    }

    #[test]
    fn wpa_enterprise_ssid() {
        let ssid = &"thisismyssid".as_bytes();
        let pass = &"thisismypassword".as_bytes();
        let user = &"thisismyusername".as_bytes();
        let start: usize = 70;
        let end: usize = start + ssid.len();
        let connection = Connection::wpa_enterprise(ssid, user, pass, Channel::default(), 0);
        let arr: [u8; 108] = connection.into();
        assert_eq!(&&arr[start..end], ssid);
        assert_eq!(&&arr[65], &&4); // 4 = wpa enterprise network type
    }
}

use crate::error::Error;
use crate::registers;
use crate::spi::SpiBus;
use crate::types::MacAddress;
use crate::wifi::{ConnectionState, StateChange, MAX_SSID_LEN};
use crate::{Mode, State, Status};
use embedded_hal::blocking::spi::Transfer;
use embedded_hal::digital::v2::OutputPin;

pub mod group_ids {
    pub const _MAIN: u8 = 0;
    pub const WIFI: u8 = 1;
    pub const _IP: u8 = 2;
    pub const _HIF: u8 = 3;
}

pub mod commands {
    pub mod main {}
    pub mod wifi {
        // station mode commands
        pub const REQ_CONNECT: u8 = 40;
        pub const REQ_DEFAULT_CONNECT: u8 = 41;
        pub const _RESP_CONNECT: u8 = 42;
        pub const REQ_DISCONNECT: u8 = 43;
        pub const RESP_CON_STATE_CHANGED: u8 = 44;
        pub const _REQ_SLEEP: u8 = 45;
        pub const _REQ_WPS_SCAN: u8 = 46;
        pub const _REQ_WPS: u8 = 47;
        pub const _REQ_DISABLE_WPS: u8 = 49;
        pub const _REQ_DHCP_CONF: u8 = 50;
        pub const _RESP_IP_CONFIGURED: u8 = 51;
        pub const _RESP_IP_CONFLICT: u8 = 52;
        pub const _REQ_ENABLE_MONITORING: u8 = 53;
        pub const _REQ_DISABLE_MONITORING: u8 = 54;
        pub const _RESP_WIFI_RX_PACKET: u8 = 55;
        pub const _REQ_SEND_WIFI_PACKET: u8 = 56;
        pub const _REQ_LSN_INT: u8 = 57;
        pub const _REQ_DOZE: u8 = 58;

        // configuration commands
        pub const _REQ_RESTART: u8 = 1;
        pub const _REQ_SET_MAC_ADDRESS: u8 = 2;
        pub const _REQ_CURRENT_RSSI: u8 = 3;
        pub const _RESP_CURRENT_RSSI: u8 = 4;
        pub const _REQ_GET_CONN_INFO: u8 = 5;
        pub const RESP_CONN_INFO: u8 = 6;
        pub const _REQ_SET_DEVICE_NAME: u8 = 7;
        pub const _REQ_START_PROVISION_MODE: u8 = 8;
        pub const _RESP_PROVISION_INFO: u8 = 9;
        pub const _REQ_STOP_PROVISION_MODE: u8 = 10;
        pub const _REQ_SET_SYS_TIME: u8 = 11;
        pub const _REQ_ENABLE_SNTP_CLIENT: u8 = 12;
        pub const _REQ_DISABLE_SNTP_CLIENT: u8 = 13;
        pub const _REQ_CUST_INFO_ELEMENT: u8 = 15;
        pub const _REQ_SCAN: u8 = 16;
        pub const _RESP_SCAN_DONE: u8 = 17;
        pub const _REQ_SCAN_RESULT: u8 = 18;
        pub const _RESP_SCAN_RESULT: u8 = 19;
        pub const _REQ_SET_SCAN_OPTION: u8 = 20;
        pub const _REQ_SET_SCAN_REGION: u8 = 21;
        pub const _REQ_SET_POWER_PROFILE: u8 = 22;
        pub const _REQ_SET_TX_POWER: u8 = 23;
        pub const _REQ_SET_BATTERY_VOLTAGE: u8 = 24;
        pub const _REQ_SET_ENABLE_LOGS: u8 = 25;
        pub const _REQ_GET_SYS_TIME: u8 = 26;
        pub const _RESP_GET_SYS_TIME: u8 = 27;
        pub const _REQ_SEND_ETHERNET_PACKET: u8 = 28;
        pub const _RESP_ETHERNET_RX_PACKET: u8 = 29;
        pub const _REQ_SET_MAC_MCAST: u8 = 30;
        pub const _REQ_GET_PRNG: u8 = 31;
        pub const _RESP_GET_PRNG: u8 = 32;
        pub const _REQ_SCAN_SSID_LIST: u8 = 33;
        pub const _REQ_SET_GAINS: u8 = 34;
        pub const _REQ_PASSIVE_SCAN: u8 = 35;
        pub const _MAX_CONFIG_AL: u8 = 36;
    }
    pub mod ip {}
    pub mod hif {}
}

const HIF_HEADER_SIZE: usize = 8;

#[derive(Copy, Clone)]
pub struct HifHeader {
    pub gid: u8,
    pub op: u8,
    pub length: u16,
}

impl HifHeader {
    /// Creates a new HifHeader automatically adding
    /// the length of itself
    pub fn new(gid: u8, op: u8, length: u16) -> Self {
        HifHeader {
            gid,
            op,
            length: length + HIF_HEADER_SIZE as u16,
        }
    }
}

impl From<HifHeader> for [u8; HIF_HEADER_SIZE] {
    /// Converts an HifHeader into an array to be sent
    /// to the Atwinc1500
    fn from(header: HifHeader) -> [u8; HIF_HEADER_SIZE] {
        [
            header.gid,
            header.op,
            header.length as u8,
            (header.length >> 8) as u8,
            0,
            0,
            0,
            0,
        ]
    }
}

impl From<HifHeader> for u32 {
    /// Converts an HifHeader to a u32
    /// to be written to an Atwinc1500 register
    fn from(header: HifHeader) -> u32 {
        combine_bytes!([
            (header.length >> 8) as u8,
            header.length as u8,
            header.op,
            header.gid,
        ])
    }
}

impl From<[u8; 4]> for HifHeader {
    /// Converts an array received from the Atwinc1500
    /// into an HifHeader
    fn from(array: [u8; 4]) -> Self {
        HifHeader {
            gid: array[0],
            op: array[1],
            length: ((array[2] as u16) << 8) | array[3] as u16,
        }
    }
}

/// Connection Information returned
/// from the Atwinc1500 after wifi callback
struct ConnectionInfo {
    _ssid: [u8; MAX_SSID_LEN],
    _security_type: u8,
    _ip_address: [u8; 4],
    _mac_address: MacAddress,
    _rssi: i8,
    _padding: [u8; 3],
}

impl From<&[u8]> for ConnectionInfo {
    fn from(slice: &[u8]) -> Self {
        let mut ssid: [u8; MAX_SSID_LEN] = [0; MAX_SSID_LEN];
        let mut ip: [u8; 4] = [0; 4];
        let mut mac: [u8; 6] = [0; 6];
        ssid[..MAX_SSID_LEN].copy_from_slice(&slice[..MAX_SSID_LEN]);
        ip[..4].copy_from_slice(&slice[MAX_SSID_LEN + 1..MAX_SSID_LEN + 5]);
        mac[..6].copy_from_slice(&slice[MAX_SSID_LEN + 6..MAX_SSID_LEN + 12]);
        Self {
            _ssid: ssid,
            _security_type: slice[MAX_SSID_LEN],
            _ip_address: ip,
            _mac_address: MacAddress(mac),
            _rssi: (!slice[MAX_SSID_LEN + 13] + 1) as i8,
            _padding: [0; 3],
        }
    }
}

/// Empty struct used to represent the Host Interface layer.
/// The host interface layer abstracts away all the low level
/// calls to the spi bus and provides a higher level api to work with.
pub(crate) struct HostInterface;

impl HostInterface {
    /// Creates a new HostInterface struct
    pub fn new() -> Self {
        Self {}
    }

    /// This method wakes the chip from sleep mode using clockless register access
    pub fn _chip_wake<SPI, O>(&mut self, spi_bus: &mut SpiBus<SPI, O>) -> Result<(), Error>
    where
        SPI: Transfer<u8>,
        O: OutputPin,
    {
        let mut trials: u32 = 0;
        let mut register_val: u32;
        let mut clock_status_val: u32;
        const _WAKEUP_TRIALS_TIMEOUT: u8 = 4;
        register_val = spi_bus.read_register(registers::HOST_CORT_COMM)?;
        if (register_val & 0x1) == 0 {
            // USE bit 0 to indicate host wakeup
            spi_bus.write_register(registers::HOST_CORT_COMM, register_val | 0x1)?;
        }
        register_val = spi_bus.read_register(registers::WAKE_CLK_REG)?;
        // Set bit 1
        if (register_val & 0x2) == 0 {
            spi_bus.write_register(registers::WAKE_CLK_REG, register_val | 0x2)?;
        }
        loop {
            clock_status_val = spi_bus.read_register(registers::CLOCKS_EN_REG)?;
            if (clock_status_val & 0x2) != 0 {
                break;
            }
            // sleep here?
            trials += 1;
            if trials > _WAKEUP_TRIALS_TIMEOUT as u32 {
                // error waking chip
                break;
            }
        }
        Ok(())
    }

    /// This method enables sleep mode for the chip
    pub fn _chip_sleep<SPI, O>(&mut self, spi_bus: &mut SpiBus<SPI, O>) -> Result<(), Error>
    where
        SPI: Transfer<u8>,
        O: OutputPin,
    {
        let mut register_val: u32;
        loop {
            register_val = spi_bus.read_register(registers::CORT_HOST_COMM)?;
            if (register_val & 0x1) == 0 {
                break;
            }
        }
        // Clear bit 1
        register_val = spi_bus.read_register(registers::WAKE_CLK_REG)?;
        if (register_val & 0x2) != 0 {
            register_val &= !0x2;
            spi_bus.write_register(registers::WAKE_CLK_REG, register_val)?;
        }
        register_val = spi_bus.read_register(registers::HOST_CORT_COMM)?;
        if (register_val & 0x1) != 0 {
            register_val &= !0x1;
            spi_bus.write_register(registers::HOST_CORT_COMM, register_val)?;
        }
        Ok(())
    }

    /// This method is the host interface interrupt service routine
    pub fn isr<SPI, O>(
        &mut self,
        spi_bus: &mut SpiBus<SPI, O>,
        state: &mut State,
    ) -> Result<(), Error>
    where
        SPI: Transfer<u8>,
        O: OutputPin,
    {
        let mut reg_value = spi_bus.read_register(registers::WIFI_HOST_RCV_CTRL_0)?;
        if reg_value & 0x1 != 0 {
            reg_value &= !0x00000001;
            spi_bus.write_register(registers::WIFI_HOST_RCV_CTRL_0, reg_value)?;
            let size: u16 = ((reg_value >> 2) & 0xfff) as u16;
            if size > 0 {
                let address: u32 = spi_bus.read_register(registers::WIFI_HOST_RCV_CTRL_1)?;
                let mut header_buf: [u8; 4] = [0; 4];
                let header_buf_len = header_buf.len() as u32;
                spi_bus.read_data(&mut header_buf, address, header_buf_len)?;
                let header = HifHeader::from(header_buf);
                match header.gid {
                    group_ids::WIFI => self.wifi_callback(
                        spi_bus,
                        header.op,
                        header.length - HIF_HEADER_SIZE as u16,
                        address + HIF_HEADER_SIZE as u32,
                        state,
                    )?,
                    group_ids::_IP => self._ip_callback(
                        spi_bus,
                        header.op,
                        header.length - HIF_HEADER_SIZE as u16,
                        address + HIF_HEADER_SIZE as u32,
                        state,
                    )?,
                    _ => { /* Invalid group id */ }
                }
            }
            self.finish_reception(spi_bus)?;
        }
        Ok(())
    }

    /// This method receives data read from the chip
    pub fn receive<SPI, O>(
        &mut self,
        spi_bus: &mut SpiBus<SPI, O>,
        address: u32,
        buffer: &mut [u8],
    ) -> Result<(), Error>
    where
        SPI: Transfer<u8>,
        O: OutputPin,
    {
        spi_bus.read_data(buffer, address, buffer.len() as u32)?;
        Ok(())
    }

    /// Lets the atwinc1500 know we're done receiving data
    fn finish_reception<SPI, O>(&mut self, spi_bus: &mut SpiBus<SPI, O>) -> Result<(), Error>
    where
        SPI: Transfer<u8>,
        O: OutputPin,
    {
        let value: u32 = spi_bus.read_register(registers::WIFI_HOST_RCV_CTRL_0)?;
        spi_bus.write_register(registers::WIFI_HOST_RCV_CTRL_0, value | 2)?;
        Ok(())
    }

    /// This method sends data to the chip
    pub fn send<SPI, O>(
        &mut self,
        spi_bus: &mut SpiBus<SPI, O>,
        header: HifHeader,
        data_buffer: &mut [u8],
        ctrl_buffer: &mut [u8],
    ) -> Result<(), Error>
    where
        SPI: Transfer<u8>,
        O: OutputPin,
    {
        let offset: u32 = data_buffer.len() as u32;
        let mut header_buf: [u8; HIF_HEADER_SIZE] = header.into();
        let hif: u32 = header.into();
        spi_bus.write_register(registers::NMI_STATE_REG, hif)?;
        spi_bus.write_register(registers::WIFI_HOST_RCV_CTRL_2, 2)?;
        let mut reg_value = spi_bus.read_register(registers::WIFI_HOST_RCV_CTRL_2)?;
        retry_while!(reg_value & 2 != 0, retries = 100, {
            reg_value = spi_bus.read_register(registers::WIFI_HOST_RCV_CTRL_2)?;
            // may need a delay here
        });
        let address: u32 = spi_bus.read_register(registers::WIFI_HOST_RCV_CTRL_4)?;
        spi_bus.write_data(&mut header_buf, address, HIF_HEADER_SIZE as u32)?;
        if !data_buffer.is_empty() {
            spi_bus.write_data(
                data_buffer,
                address + HIF_HEADER_SIZE as u32,
                data_buffer.len() as u32,
            )?;
        }
        if !ctrl_buffer.is_empty() {
            spi_bus.write_data(
                ctrl_buffer,
                address + HIF_HEADER_SIZE as u32 + offset,
                ctrl_buffer.len() as u32,
            )?;
        }
        spi_bus.write_register(registers::WIFI_HOST_RCV_CTRL_3, (address << 2) | 2)?;
        Ok(())
    }

    /// This method sets the chip sleep mode
    pub fn _set_sleep_mode<SPI, O>(&mut self, _spi_bus: &mut SpiBus<SPI, O>) -> Result<(), Error>
    where
        SPI: Transfer<u8>,
        O: OutputPin,
    {
        todo!()
    }

    /// This method returns the chip sleep mode
    pub fn _get_sleep_mode<SPI, O>(&mut self, _spi_bus: &mut SpiBus<SPI, O>) -> Result<(), Error>
    where
        SPI: Transfer<u8>,
        O: OutputPin,
    {
        todo!()
    }

    pub fn wifi_callback<SPI, O>(
        &mut self,
        spi_bus: &mut SpiBus<SPI, O>,
        opcode: u8,
        _data_size: u16,
        address: u32,
        state: &mut State,
    ) -> Result<(), Error>
    where
        SPI: Transfer<u8>,
        O: OutputPin,
    {
        match opcode {
            commands::wifi::RESP_CON_STATE_CHANGED => {
                let mut data_buf: [u8; 4] = [0; 4];
                self.receive(spi_bus, address, &mut data_buf)?;
                let state_change = StateChange::from(data_buf);
                match state_change.current_state {
                    ConnectionState::Connected => match state.mode {
                        Mode::Station => {
                            state.set_status(Status::Connected);
                        }
                        Mode::_Ap => {
                            state.set_status(Status::ApConnected);
                        }
                        _ => {}
                    },
                    ConnectionState::Disconnected => match state.mode {
                        Mode::Station => {
                            state.set_status(Status::Disconnected);
                        }
                        Mode::_Ap => {
                            state.set_status(Status::ApListening);
                        }
                        _ => {}
                    },
                    ConnectionState::Undefined => {}
                }
            }
            commands::wifi::_RESP_GET_SYS_TIME => {}
            commands::wifi::RESP_CONN_INFO => {
                let mut data_buf: [u8; 48] = [0; 48];
                self.receive(spi_bus, address, &mut data_buf)?;
                let _connection_info = ConnectionInfo::from(data_buf.as_slice());
            }
            commands::wifi::_REQ_DHCP_CONF => {}
            commands::wifi::_REQ_WPS => {}
            commands::wifi::_RESP_IP_CONFLICT => {}
            commands::wifi::_RESP_SCAN_DONE => {}
            commands::wifi::_RESP_SCAN_RESULT => {}
            commands::wifi::_RESP_CURRENT_RSSI => {}
            _ => {}
        }
        Ok(())
    }

    pub fn _ip_callback<SPI, O>(
        &mut self,
        _spi_bus: &mut SpiBus<SPI, O>,
        _opcode: u8,
        _data_size: u16,
        _address: u32,
        _state: &mut State,
    ) -> Result<(), Error>
    where
        SPI: Transfer<u8>,
        O: OutputPin,
    {
        todo!()
    }
}

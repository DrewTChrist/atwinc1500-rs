use crate::error::{Error, HifError};
use crate::registers;
use crate::socket::SocketCommand;
use crate::spi::SpiBus;
use crate::wifi::{
    ConnectionInfo, ConnectionState, ScanResult, ScanResultCount, StateChange, SystemTime,
};
use crate::{Mode, State, Status};
use embedded_hal::blocking::spi::Transfer;
use embedded_hal::digital::v2::OutputPin;

pub mod group_ids {
    pub const _MAIN: u8 = 0;
    pub const WIFI: u8 = 1;
    pub const IP: u8 = 2;
    pub const _HIF: u8 = 3;
}

#[repr(u8)]
#[derive(from_u8_derive::FromByte)]
pub enum WifiCommand {
    ReqRestart = 1,
    ReqSetMacAddress = 2,
    ReqCurrentRssi = 3,
    RespCurrentRssi = 4,
    ReqGetConnInfo = 5,
    RespConnInfo = 6,
    ReqSetDeviceName = 7,
    ReqStartProvisionMode = 8,
    RespProvisionInfo = 9,
    ReqStopProvisionMode = 10,
    ReqSetSysTime = 11,
    ReqEnableSntpClient = 12,
    ReqDisableSntpClient = 13,
    ReqCustInfoElement = 15,
    ReqScan = 16,
    RespScanDone = 17,
    ReqScanResult = 18,
    RespScanResult = 19,
    ReqSetScanOption = 20,
    ReqSetScanRegion = 21,
    ReqSetPowerProfile = 22,
    ReqSetTxPower = 23,
    ReqSetBatteryVoltage = 24,
    ReqSetEnableLogs = 25,
    ReqGetSysTime = 26,
    RespGetSysTime = 27,
    ReqSendEthernetPacket = 28,
    RespEthernetRxPacket = 29,
    ReqSetMacMcast = 30,
    ReqGetPrng = 31,
    RespGetPrng = 32,
    ReqScanSsidList = 33,
    ReqSetGains = 34,
    ReqPassiveScan = 35,
    RaxConfigAl = 36,
    ReqConnect = 40,
    ReqDefaultConnect = 41,
    RespConnect = 42,
    ReqDisconnect = 43,
    RespConStateChanged = 44,
    ReqSleep = 45,
    ReqWpsScan = 46,
    ReqWps = 47,
    ReqDisableWps = 49,
    ReqDhcpConf = 50,
    RespIpConfigured = 51,
    RespIpConflict = 52,
    ReqEnableMonitoring = 53,
    ReqDisableMonitoring = 54,
    RespWifiRxPacket = 55,
    ReqSendWifiPacket = 56,
    ReqLsnInt = 57,
    ReqDoze = 58,
    Invalid,
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

struct HifContext {
    read_addr: u32,
    read_size: u32,
    read_done: bool,
}

impl HifContext {
    fn default() -> Self {
        Self {
            read_addr: 0,
            read_size: 0,
            read_done: true,
        }
    }
}

pub enum Command {
    WifiCommand(WifiCommand),
    SocketCommand(SocketCommand),
}

impl From<WifiCommand> for Command {
    fn from(other: WifiCommand) -> Self {
        Self::WifiCommand(other)
    }
}

impl From<SocketCommand> for Command {
    fn from(other: SocketCommand) -> Self {
        Self::SocketCommand(other)
    }
}

/// Empty struct used to represent the Host Interface layer.
/// The host interface layer abstracts away all the low level
/// calls to the spi bus and provides a higher level api to work with.
pub(crate) struct HostInterface {
    ctx: HifContext,
}

impl HostInterface {
    /// Creates a new HostInterface struct
    pub fn new() -> Self {
        Self {
            ctx: HifContext::default(),
        }
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
    ) -> Result<Option<Command>, Error>
    where
        SPI: Transfer<u8>,
        O: OutputPin,
    {
        let mut command = None;
        let mut reg_value = spi_bus.read_register(registers::WIFI_HOST_RCV_CTRL_0)?;
        if reg_value & 0x1 != 0 {
            reg_value &= !0x00000001;
            spi_bus.write_register(registers::WIFI_HOST_RCV_CTRL_0, reg_value)?;
            self.ctx.read_done = false;
            let size: u32 = (reg_value >> 2) & 0xfff;
            if size > 0 {
                let address: u32 = spi_bus.read_register(registers::WIFI_HOST_RCV_CTRL_1)?;
                self.ctx.read_addr = address;
                self.ctx.read_size = size;
                let mut header_buf: [u8; 4] = [0; 4];
                let header_buf_len = header_buf.len() as u32;
                spi_bus.read_data(&mut header_buf, address, header_buf_len)?;
                let header = HifHeader::from(header_buf);
                match header.gid {
                    group_ids::WIFI => {
                        self.wifi_callback(
                            spi_bus,
                            WifiCommand::from(header.op),
                            header.length - HIF_HEADER_SIZE as u16,
                            address + HIF_HEADER_SIZE as u32,
                            state,
                        )?;
                        command = Some(Command::from(WifiCommand::from(header.op)));
                    }
                    group_ids::IP => {
                        self.ip_callback(
                            spi_bus,
                            SocketCommand::from(header.op),
                            header.length - HIF_HEADER_SIZE as u16,
                            address + HIF_HEADER_SIZE as u32,
                            state,
                        )?;
                        command = Some(Command::from(SocketCommand::from(header.op)));
                    }
                    _ => { /* Invalid group id */ }
                }
            }
            if !self.ctx.read_done {
                self.finish_reception(spi_bus)?;
            }
        }
        Ok(command)
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
        if buffer.len() as u32 > self.ctx.read_size {
            return Err(HifError::SizeMismatch(buffer.len(), self.ctx.read_size as usize).into());
        }

        spi_bus.read_data(buffer, address, buffer.len() as u32)?;

        if (self.ctx.read_addr + self.ctx.read_size) - (address + buffer.len() as u32) == 0 {
            self.finish_reception(spi_bus)?;
        }
        Ok(())
    }

    /// Lets the atwinc1500 know we're done receiving data
    fn finish_reception<SPI, O>(&mut self, spi_bus: &mut SpiBus<SPI, O>) -> Result<(), Error>
    where
        SPI: Transfer<u8>,
        O: OutputPin,
    {
        self.ctx.read_done = true;
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
        opcode: WifiCommand,
        _data_size: u16,
        address: u32,
        state: &mut State,
    ) -> Result<(), Error>
    where
        SPI: Transfer<u8>,
        O: OutputPin,
    {
        match opcode {
            WifiCommand::RespConStateChanged => {
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
            WifiCommand::RespGetSysTime => {
                let mut data_buf: [u8; 8] = [0; 8];
                self.receive(spi_bus, address, &mut data_buf)?;
                let system_time = SystemTime::from(data_buf);
                if system_time.year > 0 {
                    state.system_time = Some(system_time);
                }
                // may need to return an error here
            }
            WifiCommand::RespConnInfo => {
                let mut data_buf: [u8; 48] = [0; 48];
                self.receive(spi_bus, address, &mut data_buf)?;
                state.connection_info = Some(ConnectionInfo::from(data_buf.as_slice()));
            }
            WifiCommand::ReqDhcpConf => {}
            WifiCommand::ReqWps => {}
            WifiCommand::RespIpConflict => {}
            WifiCommand::RespScanDone => {
                let mut data_buf: [u8; 4] = [0; 4];
                self.receive(spi_bus, address, &mut data_buf)?;
                let scan_count = ScanResultCount::from(data_buf);
                state.num_ap = scan_count.num_ap;
                state.scan_in_progress = false;
                // TODO: Handle potential scan_count.scan_state error
            }
            WifiCommand::RespScanResult => {
                let mut data_buf: [u8; 44] = [0; 44];
                self.receive(spi_bus, address, &mut data_buf)?;
                let result = ScanResult::from(data_buf);
                state.scan_result = Some(result);
            }
            WifiCommand::RespCurrentRssi => {}
            _ => {}
        }
        Ok(())
    }

    pub fn ip_callback<SPI, O>(
        &mut self,
        _spi_bus: &mut SpiBus<SPI, O>,
        opcode: SocketCommand,
        _data_size: u16,
        _address: u32,
        _state: &mut State,
    ) -> Result<(), Error>
    where
        SPI: Transfer<u8>,
        O: OutputPin,
    {
        match opcode {
            SocketCommand::Bind | SocketCommand::SslBind => {}
            SocketCommand::Listen => {}
            SocketCommand::Accept => {}
            SocketCommand::Connect | SocketCommand::SslConnect => {}
            SocketCommand::DnsResolve => {}
            SocketCommand::Recv | SocketCommand::Recvfrom | SocketCommand::SslRecv => {}
            SocketCommand::Send | SocketCommand::Sendto | SocketCommand::SslSend => {}
            SocketCommand::Ping => {}
            _ => {}
        }
        Ok(())
    }
}

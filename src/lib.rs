//! This is a driver for the Atwinc1500 network controller written in pure Rust. The
//! primary targets for this driver are boards like the [Adafruit Feather M0 Wifi](https://adafruit.com/product/3010)
//! or the [Adafruit Atwinc1500 Breakout](https://adafruit.com/product/2999).
//!
//! ## Examples
//! Due to each HAL working slightly different, the examples here will be brief. There
//! are several working examples in [this repository](https://github.com/drewtchrist/atwinc1500-rs-examples)
//! with different host targets. These examples show how to set up the
//! [handle_events](Atwinc1500::handle_events) method inside of an interrupt.
//!
//! ## Callbacks
//! The Atwinc1500 as a client device is able to let the host know when it is ready to return a
//! response. The Atwinc1500 pulls the interrupt pin low notifying the host device. The methods
//! prefixed by `request` do not receive an immediate response. The response must be collected by
//! calling [handle_events](Atwinc1500::handle_events). This is best done by creating an interrupt
//! that triggers when the irq line between the host and the Atwinc1500 is pulled low. The methods
//! prefixed by `get` do not require a callback or are meant to collect the response after a
//! `request`.
#![no_std]
#![warn(missing_docs)]

extern crate from_u8_derive;
#[macro_use]
mod macros;
mod crc;
pub mod error;
pub mod gpio;
mod hif;
#[doc(hidden)]
pub mod registers;
#[doc(hidden)]
pub mod socket;
#[doc(hidden)]
pub mod spi;
pub mod types;
pub mod wifi;

use embedded_hal::blocking::{delay::DelayMs, spi::Transfer};
use embedded_hal::digital::v2::OutputPin;
use embedded_nal::{SocketAddr, TcpClientStack, TcpFullStack};

use error::{Error, ScanError};
use gpio::{AtwincGpio, GpioDirection, GpioValue};
use hif::{group_ids, HifHeader, HostInterface, WifiCommand};
use socket::TcpSocket;
use spi::SpiBus;
use types::{FirmwareVersion, MacAddress};
use wifi::{
    Channel, Connection, ConnectionInfo, OldConnection, ScanChannel, ScanResult, ScanResultIndex,
    SystemTime,
};

/// Connection status of the Atwinc1500
#[derive(Default, Eq, PartialEq, Debug, defmt::Format)]
pub enum Status {
    /// Atwinc1500 is idle
    #[default]
    Idle,
    /// SSID not available
    NoSsidAvail,
    /// Scan is complete
    ScanComplete,
    /// Atwinc1500 is connected to a network
    Connected,
    /// Connection attempt failed
    ConnectionFailed,
    /// Atwinc1500 lost connection
    ConnectionLost,
    /// Atwinc1500 is disconnected
    Disconnected,
    /// Access point mode listening
    ApListening,
    /// Access point mode connected
    ApConnected,
    /// Access point mode failed
    ApFailed,
    /// Provisioning mode
    Provisioning,
    /// Provisioning mode failed
    ProvisioningFailed,
}

#[derive(Default)]
enum Mode {
    _Reset,
    #[default]
    Station,
    _Provisioning,
    _Ap,
}

struct State {
    firmware_version: Option<FirmwareVersion>,
    mac_address: Option<MacAddress>,
    status: Status,
    mode: Mode,
    _dhcp: bool,
    connection_info: Option<ConnectionInfo>,
    scan_in_progress: bool,
    num_ap: u8,
    scan_result: Option<ScanResult>,
    system_time: Option<SystemTime>,
}

impl State {
    fn default() -> Self {
        Self {
            firmware_version: None,
            mac_address: None,
            mode: Mode::default(),
            status: Status::default(),
            _dhcp: true,
            connection_info: None,
            scan_in_progress: false,
            num_ap: 0,
            scan_result: None,
            system_time: None,
        }
    }

    fn set_firmware_version(&mut self, version: FirmwareVersion) {
        self.firmware_version = Some(version);
    }

    fn set_mac_address(&mut self, mac: MacAddress) {
        self.mac_address = Some(mac);
    }

    fn set_status(&mut self, status: Status) {
        self.status = status;
    }

    fn _set_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }

    fn _set_dhcp(&mut self, dhcp: bool) {
        self._dhcp = dhcp;
    }
}

/// Atwin1500 driver struct
pub struct Atwinc1500<SPI, D, O>
where
    SPI: Transfer<u8>,
    D: DelayMs<u32>,
    O: OutputPin,
{
    delay: D,
    spi_bus: SpiBus<SPI, O>,
    hif: HostInterface,
    reset: O,
    crc: bool,
    state: State,
}

/// Atwinc1500 struct implementation containing non embedded-nal
/// public methods
impl<SPI, D, O> Atwinc1500<SPI, D, O>
where
    SPI: Transfer<u8>,
    D: DelayMs<u32>,
    O: OutputPin,
{
    /// Returns an Atwin1500 struct
    ///
    /// # Arguments
    ///
    /// * `spi` - An spi struct implementing traits from embedded-hal
    ///
    /// * `delay` - A delay implementing Delay from embedded-hal
    ///
    /// * `cs` - An OutputPin for the chip select
    ///
    /// * `reset` - An OutputPin for chip reset
    ///
    /// * `crc` - Turn on CRC in transactions
    ///
    pub fn new(spi: SPI, delay: D, cs: O, reset: O, crc: bool) -> Self {
        Self {
            delay,
            spi_bus: SpiBus::new(spi, cs, crc),
            hif: HostInterface::new(),
            reset,
            crc,
            state: State::default(),
        }
    }

    /// This needs to be called after creating the Atwinc1500 struct
    ///
    /// Initializes the driver by:
    /// * Initializing pins between devices
    /// * Disables crc if needed
    /// * Waits for efuse ready
    /// * Waits for boot rom ready
    /// * Writes driver version and configuration
    /// * Enables chip interrupt
    pub fn initialize(&mut self) -> Result<(), Error> {
        const FINISH_BOOT_VAL: u32 = 0x10add09e;
        const DRIVER_VER_INFO: u32 = 0x13521330;
        const CONF_VAL: u32 = 0x102;
        const START_FIRMWARE: u32 = 0xef522f61;
        const FINISH_INIT_VAL: u32 = 0x02532636;
        self.init_pins()?;
        self.disable_crc()?;
        let mut efuse_value: u32 = 0;
        retry_while!((efuse_value & 0x80000000) == 0, retries = 10, {
            efuse_value = self.spi_bus.read_register(registers::EFUSE_REG)?;
            self.delay.delay_ms(1000);
        });
        let wait: u32 = self
            .spi_bus
            .read_register(registers::M2M_WAIT_FOR_HOST_REG)?;
        if (wait & 1) == 0 {
            let mut bootrom: u32 = 0;
            retry_while!(bootrom != FINISH_BOOT_VAL, retries = 3, {
                bootrom = self.spi_bus.read_register(registers::BOOTROM_REG)?;
                self.delay.delay_ms(1000);
            });
        }
        self.spi_bus
            .write_register(registers::NMI_STATE_REG, DRIVER_VER_INFO)?;
        self.spi_bus
            .write_register(registers::rNMI_GP_REG_1, CONF_VAL)?;
        self.spi_bus
            .write_register(registers::BOOTROM_REG, START_FIRMWARE)?;
        let mut state: u32 = 0;
        retry_while!(state != FINISH_INIT_VAL, retries = 20, {
            state = self.spi_bus.read_register(registers::NMI_STATE_REG)?;
            self.delay.delay_ms(1000);
        });
        self.spi_bus.write_register(registers::NMI_STATE_REG, 0)?;
        self.enable_chip_interrupt()?;
        self.get_firmware_version()?;
        self.get_mac_address()?;
        Ok(())
    }

    /// Pulls the chip select and wake pins high
    /// Then pulses (low/high) the reset pin with
    /// a delay
    fn init_pins(&mut self) -> Result<(), Error> {
        self.spi_bus.init_cs()?;
        if self.reset.set_low().is_err() {
            return Err(Error::PinStateError);
        }
        self.delay.delay_ms(1000);
        if self.reset.set_high().is_err() {
            return Err(Error::PinStateError);
        }
        self.delay.delay_ms(1000);
        Ok(())
    }

    /// Disables crc if self.crc is false
    fn disable_crc(&mut self) -> Result<(), Error> {
        if !self.crc {
            self.spi_bus
                .write_register(registers::NMI_SPI_PROTOCOL_CONFIG, 0x52)?;
            self.spi_bus.crc_disabled()?;
        }
        Ok(())
    }

    fn enable_chip_interrupt(&mut self) -> Result<(), Error> {
        let mux: u32 = self.spi_bus.read_register(registers::NMI_PIN_MUX_0)?;
        self.spi_bus
            .write_register(registers::NMI_PIN_MUX_0, mux | 0x100)?;
        let base: u32 = self.spi_bus.read_register(registers::NMI_INTR_REG_BASE)?;
        self.spi_bus
            .write_register(registers::NMI_INTR_REG_BASE, base | 0x10000)?;
        Ok(())
    }

    /// Gets the version of the firmware on the Atwinc1500
    pub fn get_firmware_version(&mut self) -> Result<FirmwareVersion, Error> {
        match self.state.firmware_version {
            Some(fw) => Ok(fw),
            None => {
                let mut reg_value = self.spi_bus.read_register(registers::NMI_REV_REG)?;
                if reg_value == registers::M2M_ATE_FW_IS_UP_VALUE {
                    reg_value = self.spi_bus.read_register(registers::NMI_REV_REG_ATE)?;
                }
                let fw_vers = FirmwareVersion([
                    ((reg_value >> 8) & 0xff) as u8, // major
                    ((reg_value >> 4) & 0x0f) as u8, // minor
                    (reg_value & 0x0f) as u8,        // patch
                ]);
                if self.state.firmware_version.is_none() {
                    self.state.set_firmware_version(fw_vers);
                }
                Ok(fw_vers)
            }
        }
    }

    #[doc(hidden)]
    /// Gets the mac address stored in one time programmable memory
    pub fn get_otp_mac_address(&mut self) -> Result<MacAddress, Error> {
        todo!()
    }

    /// Gets the working mac address on the Atwinc1500
    pub fn get_mac_address(&mut self) -> Result<MacAddress, Error> {
        match self.state.mac_address {
            Some(mac) => Ok(mac),
            None => {
                const MAC_SIZE: usize = 6;
                const DATA_SIZE: usize = 8;
                let mut mac: MacAddress = MacAddress([0; MAC_SIZE]);
                let mut data: [u8; DATA_SIZE] = [0; DATA_SIZE];
                let mut reg_value = self.spi_bus.read_register(registers::rNMI_GP_REG_2)?;
                reg_value |= 0x30000;
                self.spi_bus
                    .read_data(&mut data, reg_value, DATA_SIZE as u32)?;
                reg_value = combine_bytes_lsb!(data[0..4]);
                reg_value &= 0x0000ffff;
                reg_value |= 0x30000;
                self.spi_bus
                    .read_data(&mut mac.0, reg_value, MAC_SIZE as u32)?;
                if self.state.mac_address.is_none() {
                    self.state.set_mac_address(mac);
                }
                Ok(mac)
            }
        }
    }

    /// Sets the direction of a gpio pin to either Output or Input
    pub fn set_gpio_direction(
        &mut self,
        gpio: AtwincGpio,
        direction: GpioDirection,
    ) -> Result<(), Error> {
        const GPIO_DIR_REG: u32 = 0x20108;
        let mut value = self.spi_bus.read_register(GPIO_DIR_REG)?;
        match direction {
            GpioDirection::Output => value |= 1 << gpio as u8,
            GpioDirection::Input => value &= !(1 << gpio as u8),
        }
        self.spi_bus.write_register(GPIO_DIR_REG, value)?;
        Ok(())
    }

    /// Sets the value of a gpio pin as either High or Low
    pub fn set_gpio_value(&mut self, gpio: AtwincGpio, value: GpioValue) -> Result<(), Error> {
        const GPIO_VAL_REG: u32 = 0x20100;
        let mut response = self.spi_bus.read_register(GPIO_VAL_REG)?;
        match value {
            GpioValue::Low => response |= 1 << gpio as u8,
            GpioValue::High => response &= !(1 << gpio as u8),
        }
        self.spi_bus.write_register(GPIO_VAL_REG, response)?;
        Ok(())
    }

    /// Gets the direction of a gpio pin as either Ouput or Input
    pub fn get_gpio_direction(&mut self, gpio: AtwincGpio) -> Result<GpioDirection, Error> {
        const GPIO_GET_DIR_REG: u32 = 0x20104;
        let response = self.spi_bus.read_register(GPIO_GET_DIR_REG)?;
        Ok(GpioDirection::from(((response >> gpio as u8) & 0x01) as u8))
    }

    /// Connects to a wireless network
    ///
    /// # Examples
    /// ```ignore
    /// use atwinc1500::wifi::Channel;
    /// use atwinc1500::wifi::Connection;
    /// use atwinc1500::Atwinc1500;
    ///
    /// let connection = Connection::wpa_psk("my_network".as_bytes(), "my_password".as_bytes(), Channel::default(), 0);
    /// let mut atwinc = Atwinc1500::new(spi, delay, cs, reset, false);
    /// match atwinc.initialize() {
    ///     Err(e) => info!("{:?}", e),
    /// }
    /// match atwinc.connect_network(connection) {
    ///     Err(e) => info!("{:?}", e),
    /// }
    /// ```
    pub fn connect_network(&mut self, connection: Connection) -> Result<(), Error> {
        let mut conn_header: OldConnection = connection.into();
        let hif_header = HifHeader::new(
            group_ids::WIFI,
            WifiCommand::ReqConnect as u8,
            conn_header.len() as u16,
        );
        self.hif
            .send(&mut self.spi_bus, hif_header, &mut conn_header, &mut [])?;
        Ok(())
    }

    /// Disconnects from a wireless network
    pub fn disconnect_network(&mut self) -> Result<(), Error> {
        let hif_header = HifHeader::new(group_ids::WIFI, WifiCommand::ReqDisconnect as u8, 0);
        self.hif
            .send(&mut self.spi_bus, hif_header, &mut [], &mut [])?;
        Ok(())
    }

    /// Connects to the last remembered network
    pub fn connect_default_network(&mut self) -> Result<(), Error> {
        let hif_header = HifHeader::new(group_ids::WIFI, WifiCommand::ReqDefaultConnect as u8, 0);
        self.hif
            .send(&mut self.spi_bus, hif_header, &mut [], &mut [])?;
        Ok(())
    }

    /// Request the connection info of the network currently connected to
    pub fn request_connection_info(&mut self) -> Result<(), Error> {
        let hif_header = HifHeader::new(group_ids::WIFI, WifiCommand::ReqGetConnInfo as u8, 0);
        self.hif
            .send(&mut self.spi_bus, hif_header, &mut [], &mut [])?;
        Ok(())
    }

    /// Requests the Atwinc1500 to begin a scan for networks
    pub fn request_network_scan(&mut self, channel: Channel) -> Result<(), Error> {
        if self.state.scan_in_progress {
            return Err(Error::ScanError(ScanError::ScanInProgress));
        }
        let mut channel: [u8; 4] = ScanChannel::new(channel).into();
        let hif_header = HifHeader::new(
            group_ids::WIFI,
            WifiCommand::ReqScan as u8,
            channel.len() as u16,
        );
        self.hif
            .send(&mut self.spi_bus, hif_header, &mut channel, &mut [])?;
        self.state.scan_in_progress = true;
        Ok(())
    }

    /// Gets the result from the previous scan at the index passed to this function
    pub fn request_scan_result(&mut self, index: u8) -> Result<(), Error> {
        if index >= self.state.num_ap {
            return Err(Error::ScanError(ScanError::IndexOutOfRange));
        }
        let mut scan_index: [u8; 4] = ScanResultIndex(index).into();
        let hif_header = HifHeader::new(
            group_ids::WIFI,
            WifiCommand::ReqScanResult as u8,
            scan_index.len() as u16,
        );
        self.hif
            .send(&mut self.spi_bus, hif_header, &mut scan_index, &mut [])?;
        Ok(())
    }

    /// Requests the system time from the Atwinc1500
    pub fn request_system_time(&mut self) -> Result<(), Error> {
        let hif_header = HifHeader::new(group_ids::WIFI, WifiCommand::ReqGetSysTime as u8, 0);
        self.hif
            .send(&mut self.spi_bus, hif_header, &mut [], &mut [])?;
        Ok(())
    }

    /// The handle_events method takes care of the Atwinc1500 callbacks
    ///
    /// The Atwinc1500 has an interrupt line that it pulls low
    /// when events need handling. It's best to set up an interrupt on the
    /// irq pin falling low and call this method in the interrupt routine. In
    /// an environment with std a thread could be used as well. The
    /// handle_events method can also be called in a synchronous fashion after
    /// any Atwinc1500 method that uses a callback.
    ///
    /// # Examples
    ///
    /// Handling events synchronously:
    /// ```ignore
    /// match atwinc1500.request_system_time() {
    ///     Err(e) => info!("{}", e),
    /// }
    /// match atwinc1500.handle_events().unwrap() {
    ///     Err(e) => info!("{}", e),
    /// }
    /// if let Some(time) = atwinc1500.get_system_time() {
    ///     info!("{:?}", time);
    /// }
    /// ```
    /// To see an example of handling events with an interrupt or threads
    /// see the [examples repo](https://github.com/drewtchrist/atwinc1500-rs-examples)
    pub fn handle_events(&mut self) -> Result<(), Error> {
        self.hif.isr(&mut self.spi_bus, &mut self.state)?;
        Ok(())
    }

    /// Returns most recently retrieved scan result after a call to
    /// [request_scan_result](Atwinc1500::request_scan_result)
    pub fn get_scan_result(&self) -> &Option<ScanResult> {
        &self.state.scan_result
    }

    /// Returns the number of access points that were found in a scan
    /// after calling [request_network_scan](Atwinc1500::request_network_scan)
    pub fn get_num_ap(&self) -> u8 {
        self.state.num_ap
    }

    /// Returns the connection status of the Atwinc1500 after calling
    /// [connect_network](Atwinc1500::connect_network) or
    /// [connect_default_network](Atwinc1500::connect_default_network)
    pub fn get_status(&self) -> &Status {
        &self.state.status
    }

    /// Returns the system time of the Atwinc1500 after calling
    /// [request_system_time](Atwinc1500::request_system_time)
    pub fn get_system_time(&self) -> &Option<SystemTime> {
        &self.state.system_time
    }

    /// Returns the [ConnectionInfo] for the current connection
    /// after calling [request_connection_info](Atwinc1500::request_connection_info)
    pub fn get_connection_info(&self) -> &Option<ConnectionInfo> {
        &self.state.connection_info
    }
}

#[doc(hidden)]
impl<SPI, D, O> TcpClientStack for Atwinc1500<SPI, D, O>
where
    SPI: Transfer<u8>,
    D: DelayMs<u32>,
    O: OutputPin,
{
    type TcpSocket = TcpSocket;
    type Error = Error;

    fn socket(&mut self) -> Result<TcpSocket, Error> {
        todo!()
    }

    fn connect(
        &mut self,
        _socket: &mut TcpSocket,
        _address: SocketAddr,
    ) -> Result<(), embedded_nal::nb::Error<Error>> {
        todo!()
    }

    fn is_connected(&mut self, _socket: &TcpSocket) -> Result<bool, Error> {
        todo!()
    }

    fn send(
        &mut self,
        _socket: &mut TcpSocket,
        _data: &[u8],
    ) -> Result<usize, embedded_nal::nb::Error<Error>> {
        todo!()
    }

    fn receive(
        &mut self,
        _socket: &mut TcpSocket,
        _data: &mut [u8],
    ) -> Result<usize, embedded_nal::nb::Error<Error>> {
        todo!()
    }

    fn close(&mut self, _socket: TcpSocket) -> Result<(), Error> {
        todo!()
    }
}

#[doc(hidden)]
impl<SPI, D, O> TcpFullStack for Atwinc1500<SPI, D, O>
where
    SPI: Transfer<u8>,
    D: DelayMs<u32>,
    O: OutputPin,
{
    fn bind(&mut self, _socket: &mut TcpSocket, _port: u16) -> Result<(), Error> {
        todo!()
    }

    fn listen(&mut self, _socket: &mut TcpSocket) -> Result<(), Error> {
        todo!()
    }

    fn accept(
        &mut self,
        _socket: &mut TcpSocket,
    ) -> Result<(TcpSocket, SocketAddr), embedded_nal::nb::Error<Error>> {
        todo!()
    }
}

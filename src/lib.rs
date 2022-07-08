//! This is a driver for the atwinc1500 wifi module written in Rust. The
//! primary targets for this driver are the [Adafruit Feather M0 Wifi](https://adafruit.com/product/3010)
//! and the [Adafruit Atwinc1500 Breakout](https://adafruit.com/product/2999).
//! This may put some features outside the scope of this project, but they
//! are still welcomed additions. Parts of this code have been adapted from code found in
//! these two repositories [WiFi101](https://github.com/arduino-libraries/wifi101)
//! and [winc_wifi](https://github.com/jbentham/winc_wifi).
//!
//! ## Examples
//! Examples can be found at
//! [github.com/DrewTChrist/atwin1500-rs-examples](https://github.com/drewtchrist/atwinc1500-rs-examples).
//!
#![no_std]
#![allow(dead_code)]
#![allow(unused_variables)]

#[macro_use]
mod macros;
pub mod error;
pub mod gpio;
mod hif;
#[doc(hidden)]
pub mod registers;
#[doc(hidden)]
pub mod spi;
pub mod types;
pub mod wifi;

use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::blocking::spi::Transfer;
use embedded_hal::digital::v2::{InputPin, OutputPin};
use embedded_nal::SocketAddr;
use embedded_nal::TcpClientStack;
use embedded_nal::TcpFullStack;

use error::Error;
use gpio::{AtwincGpio, GpioDirection, GpioValue};
use hif::HostInterface;
use spi::SpiBusWrapper;
use types::FirmwareVersion;
use types::MacAddress;
use wifi::ConnectionParameters;
use wifi::TcpSocket;

/// Atwin1500 driver struct
pub struct Atwinc1500<SPI, D, O, I>
where
    SPI: Transfer<u8>,
    D: DelayMs<u32>,
    O: OutputPin,
    I: InputPin,
{
    delay: D,
    spi_bus: SpiBusWrapper<SPI, O>,
    hif: HostInterface,
    irq: I,
    reset: O,
    wake: O,
    crc: bool,
}

/// Atwinc1500 struct implementation containing non embedded-nal
/// public methods
impl<SPI, D, O, I> Atwinc1500<SPI, D, O, I>
where
    SPI: Transfer<u8>,
    D: DelayMs<u32>,
    O: OutputPin,
    I: InputPin,
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
    /// * `irq` - An InputPin for interrupt requests
    ///
    /// * `reset` - An OutputPin for chip reset
    ///
    /// * `wake` - An OutputPin for chip wake
    ///
    /// * `crc` - Turn on CRC in transactions
    ///
    pub fn new(
        spi: SPI,
        delay: D,
        cs: O,
        irq: I,
        reset: O,
        wake: O,
        crc: bool,
    ) -> Result<Self, Error> {
        let mut s = Self {
            delay,
            spi_bus: SpiBusWrapper::new(spi, cs),
            hif: HostInterface {},
            irq,
            reset,
            wake,
            crc,
        };
        s.initialize()?;
        Ok(s)
    }

    /// Initializes the driver by:
    /// * Initializing pins between devices
    /// * Disables crc if needed
    /// * Waits for efuse ready
    /// * Waits for boot rom ready
    /// * Writes driver version and configuration
    /// * Enables chip interrupt
    fn initialize(&mut self) -> Result<(), Error> {
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
        Ok(())
    }

    /// Pulls the chip select and wake pins high
    /// Then pulses (low/high) the reset pin with
    /// a delay
    fn init_pins(&mut self) -> Result<(), Error> {
        self.spi_bus.init_cs()?;
        if self.wake.set_high().is_err() {
            return Err(Error::PinStateError);
        }
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
            let mut cmd_buffer: [u8; 11] = [0; 11];
            let command = spi::commands::CMD_SINGLE_WRITE;
            let address = registers::NMI_SPI_PROTOCOL_CONFIG;
            let data = 0x52; // Still unsure of this value
            cmd_buffer[8] = 0x5c; // CRC value for this write
            self.spi_bus
                .command(&mut cmd_buffer, command, address, data, 0, false)?;
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

    /// Gets the version of the firmware on
    /// the Atwinc1500
    pub fn get_firmware_version(&mut self) -> Result<FirmwareVersion, Error> {
        let mut reg_value = self.spi_bus.read_register(registers::NMI_REV_REG)?;
        if reg_value == registers::M2M_ATE_FW_IS_UP_VALUE {
            reg_value = self.spi_bus.read_register(registers::NMI_REV_REG_ATE)?;
        }
        Ok(FirmwareVersion([
            ((reg_value >> 8) & 0xff) as u8, // major
            ((reg_value >> 4) & 0x0f) as u8, // minor
            (reg_value & 0x0f) as u8,        // patch
        ]))
    }

    pub fn get_otp_mac_address(&mut self) -> Result<MacAddress, Error> {
        todo!()
    }

    /// Gets the working mac address
    /// on the Atwinc1500
    pub fn get_mac_address(&mut self) -> Result<MacAddress, Error> {
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
        Ok(mac)
    }

    /// Sets the direction of a gpio pin
    /// to either Output or Input
    pub fn set_gpio_direction(
        &mut self,
        gpio: AtwincGpio,
        direction: GpioDirection,
    ) -> Result<(), Error> {
        const GPIO_DIR_REG: u32 = 0x20108;
        let mut value = self.spi_bus.read_register(GPIO_DIR_REG)?;
        if direction == GpioDirection::Output {
            value |= 1 << gpio as u8;
        } else {
            value &= !(1 << gpio as u8);
        }
        self.spi_bus.write_register(GPIO_DIR_REG, value)
    }

    /// Sets the value of a gpio
    /// pin as either High or Low
    pub fn set_gpio_value(&mut self, gpio: AtwincGpio, value: GpioValue) -> Result<(), Error> {
        const GPIO_VAL_REG: u32 = 0x20100;
        let mut response = self.spi_bus.read_register(GPIO_VAL_REG)?;
        if value == GpioValue::Low {
            response |= 1 << gpio as u8;
        } else {
            response &= !(1 << gpio as u8);
        }
        self.spi_bus.write_register(GPIO_VAL_REG, response)
    }

    /// Gets the direction of a gpio pin
    /// as either Ouput or Input
    pub fn get_gpio_direction(&mut self, gpio: AtwincGpio) -> Result<GpioDirection, Error> {
        const GPIO_GET_DIR_REG: u32 = 0x20104;
        match self.spi_bus.read_register(GPIO_GET_DIR_REG) {
            Ok(v) => Ok(GpioDirection::from(((v >> gpio as u8) & 0x01) as u8)),
            Err(e) => Err(e),
        }
    }

    pub fn connect_network(&mut self, connection: ConnectionParameters) -> Result<(), Error> {
        Ok(())
    }
}

impl<SPI, D, O, I> TcpClientStack for Atwinc1500<SPI, D, O, I>
where
    SPI: Transfer<u8>,
    D: DelayMs<u32>,
    O: OutputPin,
    I: InputPin,
{
    type TcpSocket = TcpSocket;
    type Error = Error;

    fn socket(&mut self) -> Result<TcpSocket, Error> {
        todo!()
    }

    fn connect(
        &mut self,
        socket: &mut TcpSocket,
        address: SocketAddr,
    ) -> Result<(), embedded_nal::nb::Error<Error>> {
        todo!()
    }

    fn is_connected(&mut self, socket: &TcpSocket) -> Result<bool, Error> {
        todo!()
    }

    fn send(
        &mut self,
        socket: &mut TcpSocket,
        data: &[u8],
    ) -> Result<usize, embedded_nal::nb::Error<Error>> {
        todo!()
    }

    fn receive(
        &mut self,
        socket: &mut TcpSocket,
        data: &mut [u8],
    ) -> Result<usize, embedded_nal::nb::Error<Error>> {
        todo!()
    }

    fn close(&mut self, socket: TcpSocket) -> Result<(), Error> {
        todo!()
    }
}

impl<SPI, D, O, I> TcpFullStack for Atwinc1500<SPI, D, O, I>
where
    SPI: Transfer<u8>,
    D: DelayMs<u32>,
    O: OutputPin,
    I: InputPin,
{
    fn bind(&mut self, socket: &mut TcpSocket, port: u16) -> Result<(), Error> {
        todo!()
    }

    fn listen(&mut self, socket: &mut TcpSocket) -> Result<(), Error> {
        todo!()
    }

    fn accept(
        &mut self,
        socket: &mut TcpSocket,
    ) -> Result<(TcpSocket, SocketAddr), embedded_nal::nb::Error<Error>> {
        todo!()
    }
}

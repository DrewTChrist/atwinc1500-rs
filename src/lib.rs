#![no_std]
#![allow(dead_code)]
#![allow(unused_variables)]

pub mod error;
mod hif;
#[macro_use]
mod macros;
mod registers;
mod spi;
mod traits;

use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::digital::v2::{InputPin, OutputPin};
use embedded_hal::spi::FullDuplex;
use embedded_nal::SocketAddr;
use embedded_nal::TcpClientStack;
use embedded_nal::TcpFullStack;
#[macro_use(block)]
extern crate nb;

use error::Error;
use traits::HifLayer;
use traits::SpiLayer;

pub struct TcpSocket {}

/// Atwin1500 driver struct
pub struct Atwinc1500<SPI, D, O, I>
where
    SPI: FullDuplex<u8>,
    D: DelayMs<u32>,
    O: OutputPin,
    I: InputPin,
{
    spi: SPI,
    delay: D,
    cs: O,
    irq: I,
    reset: O,
    wake: O,
    crc: bool,
}

/// Atwinc1500 struct implementation containing non embedded-nal
/// public methods
impl<SPI, D, O, I> Atwinc1500<SPI, D, O, I>
where
    SPI: FullDuplex<u8>,
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
    /// # Examples
    ///
    /// Examples can be found at
    /// [github.com/DrewTChrist/atwin1500-rs-examples](https://github.com/drewtchrist/atwinc1500-rs-examples).
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
            spi,
            delay,
            cs,
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
        let mut tries: u8 = 10;
        self.init_pins()?;
        self.disable_crc()?;
        let mut efuse_value: u32 = 0;
        while tries > 0 && (efuse_value & 0x80000000) == 0 {
            efuse_value = self.spi_read_register(registers::EFUSE_REG)?;
            self.delay.delay_ms(1000);
            tries -= 1;
        }
        let wait: u32 = self.spi_read_register(registers::M2M_WAIT_FOR_HOST_REG)?;
        if (wait & 1) == 0 {
            tries = 3;
            let mut bootrom: u32 = 0;
            while tries > 0 && bootrom != FINISH_BOOT_VAL {
                bootrom = self.spi_read_register(registers::BOOTROM_REG)?;
                self.delay.delay_ms(1000);
                tries -= 1;
            }
        }
        self.spi_write_register(registers::NMI_STATE_REG, DRIVER_VER_INFO)?;
        self.spi_write_register(registers::rNMI_GP_REG_1, CONF_VAL)?;
        self.spi_write_register(registers::BOOTROM_REG, START_FIRMWARE)?;
        tries = 20;
        let mut state: u32 = 0;
        while tries > 0 && state != FINISH_INIT_VAL {
            state = self.spi_read_register(registers::NMI_STATE_REG)?;
            self.delay.delay_ms(1000);
            tries -= 1;
        }
        self.spi_write_register(registers::NMI_STATE_REG, 0)?;
        self.enable_chip_interrupt()?;
        Ok(())
    }

    /// Pulls the chip select and wake pins high
    /// Then pulses (low/high) the reset pin with
    /// a delay
    fn init_pins(&mut self) -> Result<(), Error> {
        if self.cs.set_high().is_err() {
            return Err(Error::PinStateError);
        }
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
            self.spi_command(&mut cmd_buffer, command, address, data, 0, false)?;
        }
        Ok(())
    }

    fn enable_chip_interrupt(&mut self) -> Result<(), Error> {
        let mux: u32 = self.spi_read_register(registers::NMI_PIN_MUX_0)?;
        self.spi_write_register(registers::NMI_PIN_MUX_0, mux | 0x100)?;
        let base: u32 = self.spi_read_register(registers::NMI_INTR_REG_BASE)?;
        self.spi_write_register(registers::NMI_INTR_REG_BASE, base | 0x10000)?;
        Ok(())
    }

    /// Get chip firmware version and mac address
    pub fn get_chip_info(&mut self) -> Result<([u8; 8], [u8; 6], [u8; 40]), Error> {
        let mut data: [u8; 8] = [0; 8];
        let mut info: [u8; 40] = [0; 40];
        let mut mac: [u8; 6] = [0; 6];
        let mut count = data.len();
        let val = self.spi_read_register(registers::rNMI_GP_REG_2)?;
        self.spi_read_data(&mut data, val | 0x30000, count as u32)?;
        count = info.len();
        self.spi_read_data(&mut info, combine_bytes!(data[4..5]) | 0x30000, count as u32)?;
        count = mac.len();
        self.spi_read_data(&mut mac, combine_bytes!(data[2..3]) | 0x30000, count as u32)?;
        Ok((data, mac, info))
    }
}

impl<SPI, D, O, I> SpiLayer for Atwinc1500<SPI, D, O, I>
where
    SPI: FullDuplex<u8>,
    D: DelayMs<u32>,
    O: OutputPin,
    I: InputPin,
{
    /// Sends some data then receives some data on the spi bus
    fn spi_transfer(&mut self, words: &'_ mut [u8]) -> Result<(), Error> {
        if self.cs.set_low().is_err() {
            return Err(Error::PinStateError);
        }
        for word in words.iter_mut() {
            if block!(self.spi.send(*word)).is_err() {
                return Err(Error::SpiTransferError);
            }
            match block!(self.spi.read()) {
                Ok(v) => *word = v,
                Err(_) => return Err(Error::SpiTransferError),
            }
        }
        if self.cs.set_high().is_err() {
            return Err(Error::PinStateError);
        }
        Ok(())
    }

    /// Matches the command argument and formats
    /// the address, data, and size arguments
    /// into the cmd_buffer as described in the
    /// software design guide then sends the command
    fn spi_command(
        &mut self,
        cmd_buffer: &'_ mut [u8],
        command: u8,
        address: u32,
        data: u32,
        size: u32,
        clockless: bool,
    ) -> Result<(), Error> {
        cmd_buffer[0] = command;
        match command {
            spi::commands::CMD_DMA_WRITE => {}
            spi::commands::CMD_DMA_READ => {
                cmd_buffer[1] = (address >> 16) as u8;
                cmd_buffer[2] = (address >> 8) as u8;
                cmd_buffer[3] = address as u8;
                cmd_buffer[4] = (size >> 8) as u8;
                cmd_buffer[5] = size as u8;
            }
            spi::commands::CMD_INTERNAL_WRITE => {
                cmd_buffer[1] = (address >> 8) as u8;
                if clockless {
                    cmd_buffer[1] |= 1 << 7;
                }
                cmd_buffer[2] = address as u8;
                cmd_buffer[3] = (data >> 24) as u8;
                cmd_buffer[4] = (data >> 16) as u8;
                cmd_buffer[5] = (data >> 8) as u8;
                cmd_buffer[6] = data as u8;
            }
            spi::commands::CMD_INTERNAL_READ => {
                cmd_buffer[1] = (address >> 8) as u8;
                if clockless {
                    cmd_buffer[1] |= 1 << 7;
                }
                cmd_buffer[2] = address as u8;
                cmd_buffer[3] = 0;
            }
            spi::commands::CMD_TERMINATE => {
                cmd_buffer[1] = 0x0;
                cmd_buffer[2] = 0x0;
                cmd_buffer[3] = 0x0;
            }
            spi::commands::CMD_REPEAT => {
                cmd_buffer[1] = 0x0;
                cmd_buffer[2] = 0x0;
                cmd_buffer[3] = 0x0;
            }
            spi::commands::CMD_DMA_EXT_WRITE => {}
            spi::commands::CMD_DMA_EXT_READ => {
                cmd_buffer[1] = (address >> 16) as u8;
                cmd_buffer[2] = (address >> 8) as u8;
                cmd_buffer[3] = address as u8;
                cmd_buffer[4] = (size >> 16) as u8;
                cmd_buffer[5] = (size >> 8) as u8;
                cmd_buffer[6] = size as u8;
            }
            spi::commands::CMD_SINGLE_WRITE => {
                cmd_buffer[1] = (address >> 16) as u8;
                cmd_buffer[2] = (address >> 8) as u8;
                cmd_buffer[3] = address as u8;
                cmd_buffer[4] = (data >> 24) as u8;
                cmd_buffer[5] = (data >> 16) as u8;
                cmd_buffer[6] = (data >> 8) as u8;
                cmd_buffer[7] = data as u8;
            }
            spi::commands::CMD_SINGLE_READ => {
                cmd_buffer[1] = (address >> 16) as u8;
                cmd_buffer[2] = (address >> 8) as u8;
                cmd_buffer[3] = address as u8;
            }
            spi::commands::CMD_RESET => {
                cmd_buffer[1] = 0xff;
                cmd_buffer[2] = 0xff;
                cmd_buffer[3] = 0xff;
            }
            _ => {
                return Err(Error::InvalidSpiCommandError);
            }
        }
        self.spi_transfer(cmd_buffer)?;
        Ok(())
    }

    /// Reads a value from a register at address
    /// then writes it to cmd_buffer
    fn spi_read_register(&mut self, address: u32) -> Result<u32, Error> {
        // The Atmel driver does a clockless read
        // if address is less than 0xff (0b11111111).
        // I did not spot any addresses less than 0xff.
        // To me this is a magic number, I leave
        // it here just in case
        let cmd: u8;
        let clockless: bool;
        let mut cmd_buffer: [u8; 12] = [0; 12];
        if address <= 0xff {
            cmd = spi::commands::CMD_INTERNAL_READ;
            clockless = true;
        } else {
            cmd = spi::commands::CMD_SINGLE_READ;
            clockless = false;
        }
        self.spi_command(&mut cmd_buffer, cmd, address, 0, 0, clockless)?;
        // TODO: The hardcoded indices here will
        // not be the same if crc is on
        Ok(combine_bytes_lsb!(cmd_buffer[7..11]))
    }

    fn spi_read_data(&mut self, data: &mut [u8], address: u32, count: u32) -> Result<(), Error> {
        let cmd: u8 = spi::commands::CMD_DMA_EXT_READ;
        let mut cmd_buffer: [u8; 7] = [0; 7];
        let mut transfer: [u8; 3] = [0; 3];
        self.spi_command(&mut cmd_buffer, cmd, address, 0, count, false)?;
        let mut tries = 10;
        while tries > 0 && transfer[0] == 0 {
            self.spi_transfer(&mut transfer)?;
            tries -= 1;
        }
        if transfer[0] == cmd {
            self.spi_transfer(data)?;
        }
        Ok(())
    }

    /// Writes a value to a register at
    /// address and writes the response
    /// to cmd_buffer
    fn spi_write_register(&mut self, address: u32, data: u32) -> Result<(), Error> {
        // The Atmel driver does a clockless write
        // if address is less than 0x30 (0b00110000).
        // I did not spot any addresses less than 0x30.
        // To me this is a magic number, I leave
        // it here just in case
        let cmd: u8;
        let clockless: bool;
        let mut cmd_buffer: [u8; 11] = [0; 11];
        if address <= 0x30 {
            cmd = spi::commands::CMD_INTERNAL_WRITE;
            clockless = true;
        } else {
            cmd = spi::commands::CMD_SINGLE_WRITE;
            clockless = false;
        }
        self.spi_command(&mut cmd_buffer, cmd, address, data, 0, clockless)?;
        // TODO: The hardcoded indices here will
        // not be the same if crc is on
        if cmd_buffer[8] != cmd || cmd_buffer[9] != 0 {
            return Err(Error::SpiWriteRegisterError);
        }
        Ok(())
    }

    fn spi_write_data(&mut self, address: u32, data: u32) -> Result<(), Error> {
        todo!()
    }
}

impl<SPI, D, O, I> HifLayer for Atwinc1500<SPI, D, O, I>
where
    SPI: FullDuplex<u8>,
    D: DelayMs<u32>,
    O: OutputPin,
    I: InputPin,
{
    /// This method wakes the chip from sleep mode using clockless register access
    fn hif_chip_wake(&mut self) -> Result<(), Error> {
        todo!()
    }

    /// This method enables sleep mode for the chip
    fn hif_chip_sleep(&mut self) -> Result<(), Error> {
        todo!()
    }

    /// This method sets the callback function for different events
    fn hif_register_cb(&mut self) -> Result<(), Error> {
        todo!()
    }

    /// This method is the host interface interrupt service
    fn hif_isr(&mut self) -> Result<(), Error> {
        todo!()
    }

    /// This method receives data read from the chip
    fn hif_receive(&mut self) -> Result<(), Error> {
        todo!()
    }

    /// This method sends data to the chip
    fn hif_send(&mut self) -> Result<(), Error> {
        todo!()
    }

    /// This method sets the chip sleep mode
    fn hif_set_sleep_mode(&mut self) -> Result<(), Error> {
        todo!()
    }

    /// This method returns the chip sleep mode
    fn hif_get_sleep_mode(&mut self) -> Result<(), Error> {
        todo!()
    }
}

impl<SPI, D, O, I> TcpClientStack for Atwinc1500<SPI, D, O, I>
where
    SPI: FullDuplex<u8>,
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
    SPI: FullDuplex<u8>,
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

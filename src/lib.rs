#![no_std]
#![allow(dead_code)]
#![allow(unused_variables)]

pub mod error;
mod hif;
mod registers;
mod spi;

use embedded_hal::blocking::spi::Transfer;
use embedded_hal::digital::v2::{InputPin, OutputPin};
use embedded_hal::spi::FullDuplex;
use embedded_nal::SocketAddr;
use embedded_nal::TcpClientStack;
use embedded_nal::TcpFullStack;

use error::Error;

/// Defines the needed functions to handle the spi layer
/// as described in the atwinc1500 software design guide
trait SpiLayer {
    fn spi_transfer<'w>(&mut self, words: &'w mut [u8]) -> Result<&'w [u8], Error>;
    fn spi_command<'w>(
        &mut self,
        cmd_buffer: &'w mut [u8],
        command: u8,
        address: u32,
        data: u32,
        size: u32,
        clockless: bool,
    ) -> Result<&'w [u8], Error>;
    fn spi_read_register<'w>(
        &mut self,
        cmd_buffer: &'w mut [u8],
        address: u32,
    ) -> Result<&'w [u8], Error>;
    fn spi_read_data<'w>(&mut self, cmd_buffer: &'w mut [u8]) -> Result<&'w [u8], Error>;
    fn spi_write_register<'w>(
        &mut self,
        cmd_buffer: &'w mut [u8],
        address: u32,
        data: u32,
    ) -> Result<&'w [u8], Error>;
    fn spi_write_data<'w>(&mut self, cmd_buffer: &'w mut [u8]) -> Result<&'w [u8], Error>;
}

/// Defines the needed functions to handle the host interface
/// layer as described in the atwinc1500 software design guide
trait HifLayer {
    fn hif_chip_wake(&mut self);
    fn hif_chip_sleep(&mut self);
    fn hif_register_cb(&mut self);
    fn hif_isr(&mut self);
    fn hif_receive(&mut self);
    fn hif_send(&mut self);
    fn hif_set_sleep_mode(&mut self);
    fn hif_get_sleep_mode(&mut self);
}

pub struct TcpSocket {}

/// Atwin1500 driver struct
pub struct Atwinc1500<SPI, O, I>
where
    SPI: FullDuplex<u8> + Transfer<u8>,
    O: OutputPin,
    I: InputPin,
{
    spi: SPI,
    cs: O,
    sclk: I,
    irq: I,
    reset: O,
    wake: O,
    crc: bool,
}

/// Atwinc1500 struct implementation containing non embedded-nal
/// public methods
impl<SPI, O, I> Atwinc1500<SPI, O, I>
where
    SPI: FullDuplex<u8> + Transfer<u8>,
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
    /// * `sclk` - The spi clock InputPin
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
    pub fn new(spi: SPI, cs: O, sclk: I, irq: I, reset: O, wake: O, crc: bool) -> Self {
        let mut s = Self {
            spi,
            cs,
            sclk,
            irq,
            reset,
            wake,
            crc,
        };
        s.initialize();
        s
    }

    fn initialize(&mut self) {
        todo!()
    }

    /// Get chip firmware version and mac address
    pub fn get_chip_info(&mut self) {
        //let mut buffer: [u8; spi::commands::sizes::TYPE_A] = [0; spi::commands::sizes::TYPE_A];
        //self.spi_read_register(&mut buffer);
    }
}

impl<SPI, O, I> SpiLayer for Atwinc1500<SPI, O, I>
where
    SPI: FullDuplex<u8> + Transfer<u8>,
    O: OutputPin,
    I: InputPin,
{
    /// Transfers some data then receives some data on the spi bus
    fn spi_transfer<'w>(&mut self, words: &'w mut [u8]) -> Result<&'w [u8], Error> {
        if self.cs.set_low().is_err() {
            return Err(Error::PinStateError);
        }
        let rcv = self.spi.transfer(words);
        // This while loop may not be needed
        //while self.sclk.is_high().is_ok() {}
        if self.cs.set_high().is_err() {
            return Err(Error::PinStateError);
        }
        match rcv {
            Ok(val) => Ok(val),
            Err(e) => Err(Error::SpiTransferError),
        }
    }

    fn spi_command<'w>(
        &mut self,
        cmd_buffer: &'w mut [u8],
        command: u8,
        address: u32,
        data: u32,
        size: u32,
        clockless: bool,
    ) -> Result<&'w [u8], Error> {
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
        self.spi_transfer(cmd_buffer)
    }

    fn spi_read_register<'w>(
        &mut self,
        cmd_buffer: &'w mut [u8],
        address: u32,
    ) -> Result<&'w [u8], Error> {
        todo!()
    }

    fn spi_read_data<'w>(&mut self, cmd_buffer: &'w mut [u8]) -> Result<&'w [u8], Error> {
        todo!()
    }

    fn spi_write_register<'w>(
        &mut self,
        cmd_buffer: &'w mut [u8],
        address: u32,
        data: u32,
    ) -> Result<&'w [u8], Error> {
        todo!()
    }

    fn spi_write_data<'w>(&mut self, cmd_buffer: &'w mut [u8]) -> Result<&'w [u8], Error> {
        todo!()
    }
}

impl<SPI, O, I> HifLayer for Atwinc1500<SPI, O, I>
where
    SPI: FullDuplex<u8> + Transfer<u8>,
    O: OutputPin,
    I: InputPin,
{
    /// This method wakes the chip from sleep mode using clockless register access
    fn hif_chip_wake(&mut self) {
        todo!()
    }

    /// This method enables sleep mode for the chip
    fn hif_chip_sleep(&mut self) {
        todo!()
    }

    /// This method sets the callback function for different events
    fn hif_register_cb(&mut self) {
        todo!()
    }

    /// This method is the host interface interrupt service
    fn hif_isr(&mut self) {
        todo!()
    }

    /// This method receives data read from the chip
    fn hif_receive(&mut self) {
        todo!()
    }

    /// This method sends data to the chip
    fn hif_send(&mut self) {
        todo!()
    }

    /// This method sets the chip sleep mode
    fn hif_set_sleep_mode(&mut self) {
        todo!()
    }

    /// This method returns the chip sleep mode
    fn hif_get_sleep_mode(&mut self) {
        todo!()
    }
}

impl<SPI, O, I> TcpClientStack for Atwinc1500<SPI, O, I>
where
    SPI: FullDuplex<u8> + Transfer<u8>,
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

impl<SPI, O, I> TcpFullStack for Atwinc1500<SPI, O, I>
where
    SPI: FullDuplex<u8> + Transfer<u8>,
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

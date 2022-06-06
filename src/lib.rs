#![no_std]
#![allow(dead_code)]
#![allow(unused_variables)]

pub mod error;
mod hif;
mod registers;
mod spi;

use embedded_hal::blocking::delay::DelayMs;
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
    fn hif_chip_wake(&mut self) -> Result<(), Error>;
    fn hif_chip_sleep(&mut self) -> Result<(), Error>;
    fn hif_register_cb(&mut self) -> Result<(), Error>;
    fn hif_isr(&mut self) -> Result<(), Error>;
    fn hif_receive(&mut self) -> Result<(), Error>;
    fn hif_send(&mut self) -> Result<(), Error>;
    fn hif_set_sleep_mode(&mut self) -> Result<(), Error>;
    fn hif_get_sleep_mode(&mut self) -> Result<(), Error>;
}

/// Takes an array of bytes
/// combines it into one u32
///
/// If the size of the buffer
/// is greater than 4, the
/// remaining bytes will still
/// be shifted in
macro_rules! combine_bytes {
    ($buffer:expr) => {{
        let mut value: u32 = 0;
        for i in 0..$buffer.len() {
            value <<= 8_u32;
            value |= $buffer[i] as u32;
        }
        value
    }};
}

pub struct TcpSocket {}

/// Atwin1500 driver struct
pub struct Atwinc1500<SPI, D, O, I>
where
    SPI: FullDuplex<u8> + Transfer<u8>,
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
    SPI: FullDuplex<u8> + Transfer<u8>,
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
        let mut read_buf: [u8; spi::commands::sizes::TYPE_A] = [0; spi::commands::sizes::TYPE_A];
        let mut write_buf: [u8; spi::commands::sizes::TYPE_D] = [0; spi::commands::sizes::TYPE_D];
        let mut tries: u8 = 10;

        self.init_pins()?;
        if !self.crc {
            self.disable_crc()?;
        }
        while tries > 0 && read_buf[0] != 0x80 {
            self.spi_read_register(&mut read_buf, registers::EFUSE_REG)?;
            self.delay.delay_ms(1000);
            tries -= 1;
        }
        self.spi_read_register(&mut read_buf, registers::M2M_WAIT_FOR_HOST_REG)?;
        if (combine_bytes!(read_buf[0..read_buf.len() - 1]) & 1) == 0 {
            tries = 3;
            while tries > 0 && combine_bytes!(read_buf[0..read_buf.len() - 1]) != FINISH_BOOT_VAL {
                self.spi_read_register(&mut read_buf, registers::BOOTROM_REG)?;
                self.delay.delay_ms(1000);
                tries -= 1;
            }
        }
        self.spi_write_register(&mut write_buf, registers::NMI_STATE_REG, DRIVER_VER_INFO)?;
        self.spi_write_register(&mut write_buf, registers::rNMI_GP_REG_1, CONF_VAL)?;
        self.spi_write_register(&mut write_buf, registers::BOOTROM_REG, START_FIRMWARE)?;
        tries = 20;
        while tries > 0 && combine_bytes!(read_buf[0..read_buf.len() - 1]) !=  FINISH_INIT_VAL {
            self.spi_read_register(&mut read_buf, registers::NMI_STATE_REG)?;
            self.delay.delay_ms(1000);
            tries -= 1;
        }
        self.spi_write_register(&mut write_buf, registers::NMI_STATE_REG, 0)?;
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

    /// Sends a command to the atwinc1500
    /// to disable crc checks
    fn disable_crc(&mut self) -> Result<(), Error> {
        let mut disable_crc_cmd: [u8; 11] = [0xC9, 0, 0xE8, 0x24, 0, 0, 0, 0x52, 0x5C, 0, 0];
        self.spi_transfer(&mut disable_crc_cmd)?;
        Ok(())
    }

    /// Get chip firmware version and mac address
    pub fn get_chip_info(&mut self) {
        //let mut buffer: [u8; spi::commands::sizes::TYPE_A] = [0; spi::commands::sizes::TYPE_A];
        //self.spi_read_register(&mut buffer);
    }
}

impl<SPI, D, O, I> SpiLayer for Atwinc1500<SPI, D, O, I>
where
    SPI: FullDuplex<u8> + Transfer<u8>,
    D: DelayMs<u32>,
    O: OutputPin,
    I: InputPin,
{
    /// Sends some data then receives some data on the spi bus
    fn spi_transfer<'w>(&mut self, words: &'w mut [u8]) -> Result<&'w [u8], Error> {
        if self.cs.set_low().is_err() {
            return Err(Error::PinStateError);
        }
        let response = self.spi.transfer(words);
        if self.cs.set_high().is_err() {
            return Err(Error::PinStateError);
        }
        match response {
            Ok(val) => Ok(val),
            Err(_) => Err(Error::SpiTransferError),
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
        if address <= 0x30 {
            self.spi_command(
                cmd_buffer,
                spi::commands::CMD_INTERNAL_READ,
                address,
                0,
                0,
                true,
            )
        } else {
            self.spi_command(
                cmd_buffer,
                spi::commands::CMD_SINGLE_READ,
                address,
                0,
                0,
                false,
            )
        }
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
        self.spi_command(
            cmd_buffer,
            spi::commands::CMD_SINGLE_WRITE,
            address,
            data,
            0,
            false,
        )
    }

    fn spi_write_data<'w>(&mut self, cmd_buffer: &'w mut [u8]) -> Result<&'w [u8], Error> {
        todo!()
    }
}

impl<SPI, D, O, I> HifLayer for Atwinc1500<SPI, D, O, I>
where
    SPI: FullDuplex<u8> + Transfer<u8>,
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
    SPI: FullDuplex<u8> + Transfer<u8>,
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
    SPI: FullDuplex<u8> + Transfer<u8>,
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

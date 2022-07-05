use crate::error::Error;
use embedded_hal::blocking::spi::Transfer;
use embedded_hal::digital::v2::OutputPin;

pub mod commands {
    // Command start + Command Type
    // Command start = 0b1100
    // Example: 0b1100 | 0b0001 into one byte
    // CMD_DMA_WRITE = 0b11000001
    pub const CMD_DMA_WRITE: u8 = 0xc1;
    pub const CMD_DMA_READ: u8 = 0xc2;
    pub const CMD_INTERNAL_WRITE: u8 = 0xc3;
    pub const CMD_INTERNAL_READ: u8 = 0xc4;
    pub const CMD_TERMINATE: u8 = 0xc5;
    pub const CMD_REPEAT: u8 = 0xc6;
    pub const CMD_DMA_EXT_WRITE: u8 = 0xc7;
    pub const CMD_DMA_EXT_READ: u8 = 0xc8;
    pub const CMD_SINGLE_WRITE: u8 = 0xc9;
    pub const CMD_SINGLE_READ: u8 = 0xca;
    pub const CMD_RESET: u8 = 0xcf;
}

mod sizes {
    // Full command packet size with crc bit
    const TYPE_A: usize = 5;
    const TYPE_B: usize = 7;
    const TYPE_C: usize = 8;
    const TYPE_D: usize = 9;
}

/// The SpiBusWrapper struct
/// handles all reads/writes that
/// happen over the FullDuplex spi bus
pub struct SpiBusWrapper<SPI, O>
where
    SPI: Transfer<u8>,
    O: OutputPin,
{
    spi: SPI,
    cs: O,
}

impl<SPI, O> SpiBusWrapper<SPI, O>
where
    SPI: Transfer<u8>,
    O: OutputPin,
{
    pub fn new(spi: SPI, cs: O) -> Self {
        Self { spi, cs }
    }

    /// Pulls the chip select high
    /// as it is active low
    pub fn init_cs(&mut self) -> Result<(), Error> {
        match self.cs.set_high() {
            Ok(_) => Ok(()),
            Err(_) => Err(Error::PinStateError),
        }
    }

    /// Sends some data then receives some data on the spi bus
    fn transfer(&mut self, words: &'_ mut [u8]) -> Result<(), Error> {
        if self.cs.set_low().is_err() {
            return Err(Error::PinStateError);
        }
        if self.spi.transfer(words).is_err() {
            return Err(Error::SpiTransferError);
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
    pub fn command(
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
            commands::CMD_DMA_WRITE => {}
            commands::CMD_DMA_READ => {
                cmd_buffer[1] = (address >> 16) as u8;
                cmd_buffer[2] = (address >> 8) as u8;
                cmd_buffer[3] = address as u8;
                cmd_buffer[4] = (size >> 8) as u8;
                cmd_buffer[5] = size as u8;
            }
            commands::CMD_INTERNAL_WRITE => {
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
            commands::CMD_INTERNAL_READ => {
                cmd_buffer[1] = (address >> 8) as u8;
                if clockless {
                    cmd_buffer[1] |= 1 << 7;
                }
                cmd_buffer[2] = address as u8;
                cmd_buffer[3] = 0;
            }
            commands::CMD_TERMINATE => {
                cmd_buffer[1] = 0x0;
                cmd_buffer[2] = 0x0;
                cmd_buffer[3] = 0x0;
            }
            commands::CMD_REPEAT => {
                cmd_buffer[1] = 0x0;
                cmd_buffer[2] = 0x0;
                cmd_buffer[3] = 0x0;
            }
            commands::CMD_DMA_EXT_WRITE => {}
            commands::CMD_DMA_EXT_READ => {
                cmd_buffer[1] = (address >> 16) as u8;
                cmd_buffer[2] = (address >> 8) as u8;
                cmd_buffer[3] = address as u8;
                cmd_buffer[4] = (size >> 16) as u8;
                cmd_buffer[5] = (size >> 8) as u8;
                cmd_buffer[6] = size as u8;
            }
            commands::CMD_SINGLE_WRITE => {
                cmd_buffer[1] = (address >> 16) as u8;
                cmd_buffer[2] = (address >> 8) as u8;
                cmd_buffer[3] = address as u8;
                cmd_buffer[4] = (data >> 24) as u8;
                cmd_buffer[5] = (data >> 16) as u8;
                cmd_buffer[6] = (data >> 8) as u8;
                cmd_buffer[7] = data as u8;
            }
            commands::CMD_SINGLE_READ => {
                cmd_buffer[1] = (address >> 16) as u8;
                cmd_buffer[2] = (address >> 8) as u8;
                cmd_buffer[3] = address as u8;
            }
            commands::CMD_RESET => {
                cmd_buffer[1] = 0xff;
                cmd_buffer[2] = 0xff;
                cmd_buffer[3] = 0xff;
            }
            _ => {
                return Err(Error::InvalidSpiCommandError);
            }
        }
        self.transfer(cmd_buffer)?;
        Ok(())
    }

    /// Reads a value from a register at address
    /// then writes it to cmd_buffer
    pub fn read_register(&mut self, address: u32) -> Result<u32, Error> {
        // The Atmel driver does a clockless read
        // if address is less than 0xff (0b11111111).
        // I did not spot any addresses less than 0xff.
        // To me this is a magic number, I leave
        // it here just in case
        let cmd: u8;
        let clockless: bool;
        let mut cmd_buffer: [u8; 12] = [0; 12];
        if address <= 0xff {
            cmd = commands::CMD_INTERNAL_READ;
            clockless = true;
        } else {
            cmd = commands::CMD_SINGLE_READ;
            clockless = false;
        }
        self.command(&mut cmd_buffer, cmd, address, 0, 0, clockless)?;
        if cmd_buffer[4] != cmd || cmd_buffer[6] & 0xf0 != 0xf0 {
            return Err(Error::SpiReadRegisterError);
        }
        // TODO: The hardcoded indices here will
        // not be the same if crc is on
        Ok(combine_bytes_lsb!(cmd_buffer[7..11]))
    }

    /// Reads a block of data
    pub fn read_data(&mut self, data: &mut [u8], address: u32, count: u32) -> Result<(), Error> {
        let cmd: u8 = commands::CMD_DMA_EXT_READ;
        let mut cmd_buffer: [u8; 7] = [0; 7];
        let mut transfer: [u8; 3] = [0; 3];
        self.command(&mut cmd_buffer, cmd, address, 0, count, false)?;
        retry_while!(transfer[0] == 0, retries = 10, {
            self.transfer(&mut transfer)?;
        });
        if transfer[0] == cmd {
            self.transfer(data)?;
        }
        Ok(())
    }

    /// Writes a value to a register at
    /// address and writes the response
    /// to cmd_buffer
    pub fn write_register(&mut self, address: u32, data: u32) -> Result<(), Error> {
        // The Atmel driver does a clockless write
        // if address is less than 0x30 (0b00110000).
        // I did not spot any addresses less than 0x30.
        // To me this is a magic number, I leave
        // it here just in case
        let cmd: u8;
        let clockless: bool;
        let mut cmd_buffer: [u8; 11] = [0; 11];
        if address <= 0x30 {
            cmd = commands::CMD_INTERNAL_WRITE;
            clockless = true;
        } else {
            cmd = commands::CMD_SINGLE_WRITE;
            clockless = false;
        }
        self.command(&mut cmd_buffer, cmd, address, data, 0, clockless)?;
        // TODO: The hardcoded indices here will
        // not be the same if crc is on
        if cmd_buffer[8] != cmd || cmd_buffer[9] != 0 {
            return Err(Error::SpiWriteRegisterError);
        }
        Ok(())
    }

    /// Writes a block of data
    pub fn write_data(&mut self, data: &mut [u8], address: u32, count: u32) -> Result<(), Error> {
        let cmd: u8 = commands::CMD_DMA_EXT_WRITE;
        let mut cmd_buffer: [u8; 7] = [0; 7];
        let mut response: [u8; 2] = [0; 2];
        let data_mark: u8 = 0xf3;
        self.command(&mut cmd_buffer, cmd, address, 0, count, false)?;
        self.transfer(&mut response)?;
        if response[0] == cmd {
            self.transfer(&mut [data_mark])?;
            self.transfer(data)?;
            response[0] = 0;
            retry_while!(response[0] != 0xc3, retries = 10, {
                self.transfer(&mut response[0..1])?;
            });
        }
        Ok(())
    }
}

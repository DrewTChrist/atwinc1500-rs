use crate::crc::crc7;
use crate::error::Error;
use embedded_hal::blocking::spi::Transfer;
use embedded_hal::digital::v2::OutputPin;

pub mod commands {
    // Command start + Command Type
    // Command start = 0b1100
    // Example: 0b1100 | 0b0001 into one byte
    // CMD_DMA_WRITE = 0b11000001
    pub const CMD_DMA_WRITE: u8 = 0xc1; // type B
    pub const CMD_DMA_READ: u8 = 0xc2; // type B
    pub const CMD_INTERNAL_WRITE: u8 = 0xc3; // type C
    pub const CMD_INTERNAL_READ: u8 = 0xc4; // type A
    pub const CMD_TERMINATE: u8 = 0xc5; // type A
    pub const CMD_REPEAT: u8 = 0xc6; // type A
    pub const CMD_DMA_EXT_WRITE: u8 = 0xc7; // type C
    pub const CMD_DMA_EXT_READ: u8 = 0xc8; // type C
    pub const CMD_SINGLE_WRITE: u8 = 0xc9; // type D
    pub const CMD_SINGLE_READ: u8 = 0xca; // type A
    pub const CMD_RESET: u8 = 0xcf; // type A
}

mod sizes {
    pub const CRC_BIT: usize = 1;
    pub const RESPONSE: usize = 2;
    pub const DATA_START: usize = 1;
    pub const DATA: usize = 4;
    // Command size without crc bit
    pub const TYPE_A: usize = 4;
    pub const TYPE_B: usize = 6;
    pub const TYPE_C: usize = 7;
    pub const TYPE_D: usize = 8;
    // Full command packet size with crc bit
    pub const TYPE_A_CRC: usize = TYPE_A + CRC_BIT;
    pub const _TYPE_B_CRC: usize = TYPE_B + CRC_BIT;
    pub const TYPE_C_CRC: usize = TYPE_C + CRC_BIT;
    pub const TYPE_D_CRC: usize = TYPE_D + CRC_BIT;
}

/// The SpiBus struct
/// handles all reads/writes that
/// happen over the FullDuplex spi bus
pub struct SpiBus<SPI, O>
where
    SPI: Transfer<u8>,
    O: OutputPin,
{
    spi: SPI,
    cs: O,
    crc: bool,
    crc_disabled: bool,
}

impl<SPI, O> SpiBus<SPI, O>
where
    SPI: Transfer<u8>,
    O: OutputPin,
{
    pub fn new(spi: SPI, cs: O, crc: bool) -> Self {
        Self {
            spi,
            cs,
            crc,
            crc_disabled: false,
        }
    }

    /// Pulls the chip select high
    /// as it is active low
    pub fn init_cs(&mut self) -> Result<(), Error> {
        match self.cs.set_high() {
            Ok(_) => Ok(()),
            Err(_) => Err(Error::PinStateError),
        }
    }

    /// Sets crc_disabled to true
    pub fn crc_disabled(&mut self) -> Result<(), Error> {
        self.crc_disabled = true;
        Ok(())
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
        let mut crc_index: usize = 0;
        match command {
            commands::CMD_DMA_WRITE => {}
            commands::CMD_DMA_READ => {
                cmd_buffer[1] = (address >> 16) as u8;
                cmd_buffer[2] = (address >> 8) as u8;
                cmd_buffer[3] = address as u8;
                cmd_buffer[4] = (size >> 8) as u8;
                cmd_buffer[5] = size as u8;
                crc_index = sizes::TYPE_B;
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
                crc_index = sizes::TYPE_C;
            }
            commands::CMD_INTERNAL_READ => {
                cmd_buffer[1] = (address >> 8) as u8;
                if clockless {
                    cmd_buffer[1] |= 1 << 7;
                }
                cmd_buffer[2] = address as u8;
                cmd_buffer[3] = 0;
                crc_index = sizes::TYPE_A;
            }
            commands::CMD_TERMINATE => {
                cmd_buffer[1] = 0x0;
                cmd_buffer[2] = 0x0;
                cmd_buffer[3] = 0x0;
                crc_index = sizes::TYPE_A;
            }
            commands::CMD_REPEAT => {
                cmd_buffer[1] = 0x0;
                cmd_buffer[2] = 0x0;
                cmd_buffer[3] = 0x0;
                crc_index = sizes::TYPE_A;
            }
            commands::CMD_DMA_EXT_WRITE => {
                cmd_buffer[1] = (address >> 16) as u8;
                cmd_buffer[2] = (address >> 8) as u8;
                cmd_buffer[3] = address as u8;
                cmd_buffer[4] = (size >> 16) as u8;
                cmd_buffer[5] = (size >> 8) as u8;
                cmd_buffer[6] = size as u8;
                crc_index = 0;
            }
            commands::CMD_DMA_EXT_READ => {
                cmd_buffer[1] = (address >> 16) as u8;
                cmd_buffer[2] = (address >> 8) as u8;
                cmd_buffer[3] = address as u8;
                cmd_buffer[4] = (size >> 16) as u8;
                cmd_buffer[5] = (size >> 8) as u8;
                cmd_buffer[6] = size as u8;
                crc_index = 0;
            }
            commands::CMD_SINGLE_WRITE => {
                cmd_buffer[1] = (address >> 16) as u8;
                cmd_buffer[2] = (address >> 8) as u8;
                cmd_buffer[3] = address as u8;
                cmd_buffer[4] = (data >> 24) as u8;
                cmd_buffer[5] = (data >> 16) as u8;
                cmd_buffer[6] = (data >> 8) as u8;
                cmd_buffer[7] = data as u8;
                crc_index = sizes::TYPE_D;
            }
            commands::CMD_SINGLE_READ => {
                cmd_buffer[1] = (address >> 16) as u8;
                cmd_buffer[2] = (address >> 8) as u8;
                cmd_buffer[3] = address as u8;
                crc_index = sizes::TYPE_A;
            }
            commands::CMD_RESET => {
                cmd_buffer[1] = 0xff;
                cmd_buffer[2] = 0xff;
                cmd_buffer[3] = 0xff;
                crc_index = sizes::TYPE_A;
            }
            _ => {
                return Err(Error::InvalidSpiCommandError);
            }
        }
        if self.crc || !self.crc_disabled {
            cmd_buffer[crc_index] = crc7(0x7f, &cmd_buffer[0..crc_index]) << 1;
        }
        self.transfer(cmd_buffer)?;
        Ok(())
    }

    /// Wraps the read_reg method to pass it the size
    /// of the command buffer based on crc being enabled
    pub fn read_register(&mut self, address: u32) -> Result<u32, Error> {
        match self.crc_disabled {
            true => {
                const SIZE: usize =
                    sizes::TYPE_A + sizes::RESPONSE + sizes::DATA_START + sizes::DATA;
                // 7..11 is the range of the data returned from the atwinc
                // when crc is disabled and 4 is where the response from
                // the atwinc starts
                Ok(self.read_reg::<SIZE>(address, 7, 11, 4)?)
            }
            false => {
                const SIZE: usize =
                    sizes::TYPE_A_CRC + sizes::RESPONSE + sizes::DATA_START + sizes::DATA;
                // 8..12 is the range of the data returned from the atwinc
                // when crc is enabled and 5 is where the response from
                // the atwinc starts
                Ok(self.read_reg::<SIZE>(address, 8, 12, 5)?)
            }
        }
    }

    /// Reads a value from a register at a given address
    /// and returns it
    fn read_reg<const S: usize>(
        &mut self,
        address: u32,
        beg: usize,
        end: usize,
        response_start: usize,
    ) -> Result<u32, Error> {
        let cmd: u8;
        let clockless: bool;
        let mut cmd_buffer: [u8; S] = [0; S];
        // The Atmel driver does a clockless read
        // if address is less than 0xff (0b11111111).
        if address <= 0xff {
            cmd = commands::CMD_INTERNAL_READ;
            clockless = true;
        } else {
            cmd = commands::CMD_SINGLE_READ;
            clockless = false;
        }
        self.command(&mut cmd_buffer, cmd, address, 0, 0, clockless)?;
        if cmd_buffer[response_start] != cmd || cmd_buffer[response_start + 2] & 0xf0 != 0xf0 {
            return Err(Error::SpiReadRegisterError);
        }
        Ok(combine_bytes_lsb!(cmd_buffer[beg..end]))
    }

    /// Wraps the read method to change the command buffer size
    /// depending on crc being enabled or not
    pub fn read_data(&mut self, data: &mut [u8], address: u32, count: u32) -> Result<(), Error> {
        match self.crc_disabled {
            true => {
                const SIZE: usize = sizes::TYPE_C;
                Ok(self.read::<SIZE>(data, address, count)?)
            }
            false => {
                const SIZE: usize = sizes::TYPE_C_CRC;
                Ok(self.read::<SIZE>(data, address, count)?)
            }
        }
    }

    /// Reads a block of data
    fn read<const S: usize>(
        &mut self,
        data: &mut [u8],
        address: u32,
        count: u32,
    ) -> Result<(), Error> {
        let cmd: u8 = commands::CMD_DMA_EXT_READ;
        let mut cmd_buffer: [u8; S] = [0; S];
        let mut response: [u8; sizes::RESPONSE + sizes::DATA_START] =
            [0; sizes::RESPONSE + sizes::DATA_START];
        self.command(&mut cmd_buffer, cmd, address, 0, count, false)?;
        retry_while!(response[0] == 0, retries = 10, {
            self.transfer(&mut response)?;
        });
        if response[0] == cmd {
            self.transfer(data)?;
        }
        Ok(())
    }

    /// Wraps the read_reg method to pass it the size
    /// of the command buffer based on crc being enabled
    pub fn write_register(&mut self, address: u32, data: u32) -> Result<(), Error> {
        match self.crc_disabled {
            // response starts at index 8
            true => {
                const SIZE: usize = sizes::TYPE_D + sizes::RESPONSE;
                Ok(self.write_reg::<SIZE>(address, data, 8)?)
            }
            // response starts at index 9
            false => {
                const SIZE: usize = sizes::TYPE_D_CRC + sizes::RESPONSE;
                Ok(self.write_reg::<SIZE>(address, data, 9)?)
            }
        }
    }

    /// Writes a value to a register at a given address
    fn write_reg<const S: usize>(
        &mut self,
        address: u32,
        data: u32,
        response_start: usize,
    ) -> Result<(), Error> {
        let cmd: u8;
        let clockless: bool;
        let mut cmd_buffer: [u8; S] = [0; S];
        // The Atmel driver does a clockless write
        // if address is less than 0x30 (0b00110000).
        if address <= 0x30 {
            cmd = commands::CMD_INTERNAL_WRITE;
            clockless = true;
        } else {
            cmd = commands::CMD_SINGLE_WRITE;
            clockless = false;
        }
        self.command(&mut cmd_buffer, cmd, address, data, 0, clockless)?;
        if cmd_buffer[response_start] != cmd || cmd_buffer[response_start + 1] != 0 {
            return Err(Error::SpiWriteRegisterError);
        }
        Ok(())
    }

    /// Wraps the write method to change the command buffer size
    /// depending on crc being enabled or not
    pub fn write_data(&mut self, data: &mut [u8], address: u32, count: u32) -> Result<(), Error> {
        match self.crc_disabled {
            true => {
                const SIZE: usize = sizes::TYPE_C;
                Ok(self.write::<SIZE>(data, address, count)?)
            }
            false => {
                const SIZE: usize = sizes::TYPE_C_CRC;
                Ok(self.write::<SIZE>(data, address, count)?)
            }
        }
    }

    /// Writes a block of data to the atwinc1500
    fn write<const S: usize>(
        &mut self,
        data: &mut [u8],
        address: u32,
        count: u32,
    ) -> Result<(), Error> {
        let cmd: u8 = commands::CMD_DMA_EXT_WRITE;
        let mut cmd_buffer: [u8; S] = [0; S];
        let mut response: [u8; sizes::RESPONSE] = [0; sizes::RESPONSE];
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

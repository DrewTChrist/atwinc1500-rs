use crate::error::Error;

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

// Full command packet size with crc bit
pub const A_SIZE: usize = 5;
pub const B_SIZE: usize = 7;
pub const C_SIZE: usize = 8;
pub const D_SIZE: usize = 9;

#[derive(Debug)]
pub enum SpiCommand<'b, const S: usize> {
    A(&'b mut [u8; S]),
    B(&'b mut [u8; S]),
    C(&'b mut [u8; S]),
    D(&'b mut [u8; S]),
}

impl<'b, const S: usize> SpiCommand<'b, S> {
    pub fn new(buffer: &'b mut [u8; S], cmd: u8, addr: u32, data: u32) -> Result<Self, Error> {
        let command: SpiCommand<S>;
        match cmd {
            CMD_DMA_WRITE => {
                command = SpiCommand::D(buffer);
                Ok(command)
            }
            CMD_DMA_READ => {
                command = SpiCommand::A(buffer);
                Ok(command)
            }
            CMD_INTERNAL_WRITE => {
                command = SpiCommand::C(buffer);
                Ok(command)
            }
            CMD_INTERNAL_READ => {
                command = SpiCommand::A(buffer);
                Ok(command)
            }
            CMD_TERMINATE => {
                command = SpiCommand::A(buffer);
                Ok(command)
            }
            CMD_REPEAT => {
                command = SpiCommand::A(buffer);
                Ok(command)
            }
            CMD_DMA_EXT_WRITE => {
                command = SpiCommand::C(buffer);
                Ok(command)
            }
            CMD_DMA_EXT_READ => {
                command = SpiCommand::C(buffer);
                Ok(command)
            }
            CMD_SINGLE_WRITE => {
                command = SpiCommand::B(buffer);
                Ok(command)
            }
            CMD_SINGLE_READ => {
                command = SpiCommand::B(buffer);
                Ok(command)
            }
            CMD_RESET => {
                command = SpiCommand::A(buffer);
                Ok(command)
            }
            _ => Err(Error::InvalidSpiCommandError),
        }
    }
}

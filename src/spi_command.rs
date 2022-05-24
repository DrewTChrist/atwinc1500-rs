use crate::error::Error;

#[rustfmt::skip]
pub mod constants {
    // Command start + Command Type
    // Example: 0b11000000 | 0b00000001
    // CMD_DMA_WRITE = 0b11000001
    pub const CMD_DMA_WRITE:      u8 = 0xc1;
    pub const CMD_DMA_READ:       u8 = 0xc2;
    pub const CMD_INTERNAL_WRITE: u8 = 0xc3;
    pub const CMD_INTERNAL_READ:  u8 = 0xc4;
    pub const CMD_TERMINATE:      u8 = 0xc5;
    pub const CMD_REPEAT:         u8 = 0xc6;
    pub const CMD_DMA_EXT_WRITE:  u8 = 0xc7;
    pub const CMD_DMA_EXT_READ:   u8 = 0xc8;
    pub const CMD_SINGLE_WRITE:   u8 = 0xc9;
    pub const CMD_SINGLE_READ:    u8 = 0xca;
    pub const CMD_RESET:          u8 = 0xcf;
}

// Full command packet size with crc bit
const A_SIZE: usize = 5;
const B_SIZE: usize = 7;
const C_SIZE: usize = 8;
const D_SIZE: usize = 9;

#[derive(Debug)]
enum PayloadType {
    A([u8; A_SIZE]),
    B([u8; B_SIZE]),
    C([u8; C_SIZE]),
    D([u8; D_SIZE]),
}

#[derive(Debug)]
pub struct SpiCommand {
    payload: PayloadType,
}

impl SpiCommand {
    pub fn new(cmd: u8, addr: u32, data: u32) -> Result<Self, Error> {
        match cmd {
            constants::CMD_DMA_WRITE => Ok(SpiCommand {
                payload: PayloadType::D([cmd, 0, 0, 0, 0, 0, 0, 0, 0]),
            }),
            constants::CMD_DMA_READ => Ok(SpiCommand {
                payload: PayloadType::A([cmd, 0, 0, 0, 0]),
            }),
            constants::CMD_INTERNAL_WRITE => Ok(SpiCommand {
                payload: PayloadType::C([cmd, 0, 0, 0, 0, 0, 0, 0]),
            }),
            constants::CMD_INTERNAL_READ => Ok(SpiCommand {
                payload: PayloadType::A([cmd, 0, 0, 0, 0]),
            }),
            constants::CMD_TERMINATE => Ok(SpiCommand {
                payload: PayloadType::A([cmd, 0, 0, 0, 0]),
            }),
            constants::CMD_REPEAT => Ok(SpiCommand {
                payload: PayloadType::A([cmd, 0, 0, 0, 0]),
            }),
            constants::CMD_DMA_EXT_WRITE => Ok(SpiCommand {
                payload: PayloadType::C([cmd, 0, 0, 0, 0, 0, 0, 0]),
            }),
            constants::CMD_DMA_EXT_READ => Ok(SpiCommand {
                payload: PayloadType::C([cmd, 0, 0, 0, 0, 0, 0, 0]),
            }),
            constants::CMD_SINGLE_WRITE => Ok(SpiCommand {
                payload: PayloadType::B([cmd, 0, 0, 0, 0, 0, 0]),
            }),
            constants::CMD_SINGLE_READ => Ok(SpiCommand {
                payload: PayloadType::B([cmd, 0, 0, 0, 0, 0, 0]),
            }),
            constants::CMD_RESET => Ok(SpiCommand {
                payload: PayloadType::A([cmd, 0xFF, 0xFF, 0xFF, 0]),
            }),
            _ => Err(Error::InvalidSpiCommandError),
        }
    }

    pub fn buffer(&self) -> &[u8] {
        match self.payload {
            PayloadType::A(p) => &p,
            PayloadType::B(p) => &p,
            PayloadType::C(p) => &p,
            PayloadType::D(p) => &p,
        }
    }
}

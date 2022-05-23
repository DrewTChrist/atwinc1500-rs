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

#[derive(Debug)]
enum PayloadType {
    A([u8; 3]),
    B([u8; 5]),
    C([u8; 6]),
    D([u8; 7]),
}

#[derive(Debug)]
pub struct SpiCommand {
    cmd: u8,
    payload: PayloadType,
    crc: u8,
}

impl SpiCommand {
    pub fn new(cmd: u8, addr: u32, data: u32) -> Result<Self, Error> {
        match cmd {
            constants::CMD_DMA_WRITE => Ok(SpiCommand {
                cmd,
                payload: PayloadType::D([0, 0, 0, 0, 0, 0, 0]),
                crc: 0,
            }),
            constants::CMD_DMA_READ => Ok(SpiCommand {
                cmd,
                payload: PayloadType::A([0, 0, 0]),
                crc: 0,
            }),
            constants::CMD_INTERNAL_WRITE => Ok(SpiCommand {
                cmd,
                payload: PayloadType::C([0, 0, 0, 0, 0, 0]),
                crc: 0,
            }),
            constants::CMD_INTERNAL_READ => Ok(SpiCommand {
                cmd,
                payload: PayloadType::A([0, 0, 0]),
                crc: 0,
            }),
            constants::CMD_TERMINATE => Ok(SpiCommand {
                cmd,
                payload: PayloadType::A([0, 0, 0]),
                crc: 0,
            }),
            constants::CMD_REPEAT => Ok(SpiCommand {
                cmd,
                payload: PayloadType::A([0, 0, 0]),
                crc: 0,
            }),
            constants::CMD_DMA_EXT_WRITE => Ok(SpiCommand {
                cmd,
                payload: PayloadType::C([0, 0, 0, 0, 0, 0]),
                crc: 0,
            }),
            constants::CMD_DMA_EXT_READ => Ok(SpiCommand {
                cmd,
                payload: PayloadType::C([0, 0, 0, 0, 0, 0]),
                crc: 0,
            }),
            constants::CMD_SINGLE_WRITE => Ok(SpiCommand {
                cmd,
                payload: PayloadType::B([0, 0, 0, 0, 0]),
                crc: 0,
            }),
            constants::CMD_SINGLE_READ => Ok(SpiCommand {
                cmd,
                payload: PayloadType::B([0, 0, 0, 0, 0]),
                crc: 0,
            }),
            constants::CMD_RESET => Ok(SpiCommand {
                cmd,
                payload: PayloadType::A([0xFF, 0xFF, 0xFF]),
                crc: 0,
            }),
            _ => Err(Error::InvalidSpiCommandError),
        }
    }

    pub fn to_slice(&self) -> &[u8] {
        todo!()
    }
}

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

    pub mod sizes {
        // Full command packet size with crc bit
        pub const TYPE_A: usize = 5;
        pub const TYPE_B: usize = 7;
        pub const TYPE_C: usize = 8;
        pub const TYPE_D: usize = 9;
    }
}

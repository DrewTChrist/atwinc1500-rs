use crate::error::Error;

/// Defines the needed functions to handle the spi layer
/// as described in the atwinc1500 software design guide
pub trait SpiLayer {
    fn spi_transfer(
        &mut self,
        words: &'_ mut [u8],
        command_len: usize,
        response_len: usize,
    ) -> Result<(), Error>;
    fn spi_command(
        &mut self,
        cmd_buffer: &'_ mut [u8],
        command: u8,
        address: u32,
        data: u32,
        size: u32,
        clockless: bool,
    ) -> Result<(), Error>;
    fn spi_read_register(&mut self, cmd_buffer: &'_ mut [u8], address: u32) -> Result<(), Error>;
    fn spi_read_data(&mut self, cmd_buffer: &'_ mut [u8], address: u32) -> Result<(), Error>;
    fn spi_write_register(
        &mut self,
        cmd_buffer: &'_ mut [u8],
        address: u32,
        data: u32,
    ) -> Result<(), Error>;
    fn spi_write_data(
        &mut self,
        cmd_buffer: &'_ mut [u8],
        address: u32,
        data: u32,
    ) -> Result<(), Error>;
}

/// Defines the needed functions to handle the host interface
/// layer as described in the atwinc1500 software design guide
pub trait HifLayer {
    fn hif_chip_wake(&mut self) -> Result<(), Error>;
    fn hif_chip_sleep(&mut self) -> Result<(), Error>;
    fn hif_register_cb(&mut self) -> Result<(), Error>;
    fn hif_isr(&mut self) -> Result<(), Error>;
    fn hif_receive(&mut self) -> Result<(), Error>;
    fn hif_send(&mut self) -> Result<(), Error>;
    fn hif_set_sleep_mode(&mut self) -> Result<(), Error>;
    fn hif_get_sleep_mode(&mut self) -> Result<(), Error>;
}

use crate::error::Error;

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

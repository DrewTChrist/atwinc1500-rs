use core::fmt;
use defmt::Format;

#[derive(Debug, Format)]
pub enum Error {
    InvalidSpiCommandError,
    PinStateError,
    SpiTransferError,
    SpiWriteRegisterError,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Error::InvalidSpiCommandError => write!(f, "Invalid Spi Command"),
            Error::PinStateError => write!(f, "Pin State Error"),
            Error::SpiTransferError => write!(f, "Spi Transfer Error"),
            Error::SpiWriteRegisterError => write!(f, "Error writing to register"),
    
        }
    }
}

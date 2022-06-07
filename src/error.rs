use core::fmt;
use defmt::Format;

#[derive(Debug, Format)]
pub enum Error {
    SpiTransferError,
    InvalidSpiCommandError,
    PinStateError,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Error::SpiTransferError => write!(f, "Spi Transfer Error"),
            Error::InvalidSpiCommandError => write!(f, "Invalid Spi Command"),
            Error::PinStateError => write!(f, "Pin State Error"),
        }
    }
}

//! Atwinc1500 error definitions
use core::fmt;

// Derives defmt::Format if building for bare metal
// otherwise it does not derive defmt::Format
// Unit tests get a linker error if this isn't done
#[cfg_attr(target_os = "none", derive(PartialEq, Debug, defmt::Format))]
#[cfg_attr(not(target_os = "none"), derive(PartialEq, Debug))]
/// Atwinc1500 error types
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

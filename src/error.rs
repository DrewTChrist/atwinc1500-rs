//! Atwinc1500 error definitions
use core::fmt;

/// These are the error values defined
/// in the Atwinc data sheet. InvalidError is
/// a catch all for error values greater than
/// 5 that are not real errors. If InvalidError
/// is caught, then the responses are no longer
/// being read correctly. These errors should be
/// handled with the error recovery mechanisms
/// also defined in the data sheet.
#[repr(u8)]
#[derive(Eq, PartialEq, PartialOrd)]
pub enum AtwincSpiError {
    /// No error received from the Atwinc1500
    NoError = 0,
    /// Command sent to the Atwinc1500 is not valid
    UnsupportedCommand = 1,
    /// Data sent to the Atwinc1500 was not expected
    UnexpectedDataReceived = 2,
    /// Crc7 sent to the Atwinc1500 was invalid
    Crc7Error = 3,
    /// Crc16 sent to the Atwinc1500 was invalid
    Crc16Error = 4,
    /// Atwinc1500 experienced an internal error
    InternalError = 5,
    /// Catch all for invalid errors
    /// passed to From<u8>
    InvalidError,
}

impl From<u8> for AtwincSpiError {
    /// For easily converting a response byte
    /// to an SpiError type
    fn from(other: u8) -> Self {
        match other {
            0 => AtwincSpiError::NoError,
            1 => AtwincSpiError::UnsupportedCommand,
            2 => AtwincSpiError::UnexpectedDataReceived,
            3 => AtwincSpiError::Crc7Error,
            4 => AtwincSpiError::Crc16Error,
            5 => AtwincSpiError::InternalError,
            _ => AtwincSpiError::InvalidError,
        }
    }
}

// Derives defmt::Format if building for bare metal
// otherwise it does not derive defmt::Format
// Unit tests get a linker error if this isn't done
#[cfg_attr(target_os = "none", derive(Eq, PartialEq, Debug, defmt::Format))]
#[cfg_attr(not(target_os = "none"), derive(Eq, PartialEq, Debug))]
/// Spi error variants
pub enum SpiError {
    /// Attempted to parse an invalid spi command
    InvalidCommand,
    /// Error changing the state of a pin
    PinStateError,
    /// Error transferring data over the spi bus
    TransferError,
    /// Error reading data from the atwinc1500
    ReadDataError,
    /// Error received from the atwinc1500
    /// while trying to read from register
    ReadRegisterError,
    /// Error writing data to the atwinc1500
    WriteDataError,
    /// Error received from the atwinc1500
    /// while trying to write to register
    WriteRegisterError,
}

impl fmt::Display for SpiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            SpiError::InvalidCommand => write!(f, "Invalid Spi Command"),
            SpiError::PinStateError => write!(f, "Pin State Error"),
            SpiError::TransferError => write!(f, "Spi Transfer Error"),
            SpiError::ReadDataError => write!(f, "Error reading data"),
            SpiError::ReadRegisterError => write!(f, "Error reading from register"),
            SpiError::WriteDataError => write!(f, "Error writing data"),
            SpiError::WriteRegisterError => write!(f, "Error writing to register"),
        }
    }
}

#[cfg_attr(target_os = "none", derive(Eq, PartialEq, Debug, defmt::Format))]
#[cfg_attr(not(target_os = "none"), derive(Eq, PartialEq, Debug))]
/// Atwinc1500 error variants
pub enum Error {
    /// Error occurred during Spi interaction
    SpiError(SpiError),
    /// Error updating pin state
    PinStateError,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Error::SpiError(_) => write!(f, "Error"),
            Error::PinStateError => write!(f, "Error"),
        }
    }
}

impl From<SpiError> for Error {
    fn from(value: SpiError) -> Self {
        match value {
            SpiError::PinStateError => Self::PinStateError,
            _ => Self::SpiError(value),
        }
    }
}

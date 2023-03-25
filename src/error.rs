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
#[cfg_attr(target_os = "none", derive(Eq, PartialEq, Debug, defmt::Format))]
#[cfg_attr(not(target_os = "none"), derive(Eq, PartialEq, Debug))]
pub enum SpiCommandError {
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

impl From<u8> for SpiCommandError {
    /// For easily converting a response byte
    /// to an SpiError type
    fn from(other: u8) -> Self {
        match other {
            0 => SpiCommandError::NoError,
            1 => SpiCommandError::UnsupportedCommand,
            2 => SpiCommandError::UnexpectedDataReceived,
            3 => SpiCommandError::Crc7Error,
            4 => SpiCommandError::Crc16Error,
            5 => SpiCommandError::InternalError,
            _ => SpiCommandError::InvalidError,
        }
    }
}

/// Host Interface error variants
#[cfg_attr(target_os = "none", derive(Eq, PartialEq, Debug, defmt::Format))]
#[cfg_attr(not(target_os = "none"), derive(Eq, PartialEq, Debug))]
pub enum HifError {
    /// App requested data buffer was larger than the data buffer received
    /// from the Atwinc1500
    SizeMismatch(usize, usize),
    /// App requested data from an address beyond that of the received data
    AddressMismatch(u32, u32),
}

impl fmt::Display for HifError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            HifError::SizeMismatch(app_size, data_size) => write!(
                f,
                "App requested ({} bytes) more data than received ({} bytes)",
                app_size, data_size
            ),
            HifError::AddressMismatch(app_size, data_size) => write!(
                f,
                "App requested ({} bytes) more data than received ({} bytes)",
                app_size, data_size
            ),
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
    InvalidCommand(u8),
    /// Error changing the state of a pin
    PinStateError,
    /// Error transferring data over the spi bus
    TransferError,
    /// Error reading data from the atwinc1500
    ReadDataError(u8, SpiCommandError),
    /// Error received from the atwinc1500
    /// while trying to read from register
    ReadRegisterError(u8, SpiCommandError, u8),
    /// Error writing data to the atwinc1500
    WriteDataError(u8, SpiCommandError),
    /// Error received from the atwinc1500
    /// while trying to write to register
    WriteRegisterError(u8, SpiCommandError),
}

impl fmt::Display for SpiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SpiError::InvalidCommand(cmd) => write!(f, "Invalid Spi Command: {:#04x}", cmd),
            SpiError::PinStateError => write!(f, "Pin State Error"),
            SpiError::TransferError => write!(f, "Spi Transfer Error"),
            SpiError::ReadDataError(cmd, spi_error) => write!(
                f,
                "Error reading data {{cmd: {}, err: {:?}}}",
                cmd, spi_error
            ),
            SpiError::ReadRegisterError(cmd, spi_error, pkt) => write!(
                f,
                "Error reading from register {{cmd: {:#04x}, err: {:?}, pkt: {:#04x}}}",
                cmd, spi_error, pkt
            ),
            SpiError::WriteDataError(cmd, spi_error) => write!(
                f,
                "Error writing data {{cmd: {}, err: {:?}}}",
                cmd, spi_error
            ),
            SpiError::WriteRegisterError(cmd, spi_error) => write!(
                f,
                "Error writing to register {{cmd: {}, err: {:?}}}",
                cmd, spi_error
            ),
        }
    }
}

#[cfg_attr(target_os = "none", derive(Eq, PartialEq, Debug, defmt::Format))]
#[cfg_attr(not(target_os = "none"), derive(Eq, PartialEq, Debug))]
/// Atwinc1500 error variants
pub enum Error {
    /// Error occured during Hif interaction
    HifError(HifError),
    /// Error occurred during Spi interaction
    SpiError(SpiError),
    /// Error updating pin state
    PinStateError,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Error::HifError(_) => write!(f, "Error"),
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

impl From<HifError> for Error {
    fn from(value: HifError) -> Self {
        Self::HifError(value)
    }
}

//! Atwinc1500 error definitions

/// These are the error values defined
/// in the Atwinc data sheet. InvalidError is
/// a catch all for error values greater than
/// 5 that are not real errors. If InvalidError
/// is caught, then the responses are no longer
/// being read correctly. These errors should be
/// handled with the error recovery mechanisms
/// also defined in the data sheet.
#[repr(u8)]
#[derive(Clone, Copy, Eq, PartialEq, core::fmt::Debug, defmt::Format)]
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
    /// passed to `From<u8>`
    InvalidError,
}

impl From<u8> for SpiCommandError {
    /// For easily converting a response byte
    /// to an SpiCommandError type
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

impl PartialEq<SpiCommandError> for u8 {
    /// Allows for directly comparing a byte
    /// to an SpiCommandError without casting to
    /// u8 in the comparison
    fn eq(&self, other: &SpiCommandError) -> bool {
        *self == *other as u8
    }
}

/// Host Interface error variants
#[derive(Eq, PartialEq, core::fmt::Debug)]
pub enum HifError {
    /// App requested data buffer was larger than the data buffer received
    /// from the Atwinc1500
    SizeMismatch(usize, usize),
    /// App requested data from an address beyond that of the received data
    AddressMismatch(u32, u32),
}

impl defmt::Format for HifError {
    fn format(&self, f: defmt::Formatter) {
        match *self {
            HifError::SizeMismatch(app_size, data_size) => defmt::write!(
                f,
                "App requested ({} bytes) more data than received ({} bytes)",
                app_size,
                data_size
            ),
            HifError::AddressMismatch(app_size, data_size) => defmt::write!(
                f,
                "App requested ({} bytes) more data than received ({} bytes)",
                app_size,
                data_size
            ),
        }
    }
}

type Command = u8;
type Address = u32;

/// Spi error variants
#[derive(Eq, PartialEq, core::fmt::Debug)]
pub enum SpiError {
    /// Error changing the state of a pin
    PinStateError,
    /// Error transferring data over the spi bus
    TransferError,
    /// Error reading data from the atwinc1500
    ReadDataError(Command, Address, SpiCommandError),
    /// Error received from the atwinc1500
    /// while trying to read from register
    ReadRegisterError(Command, Address, SpiCommandError, u8),
    /// Error writing data to the atwinc1500
    WriteDataError(Command, Address, SpiCommandError),
    /// Error received from the atwinc1500
    /// while trying to write to register
    WriteRegisterError(Command, Address, SpiCommandError),
}

impl defmt::Format for SpiError {
    fn format(&self, f: defmt::Formatter) {
        match self {
            SpiError::PinStateError => defmt::write!(f, "Pin State Error"),
            SpiError::TransferError => defmt::write!(f, "Spi Transfer Error"),
            SpiError::ReadDataError(cmd, address, spi_error) => defmt::write!(
                f,
                "Error reading data {{cmd: {:#04x}, addr: {:#04x}, err: {:?}}}",
                cmd,
                address,
                spi_error
            ),
            SpiError::ReadRegisterError(cmd, address, spi_error, pkt) => defmt::write!(
                f,
                "Error reading from register {{cmd: {:#04x}, addr: {:#04x}, err: {:?}, pkt: {:#04x}}}",
                cmd,
                address,
                spi_error,
                pkt
            ),
            SpiError::WriteDataError(cmd, address, spi_error) => defmt::write!(
                f,
                "Error writing data {{cmd: {:#04x}, addr: {:#04x}, err: {:?}}}",
                cmd,
                address,
                spi_error
            ),
            SpiError::WriteRegisterError(cmd, address, spi_error) => defmt::write!(
                f,
                "Error writing to register {{cmd: {:#04x}, addr: {:#04x}, err: {:?}}}",
                cmd,
                address,
                spi_error
            ),
        }
    }
}

/// Network scan error variants
#[derive(Eq, PartialEq, core::fmt::Debug, defmt::Format)]
pub enum ScanError {
    /// There is already a network
    /// scan in progress
    ScanInProgress,
    /// The scan result index
    /// is outside the range of
    /// valid indexes
    IndexOutOfRange,
}

/// Atwinc1500 error variants
#[derive(Eq, PartialEq, core::fmt::Debug, defmt::Format)]
pub enum Error {
    /// Error occured during Hif interaction
    HifError(HifError),
    /// Error occurred during network scan
    ScanError(ScanError),
    /// Error occurred during Spi interaction
    SpiError(SpiError),
    /// Error updating pin state
    PinStateError,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::HifError(hif_error) => write!(f, "{:?}", hif_error),
            Error::ScanError(scan_error) => write!(f, "{:?}", scan_error),
            Error::SpiError(spi_error) => write!(f, "{:?}", spi_error),
            Error::PinStateError => write!(f, "Pin State Error"),
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

//! Atwinc1500 gpio related members

/// Gpio pin definitions
pub enum AtwincGpio {
    /// Gpio pin 3
    Gpio3 = 3,
    /// Gpio pin 4
    Gpio4 = 4,
    /// Gpio pin 5
    Gpio5 = 5,
    /// Gpio pin 6
    Gpio6 = 6,
}

#[derive(Eq, PartialEq)]
/// Gpio pin directions
pub enum GpioDirection {
    /// Input pin
    Input,
    /// Output pin
    Output,
}

impl From<u8> for GpioDirection {
    fn from(val: u8) -> Self {
        match val {
            1 => GpioDirection::Input,
            0 => GpioDirection::Output,
            _ => todo!(),
        }
    }
}

#[derive(Eq, PartialEq)]
/// Gpio pin values
pub enum GpioValue {
    /// Low logic level
    Low,
    /// High logic level
    High,
}

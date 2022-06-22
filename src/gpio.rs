pub enum AtwincGpio {
    Gpio3 = 3,
    Gpio4 = 4,
    Gpio5 = 5,
    Gpio6 = 6,
}

#[derive(PartialEq)]
pub enum GpioDirection {
    Input,
    Output,
}

impl From<u8> for GpioDirection {
    fn from(val: u8) -> Self {
        match val {
            1 => GpioDirection::Input,
            0 => GpioDirection::Output,
            _ => todo!()
        }
    }
}


#[derive(PartialEq)]
pub enum GpioValue {
    Low,
    High,
}

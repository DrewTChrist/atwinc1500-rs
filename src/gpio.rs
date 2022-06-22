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

#[derive(PartialEq)]
pub enum GpioValue {
    Low,
    High,
}

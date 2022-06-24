#[cfg(test)]
mod spi_unit_tests {
    use atwinc1500::error::Error;
    use atwinc1500::registers;
    use atwinc1500::spi;
    use embedded_hal_mock::pin::{
        Mock as PinMock, State as PinState, Transaction as PinTransaction,
    };
    use embedded_hal_mock::spi::{Mock as SpiMock, Transaction as SpiTransaction};
    use embedded_hal_mock::MockError;
    use std::io::ErrorKind;

    /// Returns an SpiBusWrapper with
    /// mocked spi and mocked chip select
    fn get_fixture(
        spi_expect: &[SpiTransaction],
        pin_expect: &[PinTransaction],
    ) -> spi::SpiBusWrapper<SpiMock, PinMock> {
        let spi = SpiMock::new(spi_expect);
        let cs = PinMock::new(pin_expect);
        spi::SpiBusWrapper::new(spi, cs)
    }

    #[test]
    fn test_init_cs_error() {
        let err = MockError::Io(ErrorKind::NotConnected);
        let spi_expect = [];
        let pin_expect = [PinTransaction::set(PinState::High).with_error(err.clone())];
        let mut spi_bus = get_fixture(&spi_expect, &pin_expect);
        assert!(spi_bus.init_cs().is_err());
    }

    #[test]
    fn test_init_cs_ok() {
        let spi_expect = [];
        let pin_expect = [PinTransaction::set(PinState::High)];
        let mut spi_bus = get_fixture(&spi_expect, &pin_expect);
        assert!(spi_bus.init_cs().is_ok());
    }

    #[test]
    fn test_read_register_bootrom() {
        // Simulates a read from the BOOTROM_REG
        // register as performed in the
        // Atwinc1500 initialize method
        const FINISH_BOOT_VAL: u32 = 0x10add09e;
        let address: u32 = registers::BOOTROM_REG;
        let spi_expect = [
            // Send
            SpiTransaction::transfer(
                vec![
                    spi::commands::CMD_SINGLE_READ,
                    (address >> 16) as u8,
                    (address >> 8) as u8,
                    address as u8,
                    0x0,
                    0x0,
                    0x0,
                    0x0,
                    0x0,
                    0x0,
                    0x0,
                    0x0,
                ],
                // Receive
                vec![
                    0x0,
                    0x0,
                    0x0,
                    0x0,
                    spi::commands::CMD_SINGLE_READ,
                    0x0,
                    0xf3,
                    (FINISH_BOOT_VAL & 0xff) as u8,
                    ((FINISH_BOOT_VAL >> 8) & 0xff) as u8,
                    ((FINISH_BOOT_VAL >> 16) & 0xff) as u8,
                    ((FINISH_BOOT_VAL >> 24) & 0xff) as u8,
                    0x0,
                ],
            ),
        ];
        let pin_expect = [
            PinTransaction::set(PinState::High),
            PinTransaction::set(PinState::Low),
            PinTransaction::set(PinState::High),
        ];
        let mut spi_bus = get_fixture(&spi_expect, &pin_expect);
        if spi_bus.init_cs().is_err() {
            assert!(false);
        }
        match spi_bus.read_register(registers::BOOTROM_REG) {
            Ok(v) => assert_eq!(v, FINISH_BOOT_VAL),
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn test_read_register_error() {
        // Simulates a register read
        // in which the Atwinc1500 returns
        // an error
        const FINISH_BOOT_VAL: u32 = 0x10add09e;
        let address: u32 = registers::BOOTROM_REG;
        let spi_expect = [
            // Send command
            SpiTransaction::transfer(
                vec![
                    spi::commands::CMD_SINGLE_READ,
                    (address >> 16) as u8,
                    (address >> 8) as u8,
                    address as u8,
                    0x0,
                    0x0,
                    0x0,
                    0x0,
                    0x0,
                    0x0,
                    0x0,
                    0x0,
                ],
                vec![
                    0x0,
                    0x0,
                    0x0,
                    0x0,
                    spi::commands::CMD_SINGLE_READ,
                    0x0,
                    0xee, // error caused here expects 0xf-
                    (FINISH_BOOT_VAL & 0xff) as u8,
                    ((FINISH_BOOT_VAL >> 8) & 0xff) as u8,
                    ((FINISH_BOOT_VAL >> 16) & 0xff) as u8,
                    ((FINISH_BOOT_VAL >> 24) & 0xff) as u8,
                    0x0,
                ],
            ),
        ];
        let pin_expect = [
            PinTransaction::set(PinState::High),
            PinTransaction::set(PinState::Low),
            PinTransaction::set(PinState::High),
        ];
        let mut spi_bus = get_fixture(&spi_expect, &pin_expect);
        if spi_bus.init_cs().is_err() {
            assert!(false);
        }
        match spi_bus.read_register(registers::BOOTROM_REG) {
            Ok(_) => assert!(false),
            Err(e) => assert_eq!(e, Error::SpiReadRegisterError),
        }
    }
}

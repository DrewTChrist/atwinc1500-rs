#[cfg(test)]
mod spi_unit_tests {
    use atwinc1500::error::SpiError;
    use atwinc1500::registers;
    use atwinc1500::spi;
    use embedded_hal_mock::pin::{
        Mock as PinMock, State as PinState, Transaction as PinTransaction,
    };
    use embedded_hal_mock::spi::{Mock as SpiMock, Transaction as SpiTransaction};
    use embedded_hal_mock::MockError;
    use std::io::ErrorKind;

    /// Returns an SpiBus with
    /// mocked spi and mocked chip select
    fn get_fixture(
        spi_expect: &[SpiTransaction],
        pin_expect: &[PinTransaction],
    ) -> spi::SpiBus<SpiMock, PinMock> {
        let spi = SpiMock::new(spi_expect);
        let cs = PinMock::new(pin_expect);
        let mut bus = spi::SpiBus::new(spi, cs, false);
        bus.crc_disabled().unwrap();
        bus
    }

    #[test]
    fn init_cs_error() {
        let err = MockError::Io(ErrorKind::NotConnected);
        let spi_expect = [];
        let pin_expect = [PinTransaction::set(PinState::High).with_error(err.clone())];
        let mut spi_bus = get_fixture(&spi_expect, &pin_expect);
        assert!(spi_bus.init_cs().is_err());
    }

    #[test]
    fn init_cs_ok() {
        let spi_expect = [];
        let pin_expect = [PinTransaction::set(PinState::High)];
        let mut spi_bus = get_fixture(&spi_expect, &pin_expect);
        assert!(spi_bus.init_cs().is_ok());
    }

    #[test]
    fn read_register_bootrom() {
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
    fn read_register_error() {
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
            Err(e) => assert_eq!(
                e,
                SpiError::ReadRegisterError(spi::commands::CMD_SINGLE_READ, 0.into(), 0xee,)
            ),
        }
    }

    #[test]
    fn read_register_crc() {
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
                    0xde, // crc byte goes here
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
                    0x0,
                    spi::commands::CMD_SINGLE_READ,
                    0x0,
                    0xf3,
                    (FINISH_BOOT_VAL & 0xff) as u8,
                    ((FINISH_BOOT_VAL >> 8) & 0xff) as u8,
                    ((FINISH_BOOT_VAL >> 16) & 0xff) as u8,
                    ((FINISH_BOOT_VAL >> 24) & 0xff) as u8,
                ],
            ),
        ];
        let pin_expect = [
            PinTransaction::set(PinState::High),
            PinTransaction::set(PinState::Low),
            PinTransaction::set(PinState::High),
        ];
        let spi = SpiMock::new(&spi_expect);
        let cs = PinMock::new(&pin_expect);
        let mut spi_bus = spi::SpiBus::new(spi, cs, true);
        if spi_bus.init_cs().is_err() {
            assert!(false);
        }
        match spi_bus.read_register(registers::BOOTROM_REG) {
            Ok(v) => assert_eq!(v, FINISH_BOOT_VAL),
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn write_register_bootrom() {
        let address: u32 = registers::BOOTROM_REG;
        const START_FIRMWARE: u32 = 0xef522f61;
        let spi_expect = [SpiTransaction::transfer(
            vec![
                spi::commands::CMD_SINGLE_WRITE,
                (address >> 16) as u8,
                (address >> 8) as u8,
                address as u8,
                (START_FIRMWARE >> 24) as u8,
                (START_FIRMWARE >> 16) as u8,
                (START_FIRMWARE >> 8) as u8,
                START_FIRMWARE as u8,
                0x0,
                0x0,
            ],
            vec![
                0x0,
                0x0,
                0x0,
                0x0,
                0x0,
                0x0,
                0x0,
                0x0,
                spi::commands::CMD_SINGLE_WRITE,
                0x0,
            ],
        )];
        let pin_expect = [
            PinTransaction::set(PinState::High),
            PinTransaction::set(PinState::Low),
            PinTransaction::set(PinState::High),
        ];
        let mut spi_bus = get_fixture(&spi_expect, &pin_expect);
        if spi_bus.init_cs().is_err() {
            assert!(false);
        }
        assert!(spi_bus
            .write_register(registers::BOOTROM_REG, START_FIRMWARE)
            .is_ok());
    }

    #[test]
    fn write_register_error() {
        let address: u32 = registers::BOOTROM_REG;
        const START_FIRMWARE: u32 = 0xef522f61;
        let spi_expect = [SpiTransaction::transfer(
            vec![
                spi::commands::CMD_SINGLE_WRITE,
                (address >> 16) as u8,
                (address >> 8) as u8,
                address as u8,
                (START_FIRMWARE >> 24) as u8,
                (START_FIRMWARE >> 16) as u8,
                (START_FIRMWARE >> 8) as u8,
                START_FIRMWARE as u8,
                0x0,
                0x0,
            ],
            vec![
                0x0,
                0x0,
                0x0,
                0x0,
                0x0,
                0x0,
                0x0,
                0x0,
                spi::commands::CMD_SINGLE_WRITE,
                0xff, // error caused here
            ],
        )];
        let pin_expect = [
            PinTransaction::set(PinState::High),
            PinTransaction::set(PinState::Low),
            PinTransaction::set(PinState::High),
        ];
        let mut spi_bus = get_fixture(&spi_expect, &pin_expect);
        if spi_bus.init_cs().is_err() {
            assert!(false);
        }
        match spi_bus.write_register(registers::BOOTROM_REG, START_FIRMWARE) {
            Ok(_) => assert!(false),
            Err(e) => assert_eq!(
                e,
                SpiError::WriteRegisterError(spi::commands::CMD_SINGLE_WRITE, 0xff.into())
            ),
        }
    }

    #[test]
    fn write_register_crc() {
        let address: u32 = registers::BOOTROM_REG;
        const START_FIRMWARE: u32 = 0xef522f61;
        let spi_expect = [SpiTransaction::transfer(
            vec![
                spi::commands::CMD_SINGLE_WRITE,
                (address >> 16) as u8,
                (address >> 8) as u8,
                address as u8,
                (START_FIRMWARE >> 24) as u8,
                (START_FIRMWARE >> 16) as u8,
                (START_FIRMWARE >> 8) as u8,
                START_FIRMWARE as u8,
                0xd8, // crc byte here
                0x0,
                0x0,
            ],
            vec![
                0x0,
                0x0,
                0x0,
                0x0,
                0x0,
                0x0,
                0x0,
                0x0,
                0x0,
                spi::commands::CMD_SINGLE_WRITE,
                0x0,
            ],
        )];
        let pin_expect = [
            PinTransaction::set(PinState::High),
            PinTransaction::set(PinState::Low),
            PinTransaction::set(PinState::High),
        ];
        let spi = SpiMock::new(&spi_expect);
        let cs = PinMock::new(&pin_expect);
        let mut spi_bus = spi::SpiBus::new(spi, cs, true);
        if spi_bus.init_cs().is_err() {
            assert!(false);
        }
        assert!(spi_bus
            .write_register(registers::BOOTROM_REG, START_FIRMWARE)
            .is_ok());
    }
}

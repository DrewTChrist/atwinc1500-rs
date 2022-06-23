#[cfg(test)]
mod spi_unit_tests {
    use atwinc1500::spi::SpiBusWrapper;
    use std::io::ErrorKind;
    use embedded_hal_mock::pin::{
        Mock as PinMock, State as PinState, Transaction as PinTransaction,
    };
    use embedded_hal_mock::spi::{Mock as SpiMock, Transaction as SpiTransaction};
    use embedded_hal_mock::MockError;

    /// Returns an SpiBusWrapper with
    /// mocked spi and mocked chip select
    fn get_fixture(
        spi_expect: &[SpiTransaction],
        pin_expect: &[PinTransaction],
    ) -> SpiBusWrapper<SpiMock, PinMock> {
        let spi = SpiMock::new(spi_expect);
        let cs = PinMock::new(pin_expect);
        SpiBusWrapper::new(spi, cs)
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
}

#[cfg(test)]
mod tests {
    use atwinc1500::error;
    use atwinc1500::spi::SpiBusWrapper;
    use std::io::ErrorKind;
    //use embedded_hal::digital::v2::OutputPin;
    //use embedded_hal::spi::FullDuplex;
    use embedded_hal_mock::pin::{
        Mock as PinMock, State as PinState, Transaction as PinTransaction,
    };
    use embedded_hal_mock::spi::{Mock as SpiMock, Transaction as SpiTransaction};
    use embedded_hal_mock::MockError;

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
        match spi_bus.init_cs() {
            Ok(()) => {}
            Err(e) => assert_eq!(e, error::Error::PinStateError),
        }
    }

    #[test]
    fn test_init_cs_ok() {
        let spi_expect = [];
        let pin_expect = [PinTransaction::set(PinState::High)];
        let mut spi_bus = get_fixture(&spi_expect, &pin_expect);
        assert!(spi_bus.init_cs().is_ok());
    }

    //#[test]
    //fn test_read_register() {
    //    let spi_expect = [];
    //    let spi_bus = get_fixture(&spi_expect);
    //}
}

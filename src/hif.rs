use crate::error::Error;
use crate::registers;
use crate::spi::SpiBusWrapper;
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::spi::FullDuplex;

pub mod group_ids {
    const MAIN: u8 = 0;
    const WIFI: u8 = 1;
    const IP: u8 = 2;
    const HIF: u8 = 3;
}

pub mod commands {
    pub mod main {}
    pub mod wifi {
        const REQ_CONNECT: u8 = 40;
        const REQ_DEFAULT_CONNECT: u8 = 41;
        const RESP_CONNECT: u8 = 42;
        const REQ_DISCONNECT: u8 = 43;
        const RESP_CON_STATE_CHANGED: u8 = 44;
        const REQ_SLEEP: u8 = 45;
        const REQ_WPS_SCAN: u8 = 46;
        const REQ_WPS: u8 = 47;
        const REQ_DISABLE_WPS: u8 = 49;
        const REQ_DHCP_CONF: u8 = 50;
        const REQ_ENABLE_MONITORING: u8 = 53;
        const REQ_DISABLE_MONITORING: u8 = 54;
        const RESP_WIFI_RX_PACKET: u8 = 55;
        const REQ_SEND_WIFI_PACKET: u8 = 56;
        const REQ_LSN_INT: u8 = 57;
        const REQ_DOZE: u8 = 58;
        const BIND: u8 = 65;
        const LISTEN: u8 = 66;
        const ACCEPT: u8 = 67;
        const CONNECT: u8 = 68;
        const SEND: u8 = 69;
        const RECV: u8 = 70;
        const SENDTO: u8 = 71;
        const RECVFROM: u8 = 72;
        const CLOSE: u8 = 73;
    }
    pub mod ip {}
    pub mod hif {}
}

pub struct HifHeader {
    gid: u8,
    op: u8,
    length: u16,
}

pub struct HostInterface;

impl HostInterface {
    /// This method wakes the chip from sleep mode using clockless register access
    pub fn chip_wake<SPI, O>(&mut self, spi_bus: &mut SpiBusWrapper<SPI, O>) -> Result<(), Error>
    where
        SPI: FullDuplex<u8>,
        O: OutputPin,
    {
        let mut trials: u32 = 0;
        let mut register_val: u32;
        let mut clock_status_val: u32;
        const WAKEUP_TRIALS_TIMEOUT: u8 = 4;
        register_val = spi_bus.read_register(registers::HOST_CORT_COMM)?;
        if (register_val & 0x1) == 0 {
            // USE bit 0 to indicate host wakeup
            spi_bus.write_register(registers::HOST_CORT_COMM, register_val | 0x1)?;
        }
        register_val = spi_bus.read_register(registers::WAKE_CLK_REG)?;
        // Set bit 1
        if (register_val & 0x2) == 0 {
            spi_bus.write_register(registers::WAKE_CLK_REG, register_val | 0x2)?;
        }
        loop {
            clock_status_val = spi_bus.read_register(registers::CLOCKS_EN_REG)?;
            if (clock_status_val & 0x2) != 0 {
                break;
            }
            // sleep here?
            trials += 1;
            if trials > WAKEUP_TRIALS_TIMEOUT as u32 {
                // error waking chip
                break;
            }
        }
        Ok(())
    }

    /// This method enables sleep mode for the chip
    pub fn chip_sleep<SPI, O>(&mut self, spi_bus: &mut SpiBusWrapper<SPI, O>) -> Result<(), Error>
    where
        SPI: FullDuplex<u8>,
        O: OutputPin,
    {
        let mut register_val: u32;
        loop {
            register_val = spi_bus.read_register(registers::CORT_HOST_COMM)?;
            if (register_val & 0x1) == 0 {
                break;
            }
        }
        // Clear bit 1
        register_val = spi_bus.read_register(registers::WAKE_CLK_REG)?;
        if (register_val & 0x2) != 0 {
            register_val &= !0x2;
            spi_bus.write_register(registers::WAKE_CLK_REG, register_val)?;
        }
        register_val = spi_bus.read_register(registers::HOST_CORT_COMM)?;
        if (register_val & 0x1) != 0 {
            register_val &= !0x1;
            spi_bus.write_register(registers::HOST_CORT_COMM, register_val)?;
        }
        Ok(())
    }

    /// This method sets the callback function for different events
    pub fn register_cb<SPI, O>(&mut self, spi_bus: &mut SpiBusWrapper<SPI, O>) -> Result<(), Error>
    where
        SPI: FullDuplex<u8>,
        O: OutputPin,
    {
        todo!()
    }

    /// This method is the host interface interrupt service
    pub fn isr<SPI, O>(&mut self, spi_bus: &mut SpiBusWrapper<SPI, O>) -> Result<(), Error>
    where
        SPI: FullDuplex<u8>,
        O: OutputPin,
    {
        todo!()
    }

    /// This method receives data read from the chip
    pub fn receive<SPI, O>(&mut self, spi_bus: &mut SpiBusWrapper<SPI, O>) -> Result<(), Error>
    where
        SPI: FullDuplex<u8>,
        O: OutputPin,
    {
        todo!()
    }

    /// This method sends data to the chip
    pub fn send<SPI, O>(
        &mut self,
        spi_bus: &mut SpiBusWrapper<SPI, O>,
        header: HifHeader,
    ) -> Result<(), Error>
    where
        SPI: FullDuplex<u8>,
        O: OutputPin,
    {
        todo!()
    }

    /// This method sets the chip sleep mode
    pub fn set_sleep_mode<SPI, O>(
        &mut self,
        spi_bus: &mut SpiBusWrapper<SPI, O>,
    ) -> Result<(), Error>
    where
        SPI: FullDuplex<u8>,
        O: OutputPin,
    {
        todo!()
    }

    /// This method returns the chip sleep mode
    pub fn get_sleep_mode<SPI, O>(
        &mut self,
        spi_bus: &mut SpiBusWrapper<SPI, O>,
    ) -> Result<(), Error>
    where
        SPI: FullDuplex<u8>,
        O: OutputPin,
    {
        todo!()
    }
}

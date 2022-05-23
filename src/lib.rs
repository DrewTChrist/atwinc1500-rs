#![no_std]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

pub mod error;
mod spi_command;

use embedded_hal::blocking::spi::Transfer;
use embedded_hal::digital::v2::{InputPin, OutputPin};
use embedded_hal::spi::FullDuplex;
use embedded_nal::SocketAddr;
use embedded_nal::TcpClientStack;
use embedded_nal::TcpFullStack;

use crate::error::Error;
use crate::spi_command::SpiCommand;

pub struct TcpSocket {}

// I was unable to find these anywhere in the Microchip/Atmel
// datasheets.
//
// They are of course in the driver code for those
// that don't mind wading around knee deep in some nasty C.
//
// I'll be leaving these constant names the same as they're
// found in the driver code so they can be found later.
#[rustfmt::skip]
mod constants {
    const WIFI_HOST_RCV_CTRL_0:    u32 = 0x1070;
    const WIFI_HOST_RCV_CTRL_1:    u32 = 0x1084;
    const WIFI_HOST_RCV_CTRL_2:    u32 = 0x1078;
    const WIFI_HOST_RCV_CTRL_3:    u32 = 0x106c;
    const WIFI_HOST_RCV_CTRL_4:    u32 = 0x150400;
    const WIFI_HOST_RCV_CTRL_5:    u32 = 0x1088;

    const NMI_CHIPID:              u32 = 0x1000;
    const NMI_STATE_REG:           u32 = 0x108c;
    const NMI_PIN_MUX_0:           u32 = 0x1408;
    #[allow(non_upper_case_globals)]
    const rNMI_GP_REG_1:           u32 = 0x14a0;
    #[allow(non_upper_case_globals)]
    const rNMI_GP_REG_2:           u32 = 0xc0008;
    const NMI_INTR_REG_BASE:       u32 = 0x1a00;
    const NMI_SPI_PROTOCOL_CONFIG: u32 = 0xe824;
    const BOOTROM_REG:             u32 = 0xc000c;
    const M2M_WAIT_FOR_HOST_REG:   u32 = 0x207bc;
}

/// Atwin1500 driver struct
pub struct Atwinc1500<SPI, O, I>
where
    SPI: FullDuplex<u8> + Transfer<u8>,
    O: OutputPin,
    I: InputPin,
{
    spi: SPI,
    cs: O,
    sclk: I,
    irq: I,
    reset: O,
    wake: O,
    crc: bool,
}

impl<SPI, O, I> Atwinc1500<SPI, O, I>
where
    SPI: FullDuplex<u8> + Transfer<u8>,
    O: OutputPin,
    I: InputPin,
{
    pub fn new(spi: SPI, cs: O, sclk: I, irq: I, reset: O, wake: O, crc: bool) -> Self {
        Self {
            spi,
            cs,
            sclk,
            irq,
            reset,
            wake,
            crc,
        }
    }

    fn spi_transfer<'w>(&mut self, words: &'w mut [u8]) -> Result<&'w [u8], Error> {
        if self.cs.set_low().is_err() {
            return Err(Error::PinStateError);
        }
        let rec = self.spi.transfer(words);
        if self.cs.set_high().is_err() {
            return Err(Error::PinStateError);
        }
        match rec {
            Ok(val) => Ok(val),
            Err(e) => Err(Error::SpiTransferError),
        }
    }

    fn spi_command<'w>(&mut self, words: &'w mut [u8]) -> Result<&'w [u8], Error> {
        todo!()
    }

    fn spi_read_register(&mut self) {
        todo!()
    }

    fn spi_read_data(&mut self) {
        todo!()
    }

    fn spi_write_register(&mut self) {
        todo!()
    }

    fn spi_write_data(&mut self) {
        todo!()
    }
}

impl<SPI, O, I> TcpClientStack for Atwinc1500<SPI, O, I>
where
    SPI: FullDuplex<u8> + Transfer<u8>,
    O: OutputPin,
    I: InputPin,
{
    type TcpSocket = TcpSocket;
    type Error = Error;

    fn socket(&mut self) -> Result<TcpSocket, Error> {
        todo!()
    }

    fn connect(
        &mut self,
        socket: &mut TcpSocket,
        address: SocketAddr,
    ) -> Result<(), embedded_nal::nb::Error<Error>> {
        todo!()
    }

    fn is_connected(&mut self, socket: &TcpSocket) -> Result<bool, Error> {
        todo!()
    }

    fn send(
        &mut self,
        socket: &mut TcpSocket,
        data: &[u8],
    ) -> Result<usize, embedded_nal::nb::Error<Error>> {
        todo!()
    }

    fn receive(
        &mut self,
        socket: &mut TcpSocket,
        data: &mut [u8],
    ) -> Result<usize, embedded_nal::nb::Error<Error>> {
        todo!()
    }

    fn close(&mut self, socket: TcpSocket) -> Result<(), Error> {
        todo!()
    }
}

impl<SPI, O, I> TcpFullStack for Atwinc1500<SPI, O, I>
where
    SPI: FullDuplex<u8> + Transfer<u8>,
    O: OutputPin,
    I: InputPin,
{
    fn bind(&mut self, socket: &mut TcpSocket, port: u16) -> Result<(), Error> {
        todo!()
    }

    fn listen(&mut self, socket: &mut TcpSocket) -> Result<(), Error> {
        todo!()
    }

    fn accept(
        &mut self,
        socket: &mut TcpSocket,
    ) -> Result<(TcpSocket, SocketAddr), embedded_nal::nb::Error<Error>> {
        todo!()
    }
}

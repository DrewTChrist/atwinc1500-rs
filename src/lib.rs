#![no_std]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

pub mod error;
mod hif;
mod registers;
mod spi;

use embedded_hal::blocking::spi::Transfer;
use embedded_hal::digital::v2::{InputPin, OutputPin};
use embedded_hal::spi::FullDuplex;
use embedded_nal::SocketAddr;
use embedded_nal::TcpClientStack;
use embedded_nal::TcpFullStack;

use error::Error;

pub struct TcpSocket {}

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

    /// Transfers some data then receives some data on the spi bus
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

    fn spi_read_register<'w>(&mut self, cmd_buffer: &'w mut [u8], address: u16) -> Result<&'w [u8], Error> {
        // Command value goes in first byte
        // address next two bytes little endian
        // fourth byte is zero
        // fifth byte is crc
        cmd_buffer[0] = spi::commands::CMD_INTERNAL_READ;
        cmd_buffer[1] = (address >> 8) as u8;
        cmd_buffer[2] = (address & 0x0f) as u8;
        cmd_buffer[3] = 0;
        self.spi_transfer(cmd_buffer)
    }

    fn spi_read_data<'w>(&mut self, cmd_buffer: &'w mut [u8]) -> Result<&'w [u8], Error> {
        todo!()
    }

    fn spi_write_register<'w>(&mut self, cmd_buffer: &'w mut [u8]) -> Result<&'w [u8], Error> {
        todo!()
    }

    fn spi_write_data<'w>(&mut self, cmd_buffer: &'w mut [u8]) -> Result<&'w [u8], Error> {
        todo!()
    }

    fn hif_chip_wake(&mut self) {
        todo!()
    }

    fn hif_chip_sleep(&mut self) {
        todo!()
    }

    fn hif_register_cb(&mut self) {
        todo!()
    }

    fn hif_isr(&mut self) {
        todo!()
    }

    fn hif_receive(&mut self) {
        todo!()
    }

    fn hif_send(&mut self) {
        todo!()
    }

    fn hif_set_sleep_mode(&mut self) {
        todo!()
    }

    fn hif_get_sleep_mode(&mut self) {
        todo!()
    }

    pub fn get_chip_info(&mut self) {
        //let mut buffer: [u8; spi::commands::sizes::TYPE_A] = [0; spi::commands::sizes::TYPE_A];
        //self.spi_read_register(&mut buffer);
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

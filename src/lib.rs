#![no_std]
#![allow(dead_code)]
#![allow(unused_variables)]

use embedded_hal::spi;
use embedded_nal::SocketAddr;
use embedded_nal::TcpClientStack;
use embedded_nal::TcpFullStack;

#[derive(Debug)]
pub struct Error {}
pub struct TcpSocket {}

/// Atwin1500 driver struct
pub struct Atwinc1500<SPI>
where
    SPI: spi::FullDuplex<u8>,
{
    spi: SPI,
}

impl<SPI> Atwinc1500<SPI> where SPI: spi::FullDuplex<u8> {}

impl<SPI> TcpClientStack for Atwinc1500<SPI>
where
    SPI: spi::FullDuplex<u8>,
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

impl<SPI> TcpFullStack for Atwinc1500<SPI>
where
    SPI: spi::FullDuplex<u8>,
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

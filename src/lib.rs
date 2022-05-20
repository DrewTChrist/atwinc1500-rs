#![no_std]
#![allow(dead_code)]
#![allow(unused_variables)]

use embedded_hal::blocking::spi::Transfer;
use embedded_hal::spi::FullDuplex;
use embedded_nal::SocketAddr;
use embedded_nal::TcpClientStack;
use embedded_nal::TcpFullStack;

#[derive(Debug)]
pub struct Error {}
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

    const CMD_DMA_WRITE:           u32 = 0xc1;
    const CMD_DMA_READ:            u32 = 0xc2;
    const CMD_INTERNAL_WRITE:      u32 = 0xc3;
    const CMD_INTERNAL_READ:       u32 = 0xc4;
    const CMD_TERMINATE:           u32 = 0xc5;
    const CMD_REPEAT:              u32 = 0xc6;
    const CMD_DMA_EXT_WRITE:       u32 = 0xc7;
    const CMD_DMA_EXT_READ:        u32 = 0xc8;
    const CMD_SINGLE_WRITE:        u32 = 0xc9;
    const CMD_SINGLE_READ:         u32 = 0xca;
    const CMD_RESET:               u32 = 0xcf;
}

/// Atwin1500 driver struct
pub struct Atwinc1500<SPI>
where
    SPI: FullDuplex<u8> + Transfer<u8>,
{
    spi: SPI,
}

impl<SPI> Atwinc1500<SPI>
where
    SPI: FullDuplex<u8> + Transfer<u8>,
{
    pub fn new(spi: SPI) -> Self {
        Self { spi }
    }

    fn spi_transfer(&mut self) { todo!() }

    fn spi_command(&mut self) { todo!() }

    fn spi_read_register(&mut self) { todo!() }

    fn spi_read_data(&mut self) { todo!() }

    fn spi_write_register(&mut self) { todo!() }

    fn spi_write_data(&mut self) { todo!() }
}

impl<SPI> TcpClientStack for Atwinc1500<SPI>
where
    SPI: FullDuplex<u8> + Transfer<u8>,
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
    SPI: FullDuplex<u8> + Transfer<u8>,
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

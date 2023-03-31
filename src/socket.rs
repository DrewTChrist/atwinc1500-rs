//! Socket related members

#[repr(u8)]
#[derive(from_u8_derive::FromByte)]
pub enum SocketCommand {
    Bind = 0x41,
    Listen = 0x42,
    Accept = 0x43,
    Connect = 0x44,
    Send = 0x45,
    Recv = 0x46,
    Sendto = 0x47,
    Recvfrom = 0x48,
    Close = 0x49,
    DnsResolve = 0x4a,
    SslConnect = 0x4b,
    SslSend = 0x4c,
    SslRecv = 0x4d,
    SslClose = 0x4e,
    SetSocketOption = 0x4f,
    SslCreate = 0x50,
    SslSetSockOpt = 0x51,
    Ping = 0x52,
    SslSetCsList = 0x53,
    SslBind = 0x54,
    SslExpCheck = 0x55,
    Invalid,
}

/// TcpSocket implementation
pub struct TcpSocket {}

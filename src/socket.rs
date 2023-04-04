//! Socket related members

/// SocketCommand variants represent
/// valid Atwinc1500 socket commands
/// and responses
#[repr(u8)]
#[derive(from_u8_derive::FromByte, Debug)]
pub enum SocketCommand {
    /// Bind command
    Bind = 0x41,
    /// Listen command
    Listen = 0x42,
    /// Accept command
    Accept = 0x43,
    /// Connect command
    Connect = 0x44,
    /// Send command
    Send = 0x45,
    /// Receive command
    Recv = 0x46,
    /// Send to command
    Sendto = 0x47,
    /// Receive from command
    Recvfrom = 0x48,
    /// Close command
    Close = 0x49,
    /// DNS resolve command
    DnsResolve = 0x4a,
    /// SSL socket connect command
    SslConnect = 0x4b,
    /// SSL socket send command
    SslSend = 0x4c,
    /// SSL socket receive command
    SslRecv = 0x4d,
    /// SSL socket close command
    SslClose = 0x4e,
    /// Set socket option command
    SetSocketOption = 0x4f,
    SslCreate = 0x50,
    SslSetSockOpt = 0x51,
    /// Ping command
    Ping = 0x52,
    SslSetCsList = 0x53,
    SslBind = 0x54,
    SslExpCheck = 0x55,
    Invalid,
}

/// TcpSocket implementation
pub struct TcpSocket {}

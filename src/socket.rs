//! Socket related members

/// Bind command
pub const BIND: u8 = 65;
/// Listen command
pub const LISTEN: u8 = 66;
/// Accept command
pub const ACCEPT: u8 = 67;
/// Connect command
pub const CONNECT: u8 = 68;
/// Send command
pub const SEND: u8 = 69;
/// Receive command
pub const RECV: u8 = 70;
/// Send To command
pub const SENDTO: u8 = 71;
/// Receive from command
pub const RECVFROM: u8 = 72;
/// Close command
pub const CLOSE: u8 = 73;

/// TcpSocket implementation
pub struct TcpSocket {}

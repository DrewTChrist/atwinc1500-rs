# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
## Added
- Add Connection struct so pub types can be private
- Add Mode and Status enums to lib.rs

## Changed
- Change wifi::NewConnection and wifi::OldConnection to pub(crate)
- Make wifi::SecurityType and wifi::ConnectionOptions private
- Implement From<Connection> instead of From<ConnectionParameters> in wifi.rs
- Hide unimplemented method stubs and modules with from docs
- Simplify gpio methods

## Removed
- Remove channels 15 & 16 from the Channel enum as those aren't even valid
- Remove ConnectionParameters impl

## [0.1.0] - 2022-10-13
### Added
- Added module crc.rs
- Added module error.rs
- Added module gpio.rs
- Added module hif.rs
- Added module macros.rs
- Added module registers.rs
- Added module socket.rs
- Added module spi.rs
- Added module types.rs
- Added module wifi.rs
- Added some tests for spi module
- Added notice from the Atmel Atwinc1500 host driver
- Implemented read/write data, read/write register spi methods
- Roughly implemented several host interface methods
- Implemented read mac address
- Implemented read firmware version
- Implemented gpio control (direction/logical value)
- Implemented connect to station with WPA2
- Implemented disconnect from station
- Implemented default connect (last successful network)

[Unreleased]: https://github.com/drewtchrist/atwinc1500-rs/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/drewtchrist/atwinc1500-rs/tag/v0.1.0

# atwinc1500-rs

## Description
This is a driver for the atwinc1500 network controller written in pure Rust. The
primary targets for this driver are boards like the [Adafruit Feather M0 Wifi](https://adafruit.com/product/3010)
or the [Adafruit Atwinc1500 Breakout](https://adafruit.com/product/2999). Since the
datasheets for the atwinc1500 do not provide all of the information needed to interface
the device, this driver uses a lot of information from the Atmel driver that was written
in C.

## Table of Contents
1. [Roadmap](#roadmap)
2. [Installation](#installation)
3. [Usage](#usage)
4. [License](#license)

## Roadmap
- [x] Info/Hardware
    - [x] Gpio control
    - [x] Read mac address
    - [x] Read firmware version
- [ ] Wifi
    - [ ] Scan
    - [ ] Connect
        - [x] Older connection format
        - [ ] Newer connection format
        - [ ] Open network
        - [ ] WEP (**WEP is deprecated in later atwinc firmware versions**)
        - [x] Wpa2
        - [ ] Wpa2 Enterprise
    - [x] Disconnect from network
    - [ ] Read RSSI
    - [ ] AP Mode
- [ ] TcpFullStack
    - [ ] bind
    - [ ] listen
    - [ ] accept
- [ ] TcpClientStack
    - [ ] socket
    - [ ] connect
    - [ ] is_connected
    - [ ] send
    - [ ] receive
    - [ ] close
- [ ] Crypto
- [ ] SSL
- [ ] OTA
- [ ] ATE Mode
- [ ] UART

## Installation
Add this crate to your Cargo.toml:
```toml
atwinc1500 = { git = "https://github.com/drewtchrist/atwinc1500-rs" }
```

## Usage
Examples can be found [here](https://github.com/drewtchrist/atwinc1500-rs-examples). 

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

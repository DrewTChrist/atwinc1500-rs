# atwinc1500-rs

## Description
This is a driver for the atwinc1500 wifi module written in Rust. The 
primary targets for this driver are the [Adafruit Feather M0 Wifi](https://adafruit.com/product/3010) 
and the [Adafruit Atwinc1500 Breakout](https://adafruit.com/product/2999). 
This may put some features outside the scope of this project, but they 
are still welcomed additions. This code has been heavily influenced by 
[WiFi101](https://github.com/arduino-libraries/wifi101) and [winc_wifi](https://github.com/jbentham/winc_wifi).

## Table of Contents
1. [Roadmap](#roadmap)
2. [Installation](#installation)
3. [Usage](#usage)
4. [License](#license)

# Roadmap
- [ ] Chip
    - [x] Gpio control
    - [x] Read mac address
    - [x] Read firmware version
- [ ] Wifi
    - [ ] Scan
    - [ ] Connect
    - [ ] Disconnect
    - [ ] RSSI
    - [ ] Ap Mode
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

# Installation
Add this crate to your Cargo.toml:
```toml
atwinc1500 = { git = "https://github.com/drewtchrist/atwinc1500-rs" }
```

# Usage
Examples can be found [here](https://github.com/drewtchrist/atwinc1500-rs-examples). 

# License

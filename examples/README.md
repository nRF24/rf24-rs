# rf24-rs examples
This directory is a separate cargo project to demonstrate the rf24-rs package being used on various boards. The following boards are supported:
- [x] linux armhf (32bit OS on a Raspberry Pi or similar linux machine)
- [x] linux aarch64 (64bit OS on a Raspberry Pi or similar linux machine)
- [ ] rp2040
- [ ] esp32
- [ ] nRF52840
- [ ] nRF52833 (like MicroBit v2)
- [ ] nRF51822 (like MicroBit v1)


## Running an example

First you have to build the example before you run it.

For Linux boards, this can be done simply by executing the command:
```
cargo run --bin getting-started --release
```

For microcontrollers, you need a way to upload the built binary to the board. For this, we recommend using [`probe-rs`](https://probe.rs). Once installed, you'll have a new `cargo flash` subcommand at your disposal. But you'll still have to select what chip you are going to flash the example to.

1. use 

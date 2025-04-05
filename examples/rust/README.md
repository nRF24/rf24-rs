# rf24-rs examples

This directory is a separate cargo project to demonstrate the rf24-rs package being used on various boards. The examples are written for following boards:

- [x] linux armhf (32bit OS on a Raspberry Pi or similar linux machine)
- [x] linux aarch64 (64bit OS on a Raspberry Pi or similar linux machine)

Any sources located in examples/rust/src (excluding the bin folder) are
meant to be used as platform abstraction. Refer to these files for
platform-specific implementation details.

## Running an example

First you have to build the example before you can run it.
The sources for each example are located in examples/rust/src/bin/.

For Linux boards, this can be done simply by executing the command:

```shell
cargo run --bin getting-started
```

The provided examples names (value passed to the `--bin` option) are

- `getting-started`
- `streaming-data`
- `ack-payloads`
- `manual-ack`
- `multiceiver`
- `irq-config`
- `scanner`
- `quick-config`
- `fake-ble`

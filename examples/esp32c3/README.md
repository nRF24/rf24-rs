# ESP32-C3 nRF24L01+ ping-pong

This example runs on the Seeed XIAO ESP32-C3 and communicates with an nRF24L01+ over SPI. Two boards exchange bidirectional PING/PONG messages. Serial output provides diagnostic messages.

## Wiring

| nRF24L01+ | ESP32-C3 |
| --- | --- |
| VCC | 3.3 V |
| GND | GND |
| SCK | GPIO8 |
| MOSI | GPIO10 |
| MISO | GPIO21 |
| CSN | GPIO5 |
| CE | GPIO4 |
| IRQ | Not connected |

Power the radio from **3.3 V**. GPIO mappings can be adjusted in `src/main.rs` as needed.

## Build and flash

1. In `src/main.rs`, set `INITIATOR` to `true` for one board and `false` for the other.
2. From `examples/esp32c3/`, build and flash the firmware:

   ```sh
   cargo build --release
   cargo run --release
   ```

   The `cargo run` command flashes the board and opens the serial console automatically.

## Serial output

If the serial console does not open automatically, find the port manually:

```sh
ls /dev/ttyUSB*
screen /dev/ttyUSB0 115200
```

Exit `screen` with **Ctrl-A**, then **K**, then confirm.

Example output:

```text
RF24 ping-pong - ESP32-C3
SPI2: 1 MHz, Mode 0
SCK=GPIO8 MOSI=GPIO10 MISO=GPIO21
CSN=GPIO5 CE=GPIO4
RF Channel: 50
Role: Initiator
[PASS] Radio initialized
[RX] Listening on channel 50
[RF24] TX PING #1: OK
[RF24] RX PONG #1
[RF24] TX PING #2: OK
[RF24] RX PONG #2
```

The responder reports `RX PING ... -> TX PONG ...`. A `PONG timeout` on the initiator means the expected application-level response was not received within 500 ms.

## Hardware validation

This example has been tested on real hardware:

- **RP2350 ↔ ESP32-C3**: Cross-platform PING/PONG exchange confirmed on both roles.

The two boards successfully communicate using `rf24-rs` and the nRF24L01+ radio.

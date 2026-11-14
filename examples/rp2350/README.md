# RP2350 nRF24L01+ ping-pong

This example runs on the Raspberry Pi Pico 2 W (RP2350) and communicates with an nRF24L01+ over SPI. Two boards exchange bidirectional PING/PONG messages. USB CDC provides diagnostic output over the board's USB connection.

## Wiring

| nRF24L01+ | Pico 2 W      |
| --------- | ------------- |
| VCC       | 3V3           |
| GND       | GND           |
| SCK       | GP18          |
| MOSI      | GP19          |
| MISO      | GP16          |
| CSN       | GP21          |
| CE        | GP20          |
| IRQ       | Not connected |

Power the radio from **3.3 V**. GPIO mappings can be adjusted in `src/main.rs` as needed.

## Build and flash

1. In `src/main.rs`, set `INITIATOR` to `true` for one board and `false` for the other.
2. From `examples/rp2350/`, build the firmware:

   ```sh
   cargo build --release
   ```

3. Convert the ELF to UF2:

   ```sh
   picotool uf2 convert \
     target/thumbv8m.main-none-eabihf/release/rp2350-rf24 \
     -t elf \
     ./pico2_nrf24_driver.uf2
   ```

4. Put the Pico in BOOTSEL mode by holding BOOTSEL while connecting USB, then copy `pico2_nrf24_driver.uf2` to the `RPI-RP2` drive. Repeat for the second board with the opposite role.

## USB CDC output

On macOS:

```sh
ls /dev/cu.usbmodem*
screen /dev/cu.usbmodemXXXX 115200
```

On Linux:

```sh
screen /dev/ttyACM0 115200
```

Exit `screen` with **Ctrl-A**, then **K**, then confirm.

Example output:

```text
[RF24] TX PING #1: OK
[RF24] RX PONG #1
[RF24] TX PING #2: OK
[RF24] RX PONG #2
```

The responder reports `RX PING #1 -> TX PONG #1: OK`. A `PONG timeout` means the expected response was not received within 500 ms.

## Hardware validation

This example has been tested on real hardware:

- **RP2350 ↔ ESP32-C3**: Cross-platform PING/PONG exchange confirmed on both roles.

The two boards successfully communicate using `rf24-rs` and the nRF24L01+ radio.

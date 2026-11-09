# RP2350 nRF24L01+ ping-pong

This example runs on the Raspberry Pi Pico 2 W (RP2350) and communicates with an nRF24L01+ over SPI. Two boards exchange bidirectional PING/PONG messages. USB CDC provides diagnostic output over the board's USB connection, with no external UART hardware.

## Wiring

| nRF24L01+ | Pico 2 W |
| --- | --- |
| VCC | 3V3 |
| GND | GND |
| SCK | GP18 |
| MOSI | GP19 |
| MISO | GP16 |
| CSN | GP21 |
| CE | GP20 |
| IRQ | Not connected |

Power the radio from **3.3 V**. If needed, change the GPIO assignments in `src/main.rs`, observing the RP2350's valid SPI pin mappings.

## Build and flash

1. In `src/main.rs`, set `INITIATOR` to `true` for one Pico and `false` for the other.
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

4. Put the Pico in BOOTSEL mode by holding BOOTSEL while connecting USB, then copy `pico2_nrf24_driver.uf2` to the `RPI-RP2` drive. Repeat the build and flash steps for the other role; the first board keeps its firmware while you rebuild.

## USB CDC output

On macOS, find the board's serial device and open it:

```sh
ls /dev/cu.usbmodem*
screen /dev/cu.usbmodemXXXX 115200
```

Exit `screen` with **Ctrl-A**, then **K**, then confirm.

Example output:

```text
[RF24] TX PING #1: OK
[RF24] RX PONG #1
```

The responder reports `RX PING ... -> TX PONG ...: OK`. A `PONG timeout` means the expected application-level response was not received. USB diagnostics are best-effort.

This is an example for running and validating `rf24-rs` on real hardware, not a production communication protocol.

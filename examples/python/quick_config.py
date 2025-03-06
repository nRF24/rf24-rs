"""
This example demonstrates how to quickly and easily
change the radio's configuration.

This example requires no counterpart as
it does not actually transmit nor receive anything.

See documentation at https://nRF24.github.io/rf24-rs
"""

from rf24_py import RF24, RadioConfig, CrcLength


class App:
    def __init__(self):
        # The radio's CE Pin uses a GPIO number.
        ce_pin = 22  # for GPIO22

        # The radio's CSN Pin corresponds the SPI bus's CS pin (aka CE pin).
        # On Linux, consider the device path `/dev/spidev<a>.<b>`:
        #   - `<a>` is the SPI bus number (defaults to `0`)
        #   - `<b>` is the CSN pin (must be unique for each device on the same SPI bus)
        csn_pin = 0  # aka CE0 for SPI bus 0 (/dev/spidev0.0)

        # create a radio object for the specified hardware config:
        self.radio = RF24(ce_pin, csn_pin)

    def run(self):
        """Configure the radio for 2 different scenarios and
        print the configuration details for each."""

        ble_context = RadioConfig()  # library defaults
        ble_context.channel = 2  # BLE specs hop/rotate amongst channels 2, 26, and 80
        ble_context.crc_length = CrcLength.Disabled
        ble_context.auto_ack = 0
        ble_context.address_length = 4
        ble_addr = b"\x71\x91\x7d\x6b"
        ble_context.set_rx_address(1, ble_addr)
        ble_context.tx_address = ble_addr

        normal_context = RadioConfig()  # library defaults
        normal_context.set_rx_address(1, b"1Node")
        normal_context.tx_address = b"2Node"

        self.radio.with_config(ble_context)
        print(
            "Settings for BLE context\n------------------------",
        )
        self.radio.print_details()

        self.radio.with_config(normal_context)
        print(
            "\nSettings for normal context\n---------------------------",
        )
        self.radio.print_details()


if __name__ == "__main__":
    print(__file__)  # print example name

    app = App()
    app.run()
    app.radio.power_down()

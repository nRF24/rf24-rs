"""
The simplest example of using the nRF24L01 transceiver to send and receive.

This example is meant to be run on 2 separate nRF24L01 transceivers.

See documentation at https://nRF24.github.io/rf24-rs
"""

import time
import struct
from rf24_py import RF24, PaLevel


class App:
    def __init__(self) -> None:
        # The radio's CE Pin uses a GPIO number.
        ce_pin = 22  # for GPIO22

        # The radio's CSN Pin corresponds the SPI bus's CS pin (aka CE pin).
        # On Linux, consider the device path `/dev/spidev<a>.<b>`:
        #   - `<a>` is the SPI bus number (defaults to `0`)
        #   - `<b>` is the CSN pin (must be unique for each device on the same SPI bus)
        csn_pin = 0  # aka CE0 for SPI bus 0 (/dev/spidev0.0)

        # create a radio object for the specified hardware config:
        self.radio = RF24(ce_pin, csn_pin)

        # For this example, we will use different addresses
        # An address need to be a buffer protocol object (bytearray)
        address = [b"1Node", b"2Node"]
        # It is very helpful to think of an address as a path instead of as
        # an identifying device destination

        # to use different addresses on a pair of radios, we need a variable to
        # uniquely identify which address this radio will use to transmit
        # 0 uses address[radio_number] to transmit, 1 uses address[not radio_number] to transmit
        radio_number = bool(
            int(input("Which radio is this? Enter '0' or '1'. Defaults to '0' ") or 0)
        )

        # initialize the nRF24L01 on the spi bus
        self.radio.begin()

        # set the Power Amplifier level to -12 dBm since this test example is
        # usually run with nRF24L01 transceivers in close proximity of each other
        self.radio.pa_level = PaLevel.Low  # PaLevel.Max is default

        # set TX address of RX node into the TX pipe
        self.radio.open_tx_pipe(address[radio_number])  # always uses pipe 0

        # set RX address of TX node into an RX pipe
        self.radio.open_rx_pipe(1, address[not radio_number])  # using pipe 1

        # To save time during transmission, we'll set the payload size to be only what
        # we need. A float value occupies 4 bytes in memory using struct.calcsize()
        # "<f" means a little endian unsigned float
        self.radio.payload_length = struct.calcsize("<f")

        self.payload = 0.0

        # for debugging
        # self.radio.print_details()

    def tx(self, count: int = 5):
        """Transmits an incrementing float every second"""
        self.radio.as_tx()  # ensures the nRF24L01 is in TX mode

        while count:
            # use struct.pack() to pack your data into a usable payload
            # into a usable payload
            buffer = struct.pack("<f", self.payload)
            # "<f" means a single little endian (4 byte) float value.
            start_timer = time.monotonic_ns()  # start timer
            result = self.radio.send(buffer)
            end_timer = time.monotonic_ns()  # end timer
            if not result:
                print("Transmission failed or timed out")
            else:
                print(
                    "Transmission successful! Time to Transmit:",
                    f"{(end_timer - start_timer) / 1000} us. Sent: {self.payload}",
                )
                self.payload += 0.01
            time.sleep(1)
            count -= 1

    def rx(self, timeout: int = 6):
        """Polls the radio and prints the received value. This method expires
        after 6 seconds of no received transmission."""
        self.radio.as_rx()  # put radio into RX mode and power up

        end_time = time.monotonic() + timeout
        while time.monotonic() < end_time:
            has_payload, pipe_number = self.radio.available_pipe()
            if has_payload:
                # fetch 1 payload from RX FIFO
                received = (
                    self.radio.read()
                )  # also clears self.radio.irq_dr status flag
                # expecting a little endian float, thus the format string "<f"
                # received[:4] truncates padded 0s in case dynamic payloads are disabled
                self.payload = struct.unpack("<f", received[:4])[0]
                # print details about the received packet
                print(
                    f"Received {len(received)} bytes on pipe {pipe_number}: {self.payload}"
                )
                end_time = time.monotonic() + timeout  # reset the timeout timer

        # recommended behavior is to keep in TX mode while idle
        self.radio.as_tx()  # put the nRF24L01 is in TX mode

    def set_role(self):
        """Set the role using stdin stream. Timeout arg for slave() can be
        specified using a space delimiter (e.g. 'R 10' calls `slave(10)`)

        :return:
            - True when role is complete & app should continue running.
            - False when app should exit
        """
        user_input = (
            input(
                "*** Enter 'R' for receiver role.\n"
                "*** Enter 'T' for transmitter role.\n"
                "*** Enter 'Q' to quit example.\n"
            )
            or "?"
        )
        user_input = user_input.split()
        if user_input[0].upper().startswith("R"):
            self.rx(*[int(x) for x in user_input[1:2]])
            return True
        if user_input[0].upper().startswith("T"):
            self.tx(*[int(x) for x in user_input[1:2]])
            return True
        if user_input[0].upper().startswith("Q"):
            self.radio.power = False
            return False
        print(user_input[0], "is an unrecognized input. Please try again.")
        return True


if __name__ == "__main__":
    print(__file__)  # print example name

    app = App()
    try:
        while app.set_role():
            pass  # continue example until 'Q' is entered
    except KeyboardInterrupt:
        print(" Keyboard Interrupt detected. Exiting...")
        app.radio.power = False

"""
This example uses the nRF24L01 to transmit and respond with an
acknowledgment (ACK) transmissions. Notice that the auto-ack feature is
enabled, but this example doesn't use automatic ACK payloads because automatic
ACK payloads' data will always be outdated by 1 transmission. Instead, this
example uses a call-and-response paradigm.

This example is meant to be run on 2 separate nRF24L01 transceivers.

See documentation at https://nRF24.github.io/rf24-rs
"""

import struct
import time
from rf24_py import RF24, PaLevel, StatusFlags


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
        # we need.
        # "<b" means a little endian unsigned byte
        # we also need an addition 7 bytes for the payload message
        self.radio.payload_length = struct.calcsize("<b") + 7

        self.counter = 0

        # for debugging
        # self.radio.print_details()

    def tx(self, count: int = 5):
        """Transmits a message and an incrementing integer every second"""
        self.radio.as_tx()  # ensures the nRF24L01 is in TX mode

        while count:  # only transmit `count` packets
            # use struct.pack() to pack your data into a usable payload
            # "<b" means a single little endian unsigned byte.
            # NOTE we added a b"\x00" byte as a c-string's NULL terminating 0
            buffer = b"Hello \x00" + struct.pack("<b", self.counter)
            start_timer = time.monotonic_ns()  # start timer
            result = self.radio.send(buffer)
            if not result:
                print("Transmission failed or timed out")
            else:
                self.radio.as_rx()
                got_response = False
                timeout = time.monotonic() * 1000 + 200  # use 200 ms timeout
                while time.monotonic() * 1000 < timeout:
                    if self.radio.available():
                        got_response = True
                        break
                end_timer = time.monotonic_ns()  # end timer
                self.radio.as_tx()
                print(
                    "Transmission successful. Sent: ",
                    f"{buffer[:6].decode('utf-8')}{self.counter}.",
                    end=" ",
                )
                if not got_response:
                    print("No response received.")
                else:
                    ack = self.radio.read()
                    # decode response's text as an string
                    # NOTE ack[:6] ignores the NULL terminating 0
                    response = ack[:6].decode("utf-8")
                    # use struct.unpack() to get the response's appended int
                    # NOTE ack[7:] discards NULL terminating 0, and
                    # "<b" means its a single little endian unsigned byte
                    counter = struct.unpack("<b", ack[7:])[0]
                    print(
                        f"Received: {response}{counter}. Roundtrip delay:",
                        f"{(end_timer - start_timer) / 1000} us.",
                    )
                    self.counter += 1
            time.sleep(1)  # make example readable by slowing down transmissions
            count -= 1

    def rx(self, timeout: int = 6):
        """Polls the radio and prints the received value. This method expires
        after 6 seconds of no received transmission"""
        self.radio.as_rx()  # put radio into RX mode and power up

        end_time = time.monotonic() + timeout  # start a timer to detect timeout
        while time.monotonic() < end_time:
            # receive payloads or wait 6 seconds till timing out
            has_payload, pipe_number = self.radio.available_pipe()
            if has_payload:
                received = self.radio.read()  # fetch 1 payload from RX FIFO
                # use struct.unpack() to get the payload's appended int
                # NOTE received[7:] discards NULL terminating 0, and
                # "<b" means its a single little endian unsigned byte
                self.counter = struct.unpack("<b", received[7:])[0] + 1
                # use bytes() to pack our data into a usable payload
                # NOTE b"\x00" byte is a c-string's NULL terminating 0
                buffer = b"World \x00" + bytes([self.counter])

                self.radio.as_tx()  # set radio to TX mode
                self.radio.write(buffer)  # load payload into radio's RX buffer
                # keep retrying to send response for 150 milliseconds
                response_timeout = time.monotonic_ns() + 150000000
                response_result = False
                while time.monotonic_ns() < response_timeout:
                    self.radio.update()
                    flags: StatusFlags = self.radio.get_status_flags()
                    if flags.tx_ds:
                        response_result = True
                        break
                    if flags.tx_df:
                        self.radio.rewrite()
                self.radio.as_rx()  # set radio back into RX mode

                # print the payload received and the response's payload
                print(
                    f"Received {len(received)} bytes on pipe {pipe_number}:",
                    f"{received[:6].decode('utf-8')}{self.counter - 1}.",
                    end=" ",
                )
                if response_result:
                    print(f"Sent: {buffer[:6].decode('utf-8')}{self.counter}")
                else:
                    self.radio.flush_tx()
                    print("Response failed or timed out")
                end_time = time.monotonic() + timeout  # reset the timeout timer

        # recommended behavior is to keep in TX mode while idle
        self.radio.as_tx()  # put the nRF24L01 into inactive TX mode

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

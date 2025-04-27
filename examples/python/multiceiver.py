"""
This example uses 1 nRF24L01 to receive data from up to 6 other
transceivers. This technique is called "multiceiver" in the datasheet.

This example is meant to be run on at least 2 separate nRF24L01 transceivers.
Although, this example can be used on 7 transceivers at most simultaneously.

See documentation at https://nRF24.github.io/rf24-rs
"""

import struct
import time
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

        # initialize the nRF24L01 on the spi bus
        self.radio.begin()

        # setup the addresses for all transmitting nRF24L01 nodes
        self.addresses = [
            b"\x78" * 5,
            b"\xf1\xb6\xb5\xb4\xb3",
            b"\xcd\xb6\xb5\xb4\xb3",
            b"\xa3\xb6\xb5\xb4\xb3",
            b"\x0f\xb6\xb5\xb4\xb3",
            b"\x05\xb6\xb5\xb4\xb3",
        ]
        # It is very helpful to think of an address as a path instead of as
        # an identifying device destination

        # set the Power Amplifier level to -12 dBm since this test example is
        # usually run with nRF24L01 transceivers in close proximity of each other
        self.radio.pa_level = PaLevel.Low  # PaLevel.Max is default

        # To save time during transmission, we'll set the payload size to be only what
        # we need.
        # 2 int occupy 8 bytes in memory using len(struct.pack())
        # "<ii" means 2x little endian unsigned int
        self.radio.payload_length = struct.calcsize("<ii")

        # for debugging
        # self.radio.print_details()

    def tx(self, node_number: int = 0, count: int = 6):
        """start transmitting to the base station.

        :param int node_number: the node's identifying index (from the
            the `addresses` list). This is a required parameter
        :param int count: the number of times that the node will transmit
            to the base station.
        """
        # According to the datasheet, the auto-retry features's delay value should
        # be "skewed" to allow the RX node to receive 1 transmission at a time.
        # So, use varying delay between retry attempts and 15 (at most) retry attempts
        self.radio.set_auto_retries(
            ((node_number * 3) % 12) + 3, 15
        )  # max value is 15 for both args

        # set the TX address to the address of the base station (always uses pipe 0).
        self.radio.as_tx(self.addresses[node_number])  # enter inactive TX mode

        counter = 0
        # use the node_number to identify where the payload came from
        while counter < count:
            counter += 1
            # payloads will include the node_number and a payload ID character
            payload = struct.pack("<ii", node_number, counter)
            start_timer = time.monotonic_ns()
            report = self.radio.send(payload)
            end_timer = time.monotonic_ns()
            # show something to see it isn't frozen
            if report:
                print(
                    f"Transmission of payloadID {counter} as node {node_number}",
                    f"successful! Transmission time: {(end_timer - start_timer) / 1000}",
                    "us",
                )
            else:
                print("Transmission failed or timed out")
            time.sleep(1)  # slow down the test for readability

        # recommended behavior is to keep in TX mode while idle
        self.radio.as_tx()  # enter inactive TX mode

    def rx(self, timeout=10):
        """Use the nRF24L01 as a base station for listening to all nodes"""
        # write the addresses to all pipes.
        for pipe, addr in enumerate(self.addresses):
            self.radio.open_rx_pipe(pipe, addr)
        self.radio.as_rx()  # put base station into RX mode
        end_time = time.monotonic() + timeout  # start timer
        while time.monotonic() < end_time:
            has_payload, pipe_number = self.radio.available_pipe()
            if has_payload:
                data = self.radio.read()
                # unpack payload
                node_id, payload_id = struct.unpack("<ii", data)
                # show the pipe number that received the payload
                print(
                    f"Received {len(data)} bytes on pipe {pipe_number} from node {node_id}.",
                    f"PayloadID: {payload_id}",
                )
                end_time = time.monotonic() + timeout  # reset timer with every payload

        # recommended behavior is to keep in TX mode while idle
        self.radio.as_tx()  # enter inactive TX mode

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
                "    Use 'T n' to transmit as node n; n must be in range [0, 5].\n"
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

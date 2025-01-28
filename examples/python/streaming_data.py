"""
Example of library usage for streaming multiple payloads.

See documentation at https://nRF24.github.io/rf24-rs
"""

import time
from rf24_py import RF24, PaLevel, StatusFlags, FifoState


def make_payloads(size: int = 32) -> list[bytes]:
    """return a list of payloads"""
    # we'll use `size` for the number of payloads in the list and the
    # payloads' length
    stream = []
    for i in range(size):
        # prefix payload with a sequential letter to indicate which
        # payloads were lost (if any)
        buff = bytes([i + (65 if 0 <= i < 26 else 71)])
        for j in range(size - 1):
            char = bool(j >= (size - 1) / 2 + abs((size - 1) / 2 - i))
            char |= bool(j < (size - 1) / 2 - abs((size - 1) / 2 - i))
            buff += bytes([char + 48])
        stream.append(buff)
    return stream


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

    def tx(self, count: int = 1, size: int = 32):
        """Uses all 3 levels of the TX FIFO via `RF24::write()`"""
        # minimum number of payloads in stream should be at least 6 for this example
        size = max(min(size, 32), 6)

        # save on transmission time by setting the radio to only transmit the
        # number of bytes we need to transmit
        self.radio.payload_length = size  # the default is the maximum 32 bytes

        # create a stream
        stream = make_payloads(size)

        self.radio.as_tx()  # ensures the nRF24L01 is in TX mode
        for cnt in range(count):  # transmit the same payloads this many times
            self.radio.flush_tx()  # clear the TX FIFO so we can use all 3 levels
            # NOTE the write_only parameter does not initiate sending
            buf_iter = 0  # iterator of payloads for the while loop
            failures = 0  # keep track of manual retries
            start_timer = time.monotonic() * 1000  # start timer
            for buf_index in range(size):  # cycle through all payloads in stream
                while not self.radio.write(stream[buf_iter]):
                    # upload to TX FIFO failed because TX FIFO is full.
                    # check for transmission errors
                    self.radio.update()
                    flags: StatusFlags = self.radio.get_status_flags()
                    if flags.tx_df:  # transmission failed
                        failures += 1  # increment manual retry count
                        # rewrite() resets the tx_df flag and reuses top level of TX FIFO
                        self.radio.rewrite()
                        if failures > 99:
                            break
                    if failures > 99 and buf_iter < 7 and cnt < 2:
                        # we need to prevent an infinite loop
                        print(
                            "Make sure slave() node is listening. Quitting master_fifo()"
                        )
                        buf_iter = size + 1  # be sure to exit the while loop
                        self.radio.flush_tx()  # discard all payloads in TX FIFO
                        break
                buf_iter += 1
            # wait for radio to finish transmitting everything in the TX FIFO
            while self.radio.get_fifo_state(True) != FifoState.Empty and failures < 99:
                # get_fifo_state() also update()s the StatusFlags
                flags = self.radio.get_status_flags()
                if flags.tx_df:
                    failures += 1
                    self.radio.rewrite()
            end_timer = time.monotonic() * 1000  # end timer
            print(
                f"Transmission took {end_timer - start_timer} ms with",
                f"{failures} failures detected.",
            )
        self.radio.as_tx()  # ensure radio exits active TX mode

    def rx(self, timeout: int = 5, size: int = 32):
        """Stops listening after a `timeout` with no response"""

        # save on transmission time by setting the radio to only transmit the
        #  number of bytes we need to transmit
        self.radio.payload_length = size  # the default is the maximum 32 bytes

        self.radio.as_rx()  # put radio into RX mode and power up
        count = 0  # keep track of the number of received payloads
        start_timer = time.monotonic()  # start timer
        while time.monotonic() < start_timer + timeout:
            if self.radio.available():
                count += 1
                # retrieve the received packet's payload
                receive_payload = self.radio.read(size)
                print(f"Received: {repr(receive_payload)} - {count}")
                start_timer = time.monotonic()  # reset timer on every RX payload

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
            self.rx(*[int(x) for x in user_input[1:3]])
            return True
        if user_input[0].upper().startswith("T"):
            self.tx(*[int(x) for x in user_input[1:3]])
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

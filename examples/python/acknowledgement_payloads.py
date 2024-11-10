"""
Simple example of using the library to transmit
and retrieve custom automatic acknowledgment payloads.

See documentation at https://nRF24.github.io/rf24-rs
"""

from pathlib import Path
import time
from rf24_py import RF24, PaLevel


class App:
    def __init__(self) -> None:
        # The radio's CE Pin uses a GPIO number.
        # On Linux, consider the device path `/dev/gpiochip<N>`:
        #   - `<N>` is the gpio chip's identifying number.
        #     Using RPi4 (or earlier), this number is `0` (the default).
        #     Using the RPi5, this number is actually `4`.
        # The radio's CE pin must connected to a pin exposed on the specified chip.
        ce_pin = 22  # for GPIO22
        # try detecting RPi5 first; fall back to default
        gpio_chip = 4 if Path("/dev/gpiochip4").exists() else 0

        # The radio's CSN Pin corresponds the SPI bus's CS pin (aka CE pin).
        # On Linux, consider the device path `/dev/spidev<a>.<b>`:
        #   - `<a>` is the SPI bus number (defaults to `0`)
        #   - `<b>` is the CSN pin (must be unique for each device on the same SPI bus)
        csn_pin = 0  # aka CE0 for SPI bus 0 (/dev/spidev0.0)

        # create a radio object for the specified hardware config:
        self.radio = RF24(ce_pin, csn_pin, dev_gpio_chip=gpio_chip)

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

        # ACK payloads are dynamically sized, so we need to enable that feature also
        self.radio.dynamic_payloads = True

        # to enable the custom ACK payload feature
        self.radio.ack_payloads = True

        # set TX address of RX node into the TX pipe
        self.radio.open_tx_pipe(address[radio_number])  # always uses pipe 0

        # set RX address of TX node into an RX pipe
        self.radio.open_rx_pipe(1, address[not radio_number])  # using pipe 1

        self.counter = 0

        # for debugging
        # self.radio.print_details()

    def tx(self, count: int = 5):  # count = 5 will only transmit 5 packets
        """Transmits a payload every second and prints the ACK payload"""
        self.radio.as_tx()  # put radio in TX mode

        while count:
            # construct a payload to send
            buffer = b"Hello \x00" + bytes([self.counter])

            # send the payload and prompt
            start_timer = time.monotonic_ns()  # start timer
            result = self.radio.send(buffer)  # save the report
            end_timer = time.monotonic_ns()  # stop timer
            if result:
                # print timer results upon transmission success
                print(
                    "Transmission successful! Time to transmit:",
                    f"{int((end_timer - start_timer) / 1000)} us. Sent:",
                    f"{buffer[:6].decode('utf-8')}{self.counter}",
                    end=" ",
                )
                if self.radio.available():
                    # print the received ACK that was automatically sent
                    response = self.radio.read()
                    print(
                        f" Received: {response[:6].decode('utf-8')}{response[7:8][0]}"
                    )
                    self.counter += 1  # increment payload counter
                else:
                    print(" Received an empty ACK packet")
            else:
                print("Transmission failed or timed out")
            time.sleep(1)  # let the RX node prepare a new ACK payload
            count -= 1

    def rx(self, timeout: int = 6):
        """Prints the received value and sends an ACK payload"""
        self.radio.as_rx()  # put radio into RX mode, power it up

        # setup the first transmission's ACK payload
        buffer = b"World \x00" + bytes([self.counter])
        # we must set the ACK payload data and corresponding
        # pipe number [0,5]
        self.radio.write_ack_payload(1, buffer)  # load ACK for first response

        start = time.monotonic()  # start timer
        while (time.monotonic() - start) < timeout:
            has_payload, pipe_number = self.radio.available_pipe()
            if has_payload:
                received = self.radio.read()  # fetch 1 payload from RX FIFO
                print(
                    f"Received {len(received)} bytes on pipe {pipe_number}:",
                    f"{received[:6].decode('utf-8')}{received[7:8][0]} Sent:",
                    f"{buffer[:6].decode('utf-8')}{self.counter}",
                )
                start = time.monotonic()  # reset timer

                # increment counter from received payload
                self.counter = received[7:8][0] + 1
                # build a new ACK payload
                buffer = b"World \x00" + bytes([self.counter])
                self.radio.write_ack_payload(1, buffer)  # load ACK for next response

        # recommended behavior is to keep in TX mode while idle
        self.radio.as_tx()  # put radio in TX mode & flush unused ACK payloads

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

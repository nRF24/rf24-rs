"""
Simple example of detecting (and verifying) the IRQ (interrupt) pin on the
nRF24L01.

This example is meant to be run on 2 separate nRF24L01 transceivers.

This example requires gpiod lib to monitor the radio's IRQ pin.

See documentation at https://nRF24.github.io/rf24-rs
"""

import time
from rf24_py import RF24, PaLevel, FifoState, StatusFlags

try:
    import gpiod  # type: ignore[import-untyped,import-not-found]
    from gpiod.line import Edge  # type: ignore[import-untyped,import-not-found]
except ImportError as exc:
    raise ImportError(
        "This script requires gpiod installed for observing the IRQ pin. Please run\n"
        "\n    uv run --no-dev --with gpiod examples/python/irq_config.py\n\nMore "
        "details at https://pypi.org/project/gpiod/"
    ) from exc


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

        gpio_chip = 0  # change this number as needed (according to your system)
        chip = gpiod.Chip(f"/dev/gpiochip{gpio_chip}")
        # print gpio chip info
        info = chip.get_info()
        print(f"Using {info.name} [{info.label}] ({info.num_lines} lines)")

        # select your digital input pin that's connected to the IRQ pin on the nRF24L01
        self.irq_pin = 24

        # setup IRQ GPIO pin
        self.irq_line = gpiod.request_lines(
            path=f"/dev/gpiochip{gpio_chip}",
            consumer="rf24_py/examples/interrupt",  # optional
            config={self.irq_pin: gpiod.LineSettings(edge_detection=Edge.FALLING)},
        )

        # For this example, we will use different addresses
        # An address need to be a buffer protocol object (bytearray)
        address = [b"1Node", b"2Node"]
        # It is very helpful to think of an address as a path instead of as
        # an identifying device destination

        # to use different addresses on a pair of radios, we need a variable to
        # uniquely identify which address this radio will use to transmit
        # 0 uses address[0] to transmit, 1 uses address[1] to transmit
        radio_number = bool(
            int(input("Which radio is this? Enter '0' or '1'. Defaults to '0' ") or 0)
        )

        # set TX address of RX node (always uses pipe 0)
        self.radio.as_tx(address[radio_number])  # enter inactive TX mode

        # set RX address of TX node into an RX pipe
        self.radio.open_rx_pipe(1, address[not radio_number])  # using pipe 1

        # this example uses the ACK payload to trigger the IRQ pin active for
        # the "on data received" event
        self.radio.ack_payloads = True  # enable ACK payloads
        self.radio.dynamic_payloads = True  # ACK payloads are dynamically sized

        # set the Power Amplifier level to -12 dBm since this test example is
        # usually run with nRF24L01 transceivers in close proximity of each other
        self.radio.pa_level = PaLevel.Low  # PaLevel.Max is default

        # for debugging
        # self.radio.print_details()

        # For this example, we'll be using a payload containing
        # a string that changes on every transmission. (successful or not)
        # Make a couple tuples of payloads & an iterator to traverse them
        self.pl_iterator = 0

    def _interrupt_handler(self) -> None:
        """This function is called when IRQ pin is detected active LOW"""
        print("\tIRQ pin went active LOW.")
        self.radio.update()
        flags: StatusFlags = self.radio.get_status_flags()  # update IRQ status flags
        print(f"\t{repr(flags)}")
        if self.pl_iterator == 0:
            print("'data ready' event test", ("passed" if flags.rx_dr else "failed"))
        elif self.pl_iterator == 1:
            print("'data sent' event test", ("passed" if flags.tx_ds else "failed"))
        elif self.pl_iterator == 2:
            print("'data fail' event test", ("passed" if flags.tx_df else "failed"))
        self.radio.clear_status_flags()

    def _wait_for_irq(self, timeout: float = 5) -> bool:
        """Wait till IRQ pin goes active (LOW).
        IRQ pin is LOW when activated. Otherwise it is always HIGH
        """
        # wait up to ``timeout`` seconds for event to be detected.
        if not self.irq_line.wait_edge_events(timeout):
            print(f"\tInterrupt event not detected for {timeout} seconds!")
            return False
        # read event from kernel buffer
        for event in self.irq_line.read_edge_events():
            if (
                event.line_offset == self.irq_pin
                and event.event_type is event.Type.FALLING_EDGE
            ):
                return True
        return False

    def tx(self) -> None:
        """Transmits 4 times and reports results

        1. successfully receive ACK payload first
        2. successfully transmit on second
        3. send a third payload to fill RX node's RX FIFO
        (supposedly making RX node unresponsive)
        4. intentionally fail transmit on the fourth
        """

        tx_payloads = (b"Ping ", b"Pong ", b"Radio", b"1FAIL")

        self.radio.as_tx()  # put radio in TX mode

        # on data ready test
        print("\nConfiguring IRQ pin to only ignore 'on data sent' event")
        self.radio.set_status_flags(StatusFlags(rx_dr=True, tx_ds=False, tx_df=True))
        print("    Pinging slave node for an ACK payload...")
        self.pl_iterator = 0
        if not self.radio.write(tx_payloads[0]):
            print("Failed to upload payload to TX FIFO")
        elif self._wait_for_irq():
            self._interrupt_handler()

        # on "data sent" test
        print("\nConfiguring IRQ pin to only ignore 'on data ready' event")
        self.radio.set_status_flags(StatusFlags(rx_dr=False, tx_ds=True, tx_df=True))
        print("    Pinging slave node again...")
        self.pl_iterator = 1
        if not self.radio.write(tx_payloads[1]):
            print("Failed to upload payload to TX FIFO")
        elif self._wait_for_irq():
            self._interrupt_handler()

        # trigger slave node to exit by filling the slave node's RX FIFO
        print("\nSending one extra payload to fill RX FIFO on slave node.")
        print("Disabling IRQ pin for all events.")
        self.radio.set_status_flags(StatusFlags())
        if self.radio.send(tx_payloads[2]):
            print("Slave node should not be listening anymore.")
        else:
            print("Slave node was unresponsive.")
        self.radio.clear_status_flags()

        # on "data fail" test
        print("\nConfiguring IRQ pin to go active for all events.")
        self.radio.set_status_flags(StatusFlags(rx_dr=True, tx_ds=True, tx_df=True))
        print("    Sending a ping to inactive slave node...")
        self.radio.flush_tx()  # just in case any previous tests failed
        self.pl_iterator = 2
        if not self.radio.write(tx_payloads[3]):
            print("Failed to upload payload to TX FIFO")
        elif self._wait_for_irq():
            self._interrupt_handler()
        self.radio.flush_tx()  # flush artifact payload in TX FIFO from last test
        # All 3 ACK payloads received were 4 bytes each, and RX FIFO is full.
        # So, fetching 12 bytes from the RX FIFO also flushes RX FIFO.
        print("\nComplete RX FIFO:", self.radio.read(12))

        # recommended behavior is to keep in TX mode while idle
        self.radio.as_tx()  # enter inactive TX mode

    def rx(self, timeout=6):  # will listen for 6 seconds before timing out
        """Only listen for 3 payload from the master node"""
        # the "data ready" event will trigger in RX mode
        # the "data sent" or "data fail" events will trigger when we
        # receive with ACK payloads enabled (& loaded in TX FIFO)
        print("\nDisabling IRQ pin for all events.")
        self.radio.set_status_flags(StatusFlags())
        # fill TX FIFO with ACK payloads
        ack_payloads = (b"Yak ", b"Back", b" ACK")
        for ack in ack_payloads:
            self.radio.write_ack_payload(1, ack)

        self.radio.as_rx()  # start listening & clear irq_dr flag
        end_time = time.monotonic() + timeout  # set end time
        while (
            self.radio.get_fifo_state(False) != FifoState.Full
            and time.monotonic() < end_time
        ):
            # wait for RX FIFO to fill up or until timeout is reached
            pass
        time.sleep(0.5)  # wait for last ACK payload to transmit

        # exit RX mode
        # recommended behavior is to keep in TX mode while idle
        self.radio.as_tx()  # enter inactive TX mode
        # as_tx() will also flush unused ACK payloads
        # when ACK payloads are enabled

        if self.radio.available():  # if RX FIFO is not empty (timeout did not occur)
            # All 3 payloads received were 5 bytes each, and RX FIFO is full.
            # So, fetching 15 bytes from the RX FIFO also flushes RX FIFO.
            print("Complete RX FIFO:", self.radio.read(15))

    def set_role(self) -> bool:
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
        ).split()
        if user_input[0].upper().startswith("R"):
            self.rx(*[int(x) for x in user_input[1:2]])
            return True
        if user_input[0].upper().startswith("T"):
            self.tx()
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

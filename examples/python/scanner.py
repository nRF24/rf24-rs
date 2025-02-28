"""
This is an example of how to use the nRF24L01's builtin
Received Power Detection (RPD) to scan for possible interference.
This example does not require a counterpart node.

See documentation at https://nRF24.github.io/rf24-rs
"""

import time
from typing import Optional
from rf24_py import RF24, CrcLength, FifoState, DataRate

print(__file__)  # print example name


class App:
    def __init__(self, data_rate: DataRate) -> None:
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

        # turn off RX features specific to the nRF24L01 module
        self.radio.set_auto_ack(False)
        self.radio.dynamic_payloads = False
        self.radio.crc_length = CrcLength.Disabled
        self.radio.data_rate = data_rate

        # use reverse engineering tactics for a better "snapshot"
        self.radio.address_length = 2
        # The worst possible addresses. These are designed to confuse the radio into thinking
        # the RF signal's preamble is part of the packet/payload.
        noise_addresses = [
            b"\x55\x55",
            b"\xaa\xaa",
            b"\xa0\xaa",
            b"\x0a\xaa",
            b"\xa5\xaa",
            b"\x5a\xaa",
        ]
        for pipe, address in enumerate(noise_addresses):
            self.radio.open_rx_pipe(pipe, address)

    def scan(self, timeout: int = 30):
        """Traverse the spectrum of accessible frequencies and print any detection
        of ambient signals.

        :param int timeout: The number of seconds in which scanning is performed.
        """
        # print the vertical header of channel numbers
        print("0" * 100 + "1" * 26)
        for i in range(13):
            print(str(i % 10) * (10 if i < 12 else 6), sep="", end="")
        print("")  # endl
        for i in range(126):
            print(str(i % 10), sep="", end="")
        print("\n" + "~" * 126)

        signals = [0] * 126  # store the signal count for each channel
        sweeps = 0  # keep track of the number of sweeps made through all channels
        curr_channel = 0
        end_time = time.monotonic() + timeout  # start the timer
        while time.monotonic() < end_time:
            self.radio.channel = curr_channel
            self.radio.as_rx()  # start a RX session
            time.sleep(0.00013)  # wait 130 microseconds
            found_signal = self.radio.rpd
            self.radio.as_tx()  # end the RX session
            found_signal = found_signal or self.radio.rpd or self.radio.available()

            # count signal as interference
            signals[curr_channel] += found_signal
            # clear the RX FIFO if a signal was detected/captured
            if found_signal:
                self.radio.flush_rx()  # flush the RX FIFO because it asserts the RPD flag
            endl = False
            if curr_channel >= 124:
                sweeps += 1
                if int(sweeps / 100) > 0:
                    endl = True
                    sweeps = 0

            # output the signal counts per channel
            sig_cnt = signals[curr_channel]
            print(
                ("%X" % min(15, sig_cnt)) if sig_cnt else "-",
                sep="",
                end="" if curr_channel < 125 else ("\n" if endl else "\r"),
            )
            curr_channel = curr_channel + 1 if curr_channel < 125 else 0
            if endl:
                signals = [0] * 126  # reset the signal counts for new line

        # finish printing results and end with a new line
        while curr_channel < len(signals) - 1:
            curr_channel += 1
            sig_cnt = signals[curr_channel]
            print(("%X" % min(15, sig_cnt)) if sig_cnt else "-", sep="", end="")
        print("")

    def noise(self, timeout: float = 1, channel: Optional[int] = None):
        """print a stream of detected noise for duration of time.

        :param float timeout: The number of seconds to scan for ambient noise.
        :param int channel: The specific channel to focus on. If not provided, then the
            radio's current setting is used.
        """

        def hex_data_str(data: bytes) -> str:
            return " ".join(["%02x" % b for b in data])

        if channel is not None:
            self.radio.channel = channel
        self.radio.as_rx()
        timeout += time.monotonic()
        while time.monotonic() < timeout:
            signal = self.radio.read()
            if signal:
                print(hex_data_str(signal))
        self.radio.as_tx()
        while self.radio.get_fifo_state(about_tx=False) != FifoState.Empty:
            # dump the left overs in the RX FIFO
            print(hex_data_str(self.radio.read()))

    def set_role(self):
        """Set the role using stdin stream. Timeout arg for scan() can be
        specified using a space delimiter (e.g. 'S 10' calls `scan(10)`)
        """
        user_input = (
            input(
                "*** Enter 'S' to perform scan.\n"
                "*** Enter 'N' to display noise.\n"
                "*** Enter 'Q' to quit example.\n"
            )
            or "?"
        )
        user_input = user_input.split()
        if user_input[0].upper().startswith("S"):
            self.scan(*[int(x) for x in user_input[1:2]])
            return True
        if user_input[0].upper().startswith("N"):
            self.noise(*[int(x) for x in user_input[1:3]])
            return True
        if user_input[0].upper().startswith("Q"):
            self.radio.power = False
            return False
        print(user_input[0], "is an unrecognized input. Please try again.")
        return True


if __name__ == "__main__":
    print(__file__)
    print(
        "!!!Make sure the terminal is wide enough for 126 characters on 1 line."
        " If this line is wrapped, then the output will look bad!"
    )
    print(
        "\nSelect the desired DataRate: (defaults to 1 Mbps)\n"
        "1. 1 Mbps\n2. 2 Mbps\n3. 250 Kbps\n"
    )
    d_rate = input().strip()
    data_rate = (
        DataRate.Mbps2
        if d_rate.startswith("2")
        else (DataRate.Kbps250 if d_rate.startswith("3") else DataRate.Mbps1)
    )
    app = App(data_rate)
    try:
        while app.set_role():
            pass  # continue example until 'Q' is entered
    except KeyboardInterrupt:
        print(" Keyboard Interrupt detected. Powering down radio...")
        app.radio.power = False

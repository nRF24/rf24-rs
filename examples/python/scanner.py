"""
This is an example of how to use the nRF24L01's builtin
Received Power Detection (RPD) to scan for possible interference.
This example does not require a counterpart node.

See documentation at https://nRF24.github.io/rf24-rs
"""

from pathlib import Path
import time
from typing import Optional
from rf24_py import RF24, CrcLength, FifoState

print(__file__)  # print example name

# The radio's CE Pin uses a GPIO number.
# On Linux, consider the device path `/dev/gpiochip<N>`:
#   - `<N>` is the gpio chip's identifying number.
#     Using RPi4 (or earlier), this number is `0` (the default).
#     Using the RPi5, this number is actually `4`.
# The radio's CE pin must connected to a pin exposed on the specified chip.
CE_PIN = 22  # for GPIO22
# try detecting RPi5 first; fall back to default
DEV_GPIO_CHIP = 4 if Path("/dev/gpiochip4").exists() else 0

# The radio's CSN Pin corresponds the SPI bus's CS pin (aka CE pin).
# On Linux, consider the device path `/dev/spidev<a>.<b>`:
#   - `<a>` is the SPI bus number (defaults to `0`)
#   - `<b>` is the CSN pin (must be unique for each device on the same SPI bus)
CSN_PIN = 0  # aka CE0 for SPI bus 0 (/dev/spidev0.0)

# create a radio object for the specified hardware config:
radio = RF24(CE_PIN, CSN_PIN, dev_gpio_chip=DEV_GPIO_CHIP)

# initialize the nRF24L01 on the spi bus
radio.begin()

# turn off RX features specific to the nRF24L01 module
radio.set_auto_ack(False)
radio.set_dynamic_payloads(False)
radio.crc_length = CrcLength.Disabled
radio.set_auto_retries(0, 0)

# use reverse engineering tactics for a better "snapshot"
radio.address_length = 2
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
    radio.open_rx_pipe(pipe, address)


def scan(timeout: int = 30):
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
    start_timer = time.monotonic()  # start the timer
    while time.monotonic() - start_timer < timeout:
        radio.channel = curr_channel
        radio.listen = True  # start a RX session
        time.sleep(0.00013)  # wait 130 microseconds
        found_signal = radio.rpd
        radio.listen = False  # end the RX session
        found_signal = found_signal or radio.rpd or radio.available()

        # count signal as interference
        signals[curr_channel] += found_signal
        # clear the RX FIFO if a signal was detected/captured
        if found_signal:
            radio.flush_rx()  # flush the RX FIFO because it asserts the RPD flag
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


def hex_data_str(data: bytes) -> str:
    return " ".join(["%02x" % b for b in data])


def noise(timeout: float = 1, channel: Optional[int] = None):
    """print a stream of detected noise for duration of time.

    :param float timeout: The number of seconds to scan for ambient noise.
    :param int channel: The specific channel to focus on. If not provided, then the
        radio's current setting is used.
    """
    if channel is not None:
        radio.channel = channel
    radio.listen = True
    timeout += time.monotonic()
    while time.monotonic() < timeout:
        signal = radio.read()
        if signal:
            print(hex_data_str(signal))
    radio.listen = False
    while not radio.get_fifo_state(about_tx=False) != FifoState.Full:
        # dump the left overs in the RX FIFO
        print(hex_data_str(radio.read()))


def set_role():
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
        scan(*[int(x) for x in user_input[1:2]])
        return True
    if user_input[0].upper().startswith("N"):
        noise(*[int(x) for x in user_input[1:3]])
        return True
    if user_input[0].upper().startswith("Q"):
        radio.power = False
        return False
    print(user_input[0], "is an unrecognized input. Please try again.")
    return True


print("    nRF24L01 scanner test")
print(
    "!!!Make sure the terminal is wide enough for 126 characters on 1 line."
    " If this line is wrapped, then the output will look bad!"
)

if __name__ == "__main__":
    try:
        while set_role():
            pass  # continue example until 'Q' is entered
    except KeyboardInterrupt:
        print(" Keyboard Interrupt detected. Powering down radio...")
        radio.power = False
else:
    print("    Run scan() to initiate scan for ambient signals.")
    print("    Run noise() to display ambient signals' data (AKA noise).")

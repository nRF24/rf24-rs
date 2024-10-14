"""
Simple example of using the RF24 class.

See documentation at https://nRF24.github.io/rf24-rs
"""

from pathlib import Path
import time
import struct
from rf24_py import RF24, PaLevel

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

# create a radio object for the specified hard ware config:
radio = RF24(CE_PIN, CSN_PIN, dev_gpio_chip=DEV_GPIO_CHIP)

# using the python keyword global is bad practice. Instead we'll use a 1 item
# list to store our float number for the payloads sent
payload = [0.0]

# For this example, we will use different addresses
# An address needs to be a buffer protocol object (bytearray)
address = [b"1Node", b"2Node"]
# It is very helpful to think of an address as a path instead of as
# an identifying device destination

# to use different addresses on a pair of radios, we need a variable to
# uniquely identify which address this radio will use to transmit
# 0 uses address[0] to transmit, 1 uses address[1] to transmit
radio_number = bool(
    int(input("Which radio is this? Enter '0' or '1'. Defaults to '0' ") or 0)
)

# initialize the nRF24L01 on the spi bus
radio.begin()

# set the Power Amplifier level to -12 dBm since this test example is
# usually run with nRF24L01 transceivers in close proximity of each other
radio.pa_level = PaLevel.LOW  # PaLevel.MAX is default

# set TX address of RX node into the TX pipe
radio.open_tx_pipe(address[radio_number])  # always uses pipe 0

# set RX address of TX node into an RX pipe
radio.open_rx_pipe(1, address[not radio_number])  # using pipe 1

# To save time during transmission, we'll set the payload size to be only what
# we need. A float value occupies 4 bytes in memory using struct.calcsize()
# "<f" means a little endian unsigned float
radio.payload_length = struct.calcsize("<f")

# for debugging
# radio.print_details()
# or for human readable data
# radio.print_pretty_details()


def master(count: int = 5):  # count = 5 will only transmit 5 packets
    """Transmits an incrementing float every second"""
    radio.listen = False  # ensures the nRF24L01 is in TX mode

    while count:
        # use struct.pack() to pack your data into a usable payload
        # into a usable payload
        buffer = struct.pack("<f", payload[0])
        # "<f" means a single little endian (4 byte) float value.
        start_timer = time.monotonic_ns()  # start timer
        result = radio.send(buffer)
        end_timer = time.monotonic_ns()  # end timer
        if not result:
            print("Transmission failed or timed out")
        else:
            print(
                "Transmission successful! Time to Transmit:",
                f"{(end_timer - start_timer) / 1000} us. Sent: {payload[0]}",
            )
            payload[0] += 0.01
        time.sleep(1)
        count -= 1


def slave(timeout: int = 6):
    """Polls the radio and prints the received value. This method expires
    after 6 seconds of no received transmission."""
    radio.listen = True  # put radio into RX mode and power up

    start = time.monotonic()
    while (time.monotonic() - start) < timeout:
        has_payload, pipe_number = radio.available_pipe()
        if has_payload:
            # fetch 1 payload from RX FIFO
            received = radio.read()  # also clears radio.irq_dr status flag
            # expecting a little endian float, thus the format string "<f"
            # received[:4] truncates padded 0s in case dynamic payloads are disabled
            payload[0] = struct.unpack("<f", received[:4])[0]
            # print details about the received packet
            print(f"Received {len(received)} bytes on pipe {pipe_number}: {payload[0]}")
            start = time.monotonic()  # reset the timeout timer

    # recommended behavior is to keep in TX mode while idle
    radio.listen = False  # put the nRF24L01 is in TX mode


def set_role():
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
        slave(*[int(x) for x in user_input[1:2]])
        return True
    if user_input[0].upper().startswith("T"):
        master(*[int(x) for x in user_input[1:2]])
        return True
    if user_input[0].upper().startswith("Q"):
        radio.power = False
        return False
    print(user_input[0], "is an unrecognized input. Please try again.")
    return True


if __name__ == "__main__":
    try:
        while set_role():
            pass  # continue example until 'Q' is entered
    except KeyboardInterrupt:
        print(" Keyboard Interrupt detected. Exiting...")
        radio.power = False
else:
    print("    Run slave() on receiver\n    Run master() on transmitter")

"""
Simple example of using the library to transmit
and retrieve custom automatic acknowledgment payloads.

See documentation at https://nRF24.github.io/rf24-rs
"""

from pathlib import Path
import time
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

# create a radio object for the specified hardware config:
radio = RF24(CE_PIN, CSN_PIN, dev_gpio_chip=DEV_GPIO_CHIP)

# using the python keyword global is bad practice. Instead we'll use a 1 item
# list to store our integer number for the payloads' counter
counter = [0]

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

# initialize the nRF24L01 on the spi bus
radio.begin()

# set the Power Amplifier level to -12 dBm since this test example is
# usually run with nRF24L01 transceivers in close proximity of each other
radio.pa_level = PaLevel.Low  # PaLevel.Max is default

# ACK payloads are dynamically sized, so we need to enable that feature also
radio.set_dynamic_payloads(True)

# to enable the custom ACK payload feature
radio.allow_ack_payloads(True)

# set TX address of RX node into the TX pipe
radio.open_tx_pipe(address[radio_number])  # always uses pipe 0

# set RX address of TX node into an RX pipe
radio.open_rx_pipe(1, address[not radio_number])  # using pipe 1

# for debugging
# radio.print_details()


def master(count: int = 5):  # count = 5 will only transmit 5 packets
    """Transmits a payload every second and prints the ACK payload"""
    radio.listen = False  # put radio in TX mode

    while count:
        # construct a payload to send
        buffer = b"Hello \x00" + bytes([counter[0]])

        # send the payload and prompt
        start_timer = time.monotonic_ns()  # start timer
        result = radio.send(buffer)  # save the report
        end_timer = time.monotonic_ns()  # stop timer
        if result:
            # print timer results upon transmission success
            print(
                "Transmission successful! Time to transmit:",
                f"{int((end_timer - start_timer) / 1000)} us. Sent:",
                f"{buffer[:6].decode('utf-8')}{counter[0]}",
                end=" ",
            )
            if radio.available():
                # print the received ACK that was automatically sent
                response = radio.read()
                print(f" Received: {response[:6].decode('utf-8')}{response[7:8][0]}")
                counter[0] += 1  # increment payload counter
            else:
                print(" Received an empty ACK packet")
        else:
            print("Transmission failed or timed out")
        time.sleep(1)  # let the RX node prepare a new ACK payload
        count -= 1


def slave(timeout: int = 6):
    """Prints the received value and sends an ACK payload"""
    radio.listen = True  # put radio into RX mode, power it up

    # setup the first transmission's ACK payload
    buffer = b"World \x00" + bytes([counter[0]])
    # we must set the ACK payload data and corresponding
    # pipe number [0,5]
    radio.write_ack_payload(1, buffer)  # load ACK for first response

    start = time.monotonic()  # start timer
    while (time.monotonic() - start) < timeout:
        has_payload, pipe_number = radio.available_pipe()
        if has_payload:
            received = radio.read()  # fetch 1 payload from RX FIFO
            # increment counter from received payload
            counter[0] = received[7:8][0] + 1
            print(
                f"Received {len(received)} bytes on pipe {pipe_number}:",
                f"{received[:6].decode('utf-8')}{received[7:8][0]} Sent:",
                f"{buffer[:6].decode('utf-8')}{counter[0]}",
            )
            start = time.monotonic()  # reset timer

            # build a new ACK payload
            buffer = b"World \x00" + bytes([counter[0]])
            radio.write_ack_payload(1, buffer)  # load ACK for next response

    # recommended behavior is to keep in TX mode while idle
    radio.listen = False  # put radio in TX mode & flush unused ACK payloads


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

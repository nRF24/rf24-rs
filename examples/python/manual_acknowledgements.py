"""
Simple example of using the RF24 class to transmit and respond with
acknowledgment (ACK) transmissions. Notice that the auto-ack feature is
enabled, but this example doesn't use automatic ACK payloads because automatic
ACK payloads' data will always be outdated by 1 transmission. Instead, this
example uses a call and response paradigm.

See documentation at https://nRF24.github.io/rf24-rs
"""

from pathlib import Path
import struct
import time
from rf24_py import RF24, PaLevel, StatusFlags

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
radio.begin()

# set the Power Amplifier level to -12 dBm since this test example is
# usually run with nRF24L01 transceivers in close proximity of each other
radio.pa_level = PaLevel.Low  # PaLevel.Max is default

# set TX address of RX node into the TX pipe
radio.open_tx_pipe(address[radio_number])  # always uses pipe 0

# set RX address of TX node into an RX pipe
radio.open_rx_pipe(1, address[not radio_number])  # using pipe 1

# To save time during transmission, we'll set the payload size to be only what
# we need.
# "<b" means a little endian unsigned byte
# we also need an addition 7 bytes for the payload message
radio.payload_length = struct.calcsize("<b") + 7

# for debugging
# radio.print_details()

# using the python keyword global is bad practice. Instead we'll use a 1 item
# list to store our float number for the payloads sent
payload = [0]


def master(count: int = 10):
    """Transmits a message and an incrementing integer every second"""
    radio.listen = False  # ensures the nRF24L01 is in TX mode

    while count:  # only transmit `count` packets
        # use struct.pack() to pack your data into a usable payload
        # "<b" means a single little endian unsigned byte.
        # NOTE we added a b"\x00" byte as a c-string's NULL terminating 0
        buffer = b"Hello \x00" + struct.pack("<b", payload[0])
        start_timer = time.monotonic_ns()  # start timer
        result = radio.send(buffer)
        if not result:
            print("Transmission failed or timed out")
        else:
            radio.listen = True
            timeout = time.monotonic() * 1000 + 200  # use 200 ms timeout
            ack = b"\x00" * len(buffer)  # variable used for the response
            while ack[0] == 0 and time.monotonic() * 1000 < timeout:
                if radio.available():
                    # get the response & save it to ack variable
                    ack = radio.read()
            end_timer = time.monotonic_ns()  # end timer
            radio.listen = False
            print(
                "Transmission successful. Sent: ",
                f"{buffer[:6].decode('utf-8')}{payload[0]}.",
                end=" ",
            )
            if ack[0] == 0:
                print("No response received.")
            else:
                # decode response's text as an string
                # NOTE ack[:6] ignores the NULL terminating 0
                response = ack[:6].decode("utf-8")
                # use struct.unpack() to get the response's appended int
                # NOTE ack[7:] discards NULL terminating 0, and
                # "<b" means its a single little endian unsigned byte
                payload[0] = struct.unpack("<b", ack[7:])[0]
                print(
                    f"Received: {response}{payload[0]}. Roundtrip delay:",
                    f"{(end_timer - start_timer) / 1000} us.",
                )
        time.sleep(1)  # make example readable by slowing down transmissions
        count -= 1


def slave(timeout: int = 6):
    """Polls the radio and prints the received value. This method expires
    after 6 seconds of no received transmission"""
    radio.listen = True  # put radio into RX mode and power up

    start_timer = time.monotonic()  # start a timer to detect timeout
    while (time.monotonic() - start_timer) < timeout:
        # receive payloads or wait 6 seconds till timing out
        has_payload, pipe_number = radio.available_pipe()
        if has_payload:
            received = radio.read()  # fetch 1 payload from RX FIFO
            # use struct.unpack() to get the payload's appended int
            # NOTE received[7:] discards NULL terminating 0, and
            # "<b" means its a single little endian unsigned byte
            payload[0] = struct.unpack("<b", received[7:])[0] + 1
            # use bytes() to pack our data into a usable payload
            # NOTE b"\x00" byte is a c-string's NULL terminating 0
            buffer = b"World \x00" + bytes([payload[0]])
            radio.listen = False  # set radio to TX mode
            radio.write(buffer)  # load payload into radio's RX buffer
            # keep retrying to send response for 150 milliseconds
            response_timeout = time.monotonic_ns() + 150000000
            response_result = False
            while time.monotonic_ns() < response_timeout:
                radio.update()
                flags: StatusFlags = radio.get_status_flags()
                if flags.tx_ds:
                    response_result = True
                    break
                if flags.tx_df:
                    radio.rewrite()
            radio.listen = True  # set radio back into RX mode
            # print the payload received and the response's payload
            print(
                f"Received {len(received)} bytes on pipe {pipe_number}:",
                f"{received[:6].decode('utf-8')}{payload[0] - 1}.",
                end=" ",
            )
            if response_result:
                print(f"Sent: {buffer[:6].decode('utf-8')}{payload[0]}")
            else:
                radio.flush_tx()
                print("Response failed or timed out")
            start_timer = time.monotonic()  # reset the timeout timer

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

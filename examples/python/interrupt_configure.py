"""
Simple example of detecting (and verifying) the IRQ (interrupt) pin on the
nRF24L01

See documentation at https://nRF24.github.io/rf24-rs
"""

from pathlib import Path
import time
from rf24_py import RF24, PaLevel, FifoState, StatusFlags

try:
    import gpiod  # type: ignore[import-untyped]
    from gpiod.line import Edge  # type: ignore[import-untyped]
except ImportError as exc:
    raise ImportError(
        "This script requires gpiod installed for observing the IRQ pin. Please run\n"
        "\n    pip install gpiod\n\nMore details at https://pypi.org/project/gpiod/"
    ) from exc

print(__file__)  # print example name


########### USER CONFIGURATION ###########
# The radio's CE Pin uses a GPIO number.
# On Linux, consider the device path `/dev/gpiochip<N>`:
#   - `<N>` is the gpio chip's identifying number.
#     Using RPi (before RPi5), this number is `0` (the default).
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

# select your digital input pin that's connected to the IRQ pin on the nRF24L01
IRQ_PIN = 24
chip = gpiod.Chip(f"/dev/gpiochip{DEV_GPIO_CHIP}")
# print gpio chip info
info = chip.get_info()
print(f"Using {info.name} [{info.label}] ({info.num_lines} lines)")

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

# this example uses the ACK payload to trigger the IRQ pin active for
# the "on data received" event
radio.allow_ack_payloads(True)  # enable ACK payloads
radio.set_dynamic_payloads(True)  # ACK payloads are dynamically sized

# set the Power Amplifier level to -12 dBm since this test example is
# usually run with nRF24L01 transceivers in close proximity of each other
radio.pa_level = PaLevel.Low  # PaLevel.Max is default

# set TX address of RX node into the TX pipe
radio.open_tx_pipe(address[radio_number])  # always uses pipe 0

# set RX address of TX node into an RX pipe
radio.open_rx_pipe(1, address[not radio_number])  # using pipe 1

# for debugging
# radio.print_details()

# For this example, we'll be using a payload containing
# a string that changes on every transmission. (successful or not)
# Make a couple tuples of payloads & an iterator to traverse them
pl_iterator = [0]  # use a 1-item list instead of python's global keyword
tx_payloads = (b"Ping ", b"Pong ", b"Radio", b"1FAIL")
ack_payloads = (b"Yak ", b"Back", b" ACK")


def interrupt_handler() -> None:
    """This function is called when IRQ pin is detected active LOW"""
    print("\tIRQ pin went active LOW.")
    radio.update()
    flags: StatusFlags = radio.get_status_flags()  # update IRQ status flags
    print(f"\t{repr(flags)}")
    if pl_iterator[0] == 0:
        print("'data ready' event test", ("passed" if flags.rx_dr else "failed"))
    elif pl_iterator[0] == 1:
        print("'data sent' event test", ("passed" if flags.tx_ds else "failed"))
    elif pl_iterator[0] == 2:
        print("'data fail' event test", ("passed" if flags.tx_df else "failed"))
    radio.clear_status_flags()


# setup IRQ GPIO pin
irq_line = gpiod.request_lines(
    path=f"/dev/gpiochip{DEV_GPIO_CHIP}",
    consumer="rf24_py/examples/interrupt",  # optional
    config={IRQ_PIN: gpiod.LineSettings(edge_detection=Edge.FALLING)},
)


def _wait_for_irq(timeout: float = 5) -> bool:
    """Wait till IRQ_PIN goes active (LOW).
    IRQ pin is LOW when activated. Otherwise it is always HIGH
    """
    # wait up to ``timeout`` seconds for event to be detected.
    if not irq_line.wait_edge_events(timeout):
        print(f"\tInterrupt event not detected for {timeout} seconds!")
        return False
    # read event from kernel buffer
    for event in irq_line.read_edge_events():
        if event.line_offset == IRQ_PIN and event.event_type is event.Type.FALLING_EDGE:
            return True
    return False


def master() -> None:
    """Transmits 4 times and reports results

    1. successfully receive ACK payload first
    2. successfully transmit on second
    3. send a third payload to fill RX node's RX FIFO
       (supposedly making RX node unresponsive)
    4. intentionally fail transmit on the fourth
    """
    radio.listen = False  # put radio in TX mode

    # on data ready test
    print("\nConfiguring IRQ pin to only ignore 'on data sent' event")
    radio.set_status_flags(StatusFlags(rx_dr=True, tx_ds=False, tx_df=True))
    print("    Pinging slave node for an ACK payload...")
    pl_iterator[0] = 0
    radio.write(tx_payloads[0])
    if _wait_for_irq():
        interrupt_handler()

    # on "data sent" test
    print("\nConfiguring IRQ pin to only ignore 'on data ready' event")
    radio.set_status_flags(StatusFlags(rx_dr=False, tx_ds=True, tx_df=True))
    print("    Pinging slave node again...")
    pl_iterator[0] = 1
    radio.write(tx_payloads[1])
    if _wait_for_irq():
        interrupt_handler()

    # trigger slave node to exit by filling the slave node's RX FIFO
    print("\nSending one extra payload to fill RX FIFO on slave node.")
    print("Disabling IRQ pin for all events.")
    radio.set_status_flags(StatusFlags())
    if radio.send(tx_payloads[2]):
        print("Slave node should not be listening anymore.")
    else:
        print("Slave node was unresponsive.")
    radio.clear_status_flags()

    # on "data fail" test
    print("\nConfiguring IRQ pin to go active for all events.")
    radio.set_status_flags(StatusFlags(rx_dr=True, tx_ds=True, tx_df=True))
    print("    Sending a ping to inactive slave node...")
    radio.flush_tx()  # just in case any previous tests failed
    pl_iterator[0] = 2
    radio.write(tx_payloads[3])
    if _wait_for_irq():
        interrupt_handler()
    radio.flush_tx()  # flush artifact payload in TX FIFO from last test
    # all 3 ACK payloads received were 4 bytes each, and RX FIFO is full
    # so, fetching 12 bytes from the RX FIFO also flushes RX FIFO
    print("\nComplete RX FIFO:", radio.read(12))


def slave(timeout=6):  # will listen for 6 seconds before timing out
    """Only listen for 3 payload from the master node"""
    # the "data ready" event will trigger in RX mode
    # the "data sent" or "data fail" events will trigger when we
    # receive with ACK payloads enabled (& loaded in TX FIFO)
    print("\nDisabling IRQ pin for all events.")
    radio.set_status_flags(StatusFlags())
    # setup radio to receive pings, fill TX FIFO with ACK payloads
    radio.write_ack_payload(1, ack_payloads[0])
    radio.write_ack_payload(1, ack_payloads[1])
    radio.write_ack_payload(1, ack_payloads[2])
    radio.listen = True  # start listening & clear irq_dr flag
    end_timer = time.monotonic() + timeout  # set end time
    while (
        radio.get_fifo_state(False) != FifoState.Full and time.monotonic() < end_timer
    ):
        # if RX FIFO is not full and timeout is not reached, then keep waiting
        pass
    time.sleep(0.5)  # wait for last ACK payload to transmit
    radio.listen = False  # put radio in TX mode & discard any ACK payloads
    if radio.available():  # if RX FIFO is not empty (timeout did not occur)
        # all 3 payloads received were 5 bytes each, and RX FIFO is full
        # so, fetching 15 bytes from the RX FIFO also flushes RX FIFO
        print("Complete RX FIFO:", radio.read(15))


def set_role() -> bool:
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
        slave(*[int(x) for x in user_input[1:2]])
        return True
    if user_input[0].upper().startswith("T"):
        master()
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
    print(
        f"Make sure the IRQ pin is connected to the GPIO{IRQ_PIN}",
        "Run slave() on receiver",
        "Run master() on transmitter",
        sep="\n",
    )

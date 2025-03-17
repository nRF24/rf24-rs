"""
This example uses the nRF24L01 as a 'fake' BLE Beacon
"""

import time
from rf24_py import (
    RF24,
    FakeBle,
    UrlService,
    BatteryService,
    TemperatureService,
    PaLevel,
)


def _prompt(remaining):
    if remaining and (remaining % 5 == 0 or remaining < 5):
        print(remaining, "advertisements left to go!")


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

        # instantiate the helper class around the radio instance
        self.ble = FakeBle(self.radio)
        # configure the radio for BLE compatibility
        self.ble.begin()

        # set the Power Amplifier level to -12 dBm since this test example is
        # usually run with nRF24L01 transceivers in close proximity of each other
        self.radio.pa_level = PaLevel.Low  # PaLevel.Max is default

        # for debugging
        # self.radio.print_details()

    def tx_battery(self, count: int = 50):
        """Transmits a battery charge level as a BLE beacon"""
        self.radio.as_tx()  # ensures the nRF24L01 is in TX mode

        battery_service = BatteryService()
        battery_service.data = 85  # 85 % remaining charge level
        buffer = battery_service.buffer

        self.ble.name = "nRF24L01"
        self.ble.show_pa_level = True

        print(
            "Number of bytes remaining in advertisement payload:",
            self.ble.len_available(buffer),
        )

        while count:
            self.ble.send(buffer)
            # self.ble.hop_channel()
            time.sleep(0.5)
            count -= 1
            _prompt(count)

        # disable these features when done (for example purposes)
        self.ble.name = None
        self.ble.show_pa_level = False

    def tx_temperature(self, count: int = 50):
        """Transmits a battery charge level as a BLE beacon"""
        self.radio.as_tx()  # ensures the nRF24L01 is in TX mode

        temperature_service = TemperatureService()
        temperature_service.data = 45.5  # 45.5 degrees Celsius
        buffer = temperature_service.buffer

        self.ble.name = "nRF24L01"

        print(
            "Number of bytes remaining in advertisement payload:",
            self.ble.len_available(buffer),
        )

        while count:
            self.ble.send(buffer)
            # self.ble.hop_channel()
            time.sleep(0.5)
            count -= 1
            _prompt(count)

        # disable these features when done (for example purposes)
        self.ble.name = None
        self.ble.show_pa_level = False

    def tx_url(self, count: int = 50):
        """Transmits a URL as a BLE beacon"""
        self.radio.as_tx()  # ensures the nRF24L01 is in TX mode

        url_service = UrlService()
        url_service.data = "https:/www.google.com"
        buffer = url_service.buffer

        print(
            "Number of bytes remaining in advertisement payload:",
            self.ble.len_available(buffer),
        )

        while count:
            self.ble.send(buffer)
            # self.ble.hop_channel()
            time.sleep(0.5)
            count -= 1
            _prompt(count)

    def rx(self, timeout: int = 6):
        """Polls the radio and prints the received value. This method expires
        after 6 seconds of no received transmission."""
        self.radio.as_rx()  # put radio into RX mode and power up

        end_time = time.monotonic() + timeout
        while time.monotonic() < end_time:
            has_payload, pipe_number = self.radio.available_pipe()
            if has_payload:
                # fetch 1 payload from RX FIFO
                received = self.radio.read()
                print(f"Received {len(received)} bytes on pipe {pipe_number}")
                end_time = time.monotonic() + timeout  # reset the timeout timer

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
                "*** Enter 'T' to transmit a temperature measurement.\n"
                "*** Enter 'B' to transmit a battery charge level.\n"
                "*** Enter 'U' to transmit a URL.\n"
                "*** Enter 'Q' to quit example.\n"
            )
            or "?"
        )
        user_input = user_input.split()
        if user_input[0].upper().startswith("R"):
            self.rx(*[int(x) for x in user_input[1:2]])
            return True
        if user_input[0].upper().startswith("T"):
            self.tx_temperature(*[int(x) for x in user_input[1:2]])
            return True
        if user_input[0].upper().startswith("B"):
            self.tx_battery(*[int(x) for x in user_input[1:2]])
            return True
        if user_input[0].upper().startswith("U"):
            self.tx_url(*[int(x) for x in user_input[1:2]])
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

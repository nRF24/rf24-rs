"""
A scanner example written in python using the std lib's ncurses wrapper.

This is a good diagnostic tool to check whether you're picking a
good channel for your application.

See documentation at https://nRF24.github.io/rf24-rs
"""

import curses
import time
from typing import List, Tuple
from rf24_py import RF24, DataRate, CrcLength

print(__file__)  # print example name

# The radio's CE Pin uses a GPIO number.
# On Linux, consider the device path `/dev/gpiochip<N>`:
#   - `<N>` is the gpio chip's identifying number.
#     Using RPi4 (or earlier), this number is `0` (the default).
#     Using the RPi5, this number is actually `4`.
# The radio's CE pin must connected to a pin exposed on the specified chip.
CE_PIN = 22  # for GPIO22

# The radio's CSN Pin corresponds the SPI bus's CS pin (aka CE pin).
# On Linux, consider the device path `/dev/spidev<a>.<b>`:
#   - `<a>` is the SPI bus number (defaults to `0`)
#   - `<b>` is the CSN pin (must be unique for each device on the same SPI bus)
CSN_PIN = 0  # aka CE0 for SPI bus 0 (/dev/spidev0.0)

# create a radio object for the specified hardware config:
radio = RF24(CE_PIN, CSN_PIN)


OFFERED_DATA_RATES = ["1 Mbps", "2 Mbps", "250 kbps"]
AVAILABLE_RATES = [DataRate.Mbps1, DataRate.Mbps2, DataRate.Kbps250]
TOTAL_CHANNELS = 126
CACHE_MAX = 5  # the depth of history to calculate peaks

# To detect noise, we'll use the worst addresses possible (a reverse engineering
# tactic). These addresses are designed to confuse the radio into thinking that the
# RF signal's preamble is part of the packet/payload.
noise_address = [
    b"\x55\x55",
    b"\xaa\xaa",
    b"\x0a\xaa",
    b"\xa0\xaa",
    b"\x00\xaa",
    b"\xab\xaa",
]


class ChannelHistory:
    def __init__(self) -> None:
        #: FIFO for tracking peak decays
        self._history: List[bool] = [False] * CACHE_MAX
        #: for the total signal counts
        self.total: int = 0

    def push(self, value: bool) -> int:
        """Push a scan result's value into history while returning the sum of cached
        signals found. This function also increments the total signal count accordingly.
        """
        self._history = self._history[1:] + [value]
        self.total += value
        return self._history.count(True)


#: An array of histories for each channel
stored = [ChannelHistory() for _ in range(TOTAL_CHANNELS)]


class ProgressBar:
    """This represents a progress bar using a curses window object."""

    def __init__(
        self,
        x: int,
        y: int,
        cols: int,
        std_scr: curses.window,
        label: str,
        color: int,
    ):
        self.x, self.y, self.width, self.win, self.color = (x, y, cols, std_scr, color)
        self.win.move(self.y, self.x)
        self.win.attron(curses.color_pair(self.color))
        self.win.addstr(label)  # always labeled in MHz (4 digits)
        for _ in range(self.width - 8):  # draw the empty bar
            self.win.addch(curses.ACS_HLINE)
        self.win.addstr(" - ")  # draw the initial signal count
        self.win.attroff(curses.color_pair(self.color))

    def update(self, completed: int, signal_count: int):
        """Update the progress bar."""
        count = " - "
        if signal_count:
            count = " %X " % min(0xF, signal_count)
        filled = (self.width - 8) * completed / CACHE_MAX
        offset_x = 5
        self.win.move(self.y, self.x + offset_x)
        for i in range(offset_x, self.width - 3):
            bar_filled = i < (filled + offset_x)
            bar_color = 5 if bar_filled else self.color
            self.win.attron(curses.color_pair(bar_color))
            self.win.addch("=" if bar_filled else curses.ACS_HLINE)
            self.win.attroff(curses.color_pair(bar_color))
        self.win.attron(curses.color_pair(self.color))
        self.win.addstr(count)
        self.win.attroff(curses.color_pair(self.color))


def init_display(window) -> List[ProgressBar]:
    """Creates a table of progress bars (1 for each channel)."""
    progress_bars: List[ProgressBar] = [
        ProgressBar(0, 0, 0, window, "", 0)
    ] * TOTAL_CHANNELS
    bar_w = int(curses.COLS / 6)
    for i in range(21):  # 21 rows
        for j in range(i, i + (21 * 6), 21):  # 6 columns
            color = 7 if int(j / 21) % 2 else 3
            progress_bars[j] = ProgressBar(
                x=bar_w * int(j / 21),
                y=i + 3,
                cols=bar_w,
                std_scr=window,
                label=f"{2400 + (j)} ",
                color=color,
            )
    return progress_bars


def init_radio():
    """init the radio"""
    radio.begin()
    radio.set_auto_ack(False)
    radio.crc_length = CrcLength.Disabled
    radio.address_length = 2
    for pipe, address in enumerate(noise_address):
        radio.open_rx_pipe(pipe, address)
    radio.as_rx()
    radio.as_tx()
    radio.flush_rx()


def init_curses():
    """init the curses interface"""
    std_scr = curses.initscr()
    curses.noecho()
    curses.cbreak()
    curses.start_color()
    curses.use_default_colors()
    curses.init_pair(3, curses.COLOR_YELLOW, -1)
    curses.init_pair(5, curses.COLOR_MAGENTA, -1)
    curses.init_pair(7, curses.COLOR_WHITE, -1)
    return std_scr


def de_init_curses(spectrum_passes: int):
    """de-init the curses interface"""
    curses.nocbreak()
    curses.echo()
    curses.endwin()
    noisy_channels: int = 0
    digit_w = len(str(spectrum_passes))
    for channel, data in enumerate(stored):
        if data.total:
            count_padding = " " * (digit_w - len(str(data.total)))
            percentage = round(data.total / spectrum_passes * 100, 3)
            print(
                f"  {channel:>3}: {count_padding}{data.total} / {spectrum_passes} ({percentage} %)"
            )
            noisy_channels += 1
    print(
        f"{noisy_channels} channels detected signals out of {spectrum_passes}",
        "passes on the entire spectrum.",
    )


def get_user_input() -> Tuple[int, int]:
    """Get input parameters for the scan from the user."""
    for i, d_rate in enumerate(OFFERED_DATA_RATES):
        print(f"{i + 1}. {d_rate}")
    d_rate = input("Select your data rate [1, 2, 3] (defaults to 1 Mbps) ")
    duration = input("How long (in seconds) to perform scan? ")
    while not duration.isdigit():
        print("Please enter a positive number.")
        duration = input("How long (in seconds) to perform scan? ")
    return (
        max(1, min(3, 1 if not d_rate.isdigit() else int(d_rate))) - 1,
        abs(int(duration)),
    )


def scan_channel(channel: int) -> bool:
    """Scan a specified channel and report if a signal was detected."""
    radio.channel = channel
    radio.as_rx()
    time.sleep(0.00013)
    found_signal = radio.rpd
    radio.as_tx()
    if found_signal or radio.rpd or radio.available():
        radio.flush_rx()
        return True
    return False


def main():
    spectrum_passes = 0
    data_rate, duration = get_user_input()
    print(f"Scanning for {duration} seconds at {OFFERED_DATA_RATES[data_rate]}")
    init_radio()
    radio.data_rate = AVAILABLE_RATES[data_rate]
    try:
        std_scr = init_curses()
        timer_prompt = "Scanning for {:>3} seconds at " + OFFERED_DATA_RATES[data_rate]
        std_scr.addstr(0, 0, "Channels are labeled in Hz.")
        std_scr.addstr(1, 0, "Signal counts are clamped to a single hexadecimal digit.")
        bars = init_display(std_scr)
        channel, val = (0, False)
        end_time = time.monotonic() + duration
        while time.monotonic() < end_time:
            std_scr.addstr(2, 0, timer_prompt.format(int(end_time - time.monotonic())))
            val = scan_channel(channel)
            cache_sum = stored[channel].push(val)
            if stored[channel].total:
                bars[channel].update(cache_sum, stored[channel].total)
                std_scr.refresh()
            if channel + 1 == TOTAL_CHANNELS:
                channel = 0
                spectrum_passes += 1
            else:
                channel += 1
    finally:
        radio.power = False
        de_init_curses(spectrum_passes)


if __name__ == "__main__":
    main()
else:
    print("Enter 'main()' to run the program.")

from enum import Enum

class PaLevel(Enum):
    Min: "PaLevel" = ...
    Low: "PaLevel" = ...
    High: "PaLevel" = ...
    Max: "PaLevel" = ...

class CrcLength(Enum):
    Disabled: "CrcLength" = ...
    Bit8: "CrcLength" = ...
    Bit16: "CrcLength" = ...

class FifoState(Enum):
    Full: "FifoState" = ...
    Empty: "FifoState" = ...
    Occupied: "FifoState" = ...

class DataRate(Enum):
    Mbps1: "DataRate" = ...
    Mbps2: "DataRate" = ...
    Kbps250: "DataRate" = ...

class StatusFlags:
    def __init__(
        self,
        rx_dr: bool | int = False,
        tx_ds: bool | int = False,
        tx_df: bool | int = False,
    ): ...
    @property
    def rx_dr(self) -> bool: ...
    @property
    def tx_ds(self) -> bool: ...
    @property
    def tx_df(self) -> bool: ...

class RF24:
    def __init__(
        self,
        ce_pin: int,
        cs_pin: int,
        dev_gpio_chip: int = 0,
        dev_spi_bus: int = 0,
        spi_speed: int = 10000000,
    ) -> None: ...
    def begin(self) -> None: ...
    @property
    def is_rx(self) -> bool: ...
    def as_rx(self) -> None: ...
    def as_tx(self) -> None: ...
    def send(self, buf: bytes | bytearray, ask_no_ack: bool | int = False) -> bool: ...
    def write(
        self,
        buf: bytes | bytearray,
        ask_no_ack: bool | int = False,
        start_tx: bool | int = True,
    ) -> bool: ...
    def read(self, len: int | None = None) -> bytes: ...
    def resend(self) -> bool: ...
    def rewrite(self) -> None: ...
    def get_last_arc(self) -> int: ...
    @property
    def is_plus_variant(self) -> bool: ...
    @property
    def rpd(self) -> bool: ...
    def start_carrier_wave(self, level: PaLevel, channel: int) -> None: ...
    def stop_carrier_wave(self) -> None: ...
    def set_lna(self, enable: bool | int) -> None: ...
    def allow_ack_payloads(self, enable: bool | int) -> None: ...
    def set_auto_ack(self, enable: bool | int) -> None: ...
    def set_auto_ack_pipe(self, enable: bool | int, pipe: int) -> None: ...
    def allow_ask_no_ack(self, enable: bool | int) -> None: ...
    def write_ack_payload(self, pipe: int, buf: bytes | bytearray) -> bool: ...
    def set_auto_retries(self, count: int, delay: int) -> None: ...
    @property
    def channel(self) -> int: ...
    @channel.setter
    def channel(self, channel: int) -> None: ...
    @property
    def crc_length(self) -> CrcLength: ...
    @crc_length.setter
    def crc_length(self, crc_length: CrcLength) -> None: ...
    @property
    def data_rate(self) -> DataRate: ...
    @data_rate.setter
    def data_rate(self, data_rate: DataRate) -> None: ...
    def available(self) -> bool: ...
    def available_pipe(self) -> tuple[bool, int]: ...
    def flush_rx(self) -> None: ...
    def flush_tx(self) -> None: ...
    def get_fifo_state(self, about_tx: bool | int) -> FifoState: ...
    @property
    def pa_level(self) -> PaLevel: ...
    @pa_level.setter
    def pa_level(self, pa_level: PaLevel) -> None: ...
    @property
    def payload_length(self) -> int: ...
    @payload_length.setter
    def payload_length(self, length: int) -> None: ...
    def set_dynamic_payloads(self, enable: bool | int) -> None: ...
    def get_dynamic_payload_length(self) -> int: ...
    def open_rx_pipe(self, pipe: int, address: bytes | bytearray) -> None: ...
    def open_tx_pipe(self, address: bytes | bytearray) -> None: ...
    def close_rx_pipe(self, pipe: int) -> None: ...
    @property
    def address_length(self) -> int: ...
    @address_length.setter
    def address_length(self, length: int) -> None: ...
    @property
    def power(self) -> bool: ...
    @power.setter
    def power(self, enable: bool | int) -> None: ...
    def power_down(self) -> None: ...
    def power_up(self, delay: int | None = None) -> None: ...
    def set_status_flags(self, flags: StatusFlags | None = None) -> None: ...
    def clear_status_flags(self, flags: StatusFlags | None = None) -> None: ...
    def update(self) -> None: ...
    def get_status_flags(self) -> StatusFlags: ...

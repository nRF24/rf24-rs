from enum import Enum

class PaLevel(Enum):
    Min: "PaLevel" = ...  # type: ignore[misc]
    Low: "PaLevel" = ...  # type: ignore[misc]
    High: "PaLevel" = ...  # type: ignore[misc]
    Max: "PaLevel" = ...  # type: ignore[misc]

class CrcLength(Enum):
    Disabled: "CrcLength" = ...  # type: ignore[misc]
    Bit8: "CrcLength" = ...  # type: ignore[misc]
    Bit16: "CrcLength" = ...  # type: ignore[misc]

class FifoState(Enum):
    Full: "FifoState" = ...  # type: ignore[misc]
    Empty: "FifoState" = ...  # type: ignore[misc]
    Occupied: "FifoState" = ...  # type: ignore[misc]

class DataRate(Enum):
    Mbps1: "DataRate" = ...  # type: ignore[misc]
    Mbps2: "DataRate" = ...  # type: ignore[misc]
    Kbps250: "DataRate" = ...  # type: ignore[misc]

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
    def with_config(self, config: RadioConfig): ...
    @property
    def is_rx(self) -> bool: ...
    def as_rx(self) -> None: ...
    def as_tx(self, tx_address: bytes | bytearray | None = None) -> None: ...
    def ce_pin(self, value: bool | int) -> None: ...
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
    @property
    def ack_payloads(self) -> bool: ...
    @ack_payloads.setter
    def ack_payloads(self, enable: bool | int) -> None: ...
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
    @property
    def dynamic_payloads(self) -> bool: ...
    @dynamic_payloads.setter
    def dynamic_payloads(self, enable: bool | int) -> None: ...
    def get_dynamic_payload_length(self) -> int: ...
    def open_rx_pipe(self, pipe: int, address: bytes | bytearray) -> None: ...
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
    @property
    def tx_delay(self) -> int: ...
    @tx_delay.setter
    def tx_delay(self, value: int): ...
    def set_status_flags(self, flags: StatusFlags | None = None) -> None: ...
    def clear_status_flags(self, flags: StatusFlags | None = None) -> None: ...
    def update(self) -> None: ...
    def get_status_flags(self) -> StatusFlags: ...
    def print_details(self) -> None: ...

class RadioConfig:
    def __init__(self): ...
    @property
    def address_length(self) -> int: ...
    @address_length.setter
    def address_length(self, value: int): ...
    @property
    def payload_length(self) -> int: ...
    @payload_length.setter
    def payload_length(self, value: int): ...
    @property
    def channel(self) -> int: ...
    @channel.setter
    def channel(self, value: int): ...
    @property
    def pa_level(self) -> PaLevel: ...
    @pa_level.setter
    def pa_level(self, value: PaLevel): ...
    @property
    def lna_enable(self) -> bool: ...
    @lna_enable.setter
    def lna_enable(self, value: bool | int): ...
    @property
    def data_rate(self) -> DataRate: ...
    @data_rate.setter
    def data_rate(self, value: DataRate): ...
    @property
    def crc_length(self) -> CrcLength: ...
    @crc_length.setter
    def crc_length(self, value: CrcLength): ...
    @property
    def auto_ack(self) -> int: ...
    @auto_ack.setter
    def auto_ack(self, value: int): ...
    @property
    def dynamic_payloads(self) -> bool: ...
    @dynamic_payloads.setter
    def dynamic_payloads(self, value: bool | int): ...
    @property
    def ack_payloads(self) -> bool: ...
    @ack_payloads.setter
    def ack_payloads(self, value: bool | int): ...
    @property
    def ask_no_ack(self) -> bool: ...
    @ask_no_ack.setter
    def ask_no_ack(self, value: bool | int): ...
    @property
    def auto_retry_delay(self) -> int: ...
    @property
    def auto_retry_count(self) -> int: ...
    def set_auto_retries(self, delay: int, count: int): ...
    @property
    def rx_dr(self) -> bool: ...
    @rx_dr.setter
    def rx_dr(self, value: bool | int): ...
    @property
    def tx_ds(self) -> bool: ...
    @tx_ds.setter
    def tx_ds(self, value: bool | int): ...
    @property
    def tx_df(self) -> bool: ...
    @tx_df.setter
    def tx_df(self, value: bool | int): ...
    def set_rx_address(self, pipe: int, address: bytes | bytearray): ...
    def get_rx_address(self, pipe: int) -> bytes: ...
    def close_rx_pipe(self, pipe: int) -> None: ...
    @property
    def tx_address(self) -> bytes: ...
    @tx_address.setter
    def tx_address(self, value: bytes | bytearray): ...

class BatteryService:
    def __init__(self) -> None: ...
    @property
    def data(self) -> int: ...
    @data.setter
    def data(self, value: int) -> None: ...
    @property
    def buffer(self) -> bytes: ...

class TemperatureService:
    def __init__(self) -> None: ...
    @property
    def data(self) -> float: ...
    @data.setter
    def data(self, value: float | int) -> None: ...
    @property
    def buffer(self) -> bytes: ...

class UrlService:
    def __init__(self) -> None: ...
    @property
    def data(self) -> str: ...
    @data.setter
    def data(self, value: str) -> None: ...
    @property
    def pa_level(self) -> int: ...
    @pa_level.setter
    def pa_level(self, value: int) -> None: ...
    @property
    def buffer(self) -> bytes: ...

class BlePayload:
    @property
    def mac_address(self) -> bytes: ...
    @property
    def short_name(self) -> str | None: ...
    @property
    def tx_power(self) -> int | None: ...
    @property
    def battery_charge(self) -> BatteryService | None: ...
    @property
    def url(self) -> UrlService | None: ...
    @property
    def temperature(self) -> TemperatureService | None: ...

def ble_config() -> RadioConfig: ...

class FakeBle:
    def __init__(self, radio: RF24): ...
    def hop_channel(self) -> None: ...
    def len_available(self, buf: bytes | bytearray) -> int: ...
    def send(self, buf: bytes | bytearray) -> bool: ...
    def read(self) -> BlePayload | None: ...
    @property
    def show_pa_level(self) -> bool: ...
    @show_pa_level.setter
    def show_pa_level(self, enable: int | bool) -> None: ...
    @property
    def name(self) -> str | None: ...
    @name.setter
    def name(self, value: str | None) -> None: ...
    @property
    def mac_address(self) -> bytes: ...
    @mac_address.setter
    def mac_address(self, value: bytes | bytearray) -> None: ...

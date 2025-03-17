# `rf24ble-rs`

This crate uses the rf24-rs crate to make the nRF24L01 imitate a
Bluetooth-Low-Emissions (BLE) beacon. A BLE beacon can send data (referred to as
advertisements) to any BLE compatible device (ie smart devices with Bluetooth
4.0 or later) that is listening.

[fake-ble-research]: http://dmitry.gr/index.php?r=05.Projects&proj=11.%20Bluetooth%20LE%20fakery

Original research was done by [Dmitry Grinberg and his write-up (including C
source code) can be found here][fake-ble-research].
As this technique can prove invaluable in certain project designs, the code
here has been adapted to work with Python.

## Limitations

Because the nRF24L01 wasn't designed for BLE advertising, it has some limitations that helps to be aware of.

1. The maximum payload length is shortened to **18** bytes (when not broadcasting a device
   name nor the radio's PA level). This is calculated as:

   ```text
   32 (nRF24L01 maximum) - 6 (MAC address) - 5 (required flags) - 3 (CRC checksum) = 18
   ```

   Use the helper function [`FakeBle::len_available()`](fn@crate::radio::FakeBle::len_available())
   to determine if your payload can be transmit.
2. The channels that BLE use are limited to the following three:

   - 2.402 GHz
   - 2.426 GHz
   - 2.480 GHz.

   For convenience, use [`FakeBle::hop_channel()`](fn@crate::radio::FakeBle::hop_channel()) (when radio is in TX mode only) to to switch between these frequencies.
3. CRC length is disabled in the nRF24L01 firmware because BLE specifications require 3 bytes,
   and the nRF24L01 firmware can only handle a maximum of 2.
   Thus, we append the required 3 bytes of the calculated CRC24 into the payload.
4. Address length of BLE packet only uses 4 bytes, so we have set that accordingly.
5. The auto-ack (automatic acknowledgment) feature of the nRF24L01 is useless
   when transmitting to BLE devices, thus it is disabled as well as automatic
   re-transmit and custom ACK payloads features which both depend on the
   automatic acknowledgments feature.
6. Dynamic payloads feature of the nRF24L01 isn't compatible with BLE specifications.
   Thus, we have disabled it.
7. BLE specifications only allow using 1 Mbps RF data rate, so that too has been hard coded.
8. Only the "on data sent" & "on data ready" events will have
   an effect on the interrupt (IRQ) pin. The "on data fail" is never
   triggered because auto-ack feature is disabled.

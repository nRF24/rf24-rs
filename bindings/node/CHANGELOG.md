# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
<!-- markdownlint-disable MD024 -->

## [rf24-node/0.3.1] - 2025-07-22

### <!-- 6 --> 📦 Dependency updates

- Bump `rf24ble-rs` to v0.1.3 in [`d408ffe`](https://github.com/nRF24/rf24-rs/commit/d408ffeee12c94b6580e7114bc9d6ab3a7eeeb23)
- Bump brace-expansion from 1.1.11 to 1.1.12 in [`3635077`](https://github.com/nRF24/rf24-rs/commit/36350775e7dbb2ffed69c103406da7d582e28369)
- Bump the npm group across 1 directory with 7 updates in [`e0192dd`](https://github.com/nRF24/rf24-rs/commit/e0192ddb9dfe97d6bfb08a3e9f86ea66511622f5)
- Migrate to napi-rs v3 in [`886ce08`](https://github.com/nRF24/rf24-rs/commit/886ce0896504c8309e2259780deb145fb8049a62)
- Bump `rf24-rs` to v0.3.1 in [`70bdf19`](https://github.com/nRF24/rf24-rs/commit/70bdf197249712bab51fb59b34397598ba2fa86f)
- Bump `rf24ble-rs` to v0.1.4 in [`0ff3ffa`](https://github.com/nRF24/rf24-rs/commit/0ff3ffa3b96c19cf63537603260a468cc1d8d286)

### <!-- 8 --> 📝 Documentation

- Revise docs about `RF24::tx_delay` property. in [`e7b29c1`](https://github.com/nRF24/rf24-rs/commit/e7b29c14ac33cdebd4437ef64f89d2a340f89613)
- Switch to `nur` in [`163b9aa`](https://github.com/nRF24/rf24-rs/commit/163b9aa8576e4716dbff953d6c6f21125e4e4347)
- Use typedoc default HTML output in [`dc7695f`](https://github.com/nRF24/rf24-rs/commit/dc7695f97c8fc5f084eb821f1f587113835bb7ef)

### <!-- 9 --> 🗨️ Changed

- Correct typo in `napi prepublish` command in [`e9e26a4`](https://github.com/nRF24/rf24-rs/commit/e9e26a4dc60a8e114eca897d81b7cf091f9b822f)

[rf24-node/0.3.1]: https://github.com/nRF24/rf24-rs/compare/rf24-node/0.3.0...rf24-node/0.3.1

Full commit diff: [`rf24-node/0.3.0...rf24-node/0.3.1`][rf24-node/0.3.1]

## [rf24-node/0.3.0] - 2025-05-04

### <!-- 6 --> 📦 Dependency updates

- Bump the npm group with 2 updates in [`3085709`](https://github.com/nRF24/rf24-rs/commit/30857095d4b6159d041941a7b04e002a6429ad07)
- Bump the npm group across 1 directory with 6 updates in [`e3479b6`](https://github.com/nRF24/rf24-rs/commit/e3479b69c31a3b10bc606d9a9e5264094727fd23)
- Bump `rf24-rs` to v0.3.0 in [`abb8fda`](https://github.com/nRF24/rf24-rs/commit/abb8fdab9575ef30fa3445067aca11f21f07dfbb)
- Bump `rf24-node` to v0.3.0 in [`2bff30c`](https://github.com/nRF24/rf24-rs/commit/2bff30cc7500ca556acaa84ed9a239bdcb4b8061)

### <!-- 7 -->🚦 Tests

- Use concrete error type in [`72fe862`](https://github.com/nRF24/rf24-rs/commit/72fe8621778f2e3e4f437f2349e3a5c4e1e88e32)
- Replace `open_tx_pipe()` with `as_tx(Option<address>)` in [`9baaf57`](https://github.com/nRF24/rf24-rs/commit/9baaf572c5ac3e6e66349227bf8fdde8235280c1)

### <!-- 9 --> 🗨️ Changed

- Remove `_` prefix from private members in [`39d8287`](https://github.com/nRF24/rf24-rs/commit/39d8287461777bbf9d8a1c1a92636b46b29669d0)

[rf24-node/0.3.0]: https://github.com/nRF24/rf24-rs/compare/rf24-node/0.2.2...rf24-node/0.3.0

Full commit diff: [`rf24-node/0.2.2...rf24-node/0.3.0`][rf24-node/0.3.0]

## [rf24-node/0.2.2] - 2025-04-08

### <!-- 6 --> 📦 Dependency updates

- Upgrade to yarn modern in [`5e32efa`](https://github.com/nRF24/rf24-rs/commit/5e32efa9a3a8f18594261b28771a25e2ef41a70a)
- Bump the npm group across 1 directory with 2 updates in [`57ff1a5`](https://github.com/nRF24/rf24-rs/commit/57ff1a5c870f6671a52290c77005828ea3e504d8)
- Update defmt requirement from 0.3.10 to 1.0.1 in the cargo group in [`d00f119`](https://github.com/nRF24/rf24-rs/commit/d00f119c94c30e1ea41d3f3bc05921162c6024fe)
- Bump `rf24ble-rs` to v0.1.2 in [`33fe013`](https://github.com/nRF24/rf24-rs/commit/33fe0130101feb42aaa49aa5b88ac928034ec261)
- Bump `rf24-node` to v0.2.2 in [`9673628`](https://github.com/nRF24/rf24-rs/commit/96736283d11aced8c58cb1e871c71829d57a50bf)

### <!-- 8 --> 📝 Documentation

- Update `rf24-node` README in [`85ce48a`](https://github.com/nRF24/rf24-rs/commit/85ce48a87aadb16189bd08285e96b767c410a8f1)

### <!-- 9 --> 🗨️ Changed

- Regenerate change logs in [`581751a`](https://github.com/nRF24/rf24-rs/commit/581751af27d074797b4749572f05e9f8b3548e21)

[rf24-node/0.2.2]: https://github.com/nRF24/rf24-rs/compare/rf24-node/0.2.1...rf24-node/0.2.2

Full commit diff: [`rf24-node/0.2.1...rf24-node/0.2.2`][rf24-node/0.2.2]

## [rf24-node/0.2.1] - 2025-04-06

### <!-- 4 --> 🛠️ Fixed

- Use const for max BLE payload size in [`b563d4a`](https://github.com/nRF24/rf24-rs/commit/b563d4ae75704796101158d15f638092dd66ba9e)

### <!-- 6 --> 📦 Dependency updates

- Bump `rf24-rs` to v0.2.1 in [`f266b96`](https://github.com/nRF24/rf24-rs/commit/f266b9695f1c492cce1ea7720a6df4fde298c338)
- Bump `rf24ble-rs` to v0.1.0 in [`1513ada`](https://github.com/nRF24/rf24-rs/commit/1513ada7aa678588ef153cbe1511021efeb7b286)
- Bump `rf24ble-rs` to v0.1.1 in [`3094968`](https://github.com/nRF24/rf24-rs/commit/3094968d17f63dea1594b0438534319f3aac5e89)
- Bump `rf24-node` to v0.2.1 in [`75c57d3`](https://github.com/nRF24/rf24-rs/commit/75c57d3b2f140ab15e95f6de76b771aaa24a8591)

[rf24-node/0.2.1]: https://github.com/nRF24/rf24-rs/compare/rf24-node/0.2.0...rf24-node/0.2.1

Full commit diff: [`rf24-node/0.2.0...rf24-node/0.2.1`][rf24-node/0.2.1]

## [rf24-node/0.2.0] - 2025-04-06

### <!-- 1 --> 🚀 Added

- Add fake BLE API for nRF24L01 in [`1d751d3`](https://github.com/nRF24/rf24-rs/commit/1d751d3402a2a216d685f15893b60b74f983bf0b)

### <!-- 10 --> 💥 Breaking Changes

- Rename EsbConfig to RadioConfig in [`df927b6`](https://github.com/nRF24/rf24-rs/commit/df927b6c637463c2e1b46c528095eddbaebdbfb6)

### <!-- 6 --> 📦 Dependency updates

- Bump `rf24-rs` to v0.1.1 in [`8ca278b`](https://github.com/nRF24/rf24-rs/commit/8ca278bbbff72514c8c84001bbd3480d4ba7d1d9)
- Bump `rf24-rs` to v0.1.2 in [`81dd350`](https://github.com/nRF24/rf24-rs/commit/81dd350634880a4a76f3817e0e85d8099490fb37)
- Bump `rf24-rs` to v0.2.0 in [`5ce9ac4`](https://github.com/nRF24/rf24-rs/commit/5ce9ac456ec1e1bb00613e433ec8636919c58495)
- Bump the npm group with 9 updates in [`bac4607`](https://github.com/nRF24/rf24-rs/commit/bac460799b49447674df3d43eeac5e10d83300c1)
- Bump `rf24-node` to v0.2.0 in [`4ccb8aa`](https://github.com/nRF24/rf24-rs/commit/4ccb8aa09a06b154efeb9007ba1b3a88ea7c5682)

### <!-- 7 -->🚦 Tests

- Improve ``rf24-rs`` tests in [`dae2ab2`](https://github.com/nRF24/rf24-rs/commit/dae2ab2576ee662f8e08274d2f8c98990730b03e)

### <!-- 8 --> 📝 Documentation

- Update API docs for bindings in [`f64b2e2`](https://github.com/nRF24/rf24-rs/commit/f64b2e2923a7f3479248abd25a6c2f4d4b141543)
- Various doc updates in [`b392e63`](https://github.com/nRF24/rf24-rs/commit/b392e63f1283bcf9e98ac5980e3a1ab3c7f90199)

### <!-- 9 --> 🗨️ Changed

- Reassess min supported rust version in [`a9ca278`](https://github.com/nRF24/rf24-rs/commit/a9ca278b3ed38a682bba54bbf32de2b874ae9097)
- Dev workflow improvements in [`c493356`](https://github.com/nRF24/rf24-rs/commit/c493356b8044655f5a1930e7ef240243fa990d34)
- Reorganize bindings' sources in [`3383200`](https://github.com/nRF24/rf24-rs/commit/33832000723857bf7b09a94c4ab892adc9cc66bf)
- Improve readability in `rf24-rs` sources in [`f468315`](https://github.com/nRF24/rf24-rs/commit/f4683153d72bd67b0a7707a3a922a0d03b852164)

[rf24-node/0.2.0]: https://github.com/nRF24/rf24-rs/compare/rf24-node/0.1.2...rf24-node/0.2.0

Full commit diff: [`rf24-node/0.1.2...rf24-node/0.2.0`][rf24-node/0.2.0]

## [rf24-node/0.1.2] - 2025-03-09

### <!-- 6 --> 📦 Dependency updates

- Bump `rf24-node` to v0.1.2 in [`48c6631`](https://github.com/nRF24/rf24-rs/commit/48c663146c989e05e2506eda67fcbe0795e68c98)

### <!-- 9 --> 🗨️ Changed

- Run `npm pkg fix` on node bindings in [`917b7c1`](https://github.com/nRF24/rf24-rs/commit/917b7c138d0f809c4959b0aee860d2cfb7f15fac)

[rf24-node/0.1.2]: https://github.com/nRF24/rf24-rs/compare/rf24-node/0.1.1...rf24-node/0.1.2

Full commit diff: [`rf24-node/0.1.1...rf24-node/0.1.2`][rf24-node/0.1.2]

## [rf24-node/0.1.1] - 2025-03-09

### <!-- 6 --> 📦 Dependency updates

- Bump `rf24-node` to v0.1.1 in [`217f51c`](https://github.com/nRF24/rf24-rs/commit/217f51cde5c33451b9fbf845ec0a2ca89902e990)

[rf24-node/0.1.1]: https://github.com/nRF24/rf24-rs/compare/rf24-node/0.1.0...rf24-node/0.1.1

Full commit diff: [`rf24-node/0.1.0...rf24-node/0.1.1`][rf24-node/0.1.1]

## [rf24-node/0.1.0] - 2025-03-09

### <!-- 1 --> 🚀 Added

- Implement nRF24L01 driver in [`98739c1`](https://github.com/nRF24/rf24-rs/commit/98739c1d3fdbacb4971400d8671f4b43ba435bf3)

### <!-- 6 --> 📦 Dependency updates

- Specify repo metadata in package.json in [`3e2ad68`](https://github.com/nRF24/rf24-rs/commit/3e2ad68e1de85e1622a3c474c4853c2620e5f103)

### <!-- 9 --> 🗨️ Changed

- Prepare release CI in [`622a70a`](https://github.com/nRF24/rf24-rs/commit/622a70a438512d186a1aa2724bbb5275e57579e5)

[rf24-node/0.1.0]: https://github.com/nRF24/rf24-rs/compare/f8863cc36d66708bfa0fb2fb1a219c7b2f97f7d6...rf24-node/0.1.0

Full commit diff: [`f8863cc...rf24-node/0.1.0`][rf24-node/0.1.0]

<!-- generated by git-cliff -->

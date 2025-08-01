# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
<!-- markdownlint-disable MD024 -->

## [rf24-py/v0.4.1] - 2025-08-01

### <!-- 4 --> 🛠️ Fixed

- Specify README path in metadata by @2bndy5 in [`09e0ce3`](https://github.com/nRF24/rf24-rs/commit/09e0ce32063aec171ac9304a96fa301de63c28d6)

### <!-- 6 --> 📦 Dependency updates

- Bump version to `rf24-py`/v0.4.1 by @2bndy5 in [`64c1af2`](https://github.com/nRF24/rf24-rs/commit/64c1af24c5413bf479766a3c019fe540a130af1e)

[rf24-py/v0.4.1]: https://github.com/nRF24/rf24-rs/compare/rf24-py/v0.4.0...rf24-py/v0.4.1

Full commit diff: [`rf24-py/v0.4.0...rf24-py/v0.4.1`][rf24-py/v0.4.1]

## [rf24-py/v0.4.0] - 2025-08-01

### <!-- 10 --> 💥 Breaking Changes

- Migrate to uv by @2bndy5 in [#71](https://github.com/nRF24/rf24-rs/pull/71)

### <!-- 6 --> 📦 Dependency updates

- Edit bump-n-release CI workflow by @2bndy5 in [#69](https://github.com/nRF24/rf24-rs/pull/69)
- Bump version to `rf24-py`/v0.4.0 by @2bndy5 in [`bf64d49`](https://github.com/nRF24/rf24-rs/commit/bf64d49d849710b8bba7575884047f249b50153d)

[rf24-py/v0.4.0]: https://github.com/nRF24/rf24-rs/compare/rf24-py/0.3.1...rf24-py/v0.4.0

Full commit diff: [`rf24-py/0.3.1...rf24-py/v0.4.0`][rf24-py/v0.4.0]

## [rf24-py/0.3.1] - 2025-07-22

### <!-- 6 --> 📦 Dependency updates

- Migrate to napi-rs v3 by @2bndy5 in [#64](https://github.com/nRF24/rf24-rs/pull/64)
- Bump `rf24-rs` to v0.3.1 by @2bndy5 in [`70bdf19`](https://github.com/nRF24/rf24-rs/commit/70bdf197249712bab51fb59b34397598ba2fa86f)
- Bump `rf24ble-rs` to v0.1.4 by @2bndy5 in [`0ff3ffa`](https://github.com/nRF24/rf24-rs/commit/0ff3ffa3b96c19cf63537603260a468cc1d8d286)
- Bump `rf24-py` to v0.3.1 by @2bndy5 in [`4b0e1f8`](https://github.com/nRF24/rf24-rs/commit/4b0e1f82d0bd5ffa844fa1ccddc9f6c6fb86333d)

### <!-- 8 --> 📝 Documentation

- Revise docs about `RF24::tx_delay` property. by @2bndy5 in [`e7b29c1`](https://github.com/nRF24/rf24-rs/commit/e7b29c14ac33cdebd4437ef64f89d2a340f89613)
- Switch to `nur` by @2bndy5 in [#50](https://github.com/nRF24/rf24-rs/pull/50)
- Use typedoc default HTML output by @2bndy5 in [#59](https://github.com/nRF24/rf24-rs/pull/59)

[rf24-py/0.3.1]: https://github.com/nRF24/rf24-rs/compare/rf24-py/0.3.0...rf24-py/0.3.1

Full commit diff: [`rf24-py/0.3.0...rf24-py/0.3.1`][rf24-py/0.3.1]

## [rf24-py/0.3.0] - 2025-05-04

### <!-- 1 --> 🚀 Added

- Use concrete error type by @2bndy5 in [#42](https://github.com/nRF24/rf24-rs/pull/42)

### <!-- 10 --> 💥 Breaking Changes

- Replace `open_tx_pipe()` with `as_tx(Option<address>)` by @2bndy5 in [#41](https://github.com/nRF24/rf24-rs/pull/41)

### <!-- 6 --> 📦 Dependency updates

- Upgrade to yarn modern by @2bndy5 in [#33](https://github.com/nRF24/rf24-rs/pull/33)
- Update defmt requirement from 0.3.10 to 1.0.1 in the cargo group by @dependabot[bot] in [#31](https://github.com/nRF24/rf24-rs/pull/31)
- Bump `rf24ble-rs` to v0.1.2 by @2bndy5 in [`33fe013`](https://github.com/nRF24/rf24-rs/commit/33fe0130101feb42aaa49aa5b88ac928034ec261)
- Bump `rf24-rs` to v0.3.0 by @2bndy5 in [`abb8fda`](https://github.com/nRF24/rf24-rs/commit/abb8fdab9575ef30fa3445067aca11f21f07dfbb)
- Bump `rf24ble-rs` to v0.1.3 by @2bndy5 in [`d408ffe`](https://github.com/nRF24/rf24-rs/commit/d408ffeee12c94b6580e7114bc9d6ab3a7eeeb23)
- Bump `rf24-py` to v0.3.0 by @2bndy5 in [`235dc93`](https://github.com/nRF24/rf24-rs/commit/235dc93bf1bca474b9921e5af1add13f98d6182e)

### <!-- 9 --> 🗨️ Changed

- Regenerate change logs by @2bndy5 in [`581751a`](https://github.com/nRF24/rf24-rs/commit/581751af27d074797b4749572f05e9f8b3548e21)
- Remove `_` prefix from private members by @2bndy5 in [`39d8287`](https://github.com/nRF24/rf24-rs/commit/39d8287461777bbf9d8a1c1a92636b46b29669d0)

[rf24-py/0.3.0]: https://github.com/nRF24/rf24-rs/compare/rf24-py/0.2.1...rf24-py/0.3.0

Full commit diff: [`rf24-py/0.2.1...rf24-py/0.3.0`][rf24-py/0.3.0]

## [rf24-py/0.2.1] - 2025-04-06

### <!-- 4 --> 🛠️ Fixed

- Use const for max BLE payload size by @2bndy5 in [#30](https://github.com/nRF24/rf24-rs/pull/30)

### <!-- 6 --> 📦 Dependency updates

- Bump `rf24-rs` to v0.2.1 by @2bndy5 in [`f266b96`](https://github.com/nRF24/rf24-rs/commit/f266b9695f1c492cce1ea7720a6df4fde298c338)
- Bump `rf24ble-rs` to v0.1.0 by @2bndy5 in [`1513ada`](https://github.com/nRF24/rf24-rs/commit/1513ada7aa678588ef153cbe1511021efeb7b286)
- Bump `rf24ble-rs` to v0.1.1 by @2bndy5 in [`3094968`](https://github.com/nRF24/rf24-rs/commit/3094968d17f63dea1594b0438534319f3aac5e89)
- Bump `rf24-py` to v0.2.1 by @2bndy5 in [`a030660`](https://github.com/nRF24/rf24-rs/commit/a030660d255715c5069e92af745b9199b6e466a1)

[rf24-py/0.2.1]: https://github.com/nRF24/rf24-rs/compare/rf24-py/0.2.0...rf24-py/0.2.1

Full commit diff: [`rf24-py/0.2.0...rf24-py/0.2.1`][rf24-py/0.2.1]

## [rf24-py/0.2.0] - 2025-04-06

### <!-- 1 --> 🚀 Added

- Add fake BLE API for nRF24L01 by @2bndy5 in [#25](https://github.com/nRF24/rf24-rs/pull/25)

### <!-- 10 --> 💥 Breaking Changes

- Rename EsbConfig to RadioConfig by @2bndy5 in [#19](https://github.com/nRF24/rf24-rs/pull/19)

### <!-- 6 --> 📦 Dependency updates

- Bump `rf24-rs` to v0.1.1 by @2bndy5 in [`8ca278b`](https://github.com/nRF24/rf24-rs/commit/8ca278bbbff72514c8c84001bbd3480d4ba7d1d9)
- Bump `rf24-rs` to v0.1.2 by @2bndy5 in [`81dd350`](https://github.com/nRF24/rf24-rs/commit/81dd350634880a4a76f3817e0e85d8099490fb37)
- Update pyo3 requirement from 0.23.4 to 0.24.0 in the cargo group by @dependabot[bot] in [#17](https://github.com/nRF24/rf24-rs/pull/17)
- Bump `rf24-rs` to v0.2.0 by @2bndy5 in [`5ce9ac4`](https://github.com/nRF24/rf24-rs/commit/5ce9ac456ec1e1bb00613e433ec8636919c58495)
- Bump `rf24-py` to v0.2.0 by @2bndy5 in [`007f745`](https://github.com/nRF24/rf24-rs/commit/007f745b384d711ef03e7f7122d084743bd66442)

### <!-- 7 -->🚦 Tests

- Improve ``rf24-rs`` tests by @2bndy5 in [#26](https://github.com/nRF24/rf24-rs/pull/26)

### <!-- 8 --> 📝 Documentation

- Update API docs for bindings by @2bndy5 in [#20](https://github.com/nRF24/rf24-rs/pull/20)
- Various doc updates by @2bndy5 in [#22](https://github.com/nRF24/rf24-rs/pull/22)
- Some review changes by @2bndy5 in [`86b4117`](https://github.com/nRF24/rf24-rs/commit/86b4117722fccb55e7b09187b61969401ffaee1e)

### <!-- 9 --> 🗨️ Changed

- Reassess min supported rust version by @2bndy5 in [`a9ca278`](https://github.com/nRF24/rf24-rs/commit/a9ca278b3ed38a682bba54bbf32de2b874ae9097)
- Dev workflow improvements by @2bndy5 in [`c493356`](https://github.com/nRF24/rf24-rs/commit/c493356b8044655f5a1930e7ef240243fa990d34)
- Reorganize bindings' sources by @2bndy5 in [`3383200`](https://github.com/nRF24/rf24-rs/commit/33832000723857bf7b09a94c4ab892adc9cc66bf)
- Improve readability in `rf24-rs` sources by @2bndy5 in [`f468315`](https://github.com/nRF24/rf24-rs/commit/f4683153d72bd67b0a7707a3a922a0d03b852164)

[rf24-py/0.2.0]: https://github.com/nRF24/rf24-rs/compare/rf24-py/0.1.1...rf24-py/0.2.0

Full commit diff: [`rf24-py/0.1.1...rf24-py/0.2.0`][rf24-py/0.2.0]

## [rf24-py/0.1.1] - 2025-03-09

### <!-- 1 --> 🚀 Added

- Implement nRF24L01 driver by @2bndy5 in [#1](https://github.com/nRF24/rf24-rs/pull/1)

### <!-- 6 --> 📦 Dependency updates

- Bump the pip group with 2 updates by @dependabot[bot] in [#15](https://github.com/nRF24/rf24-rs/pull/15)
- Bump `rf24-py` to v0.1.1 by @2bndy5 in [`586b255`](https://github.com/nRF24/rf24-rs/commit/586b255c8ca1266bbef382b4eb3677ec87a6e79f)

### <!-- 9 --> 🗨️ Changed

- Prepare release CI by @2bndy5 in [#14](https://github.com/nRF24/rf24-rs/pull/14)

[rf24-py/0.1.1]: https://github.com/nRF24/rf24-rs/compare/f8863cc36d66708bfa0fb2fb1a219c7b2f97f7d6...rf24-py/0.1.1

Full commit diff: [`f8863cc...rf24-py/0.1.1`][rf24-py/0.1.1]

## New Contributors

- @dependabot[bot] made their first contribution in [#15](https://github.com/nRF24/rf24-rs/pull/15)

<!-- generated by git-cliff -->

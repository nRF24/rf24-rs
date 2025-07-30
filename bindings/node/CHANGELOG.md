# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
<!-- markdownlint-disable MD024 -->

## [rf24-node/0.3.1] - 2025-07-22

### <!-- 6 --> 📦 Dependency updates

- Bump `rf24ble-rs` to v0.1.3 by @2bndy5 in [`d408ffe`](https://github.com/nRF24/rf24-rs/commit/d408ffeee12c94b6580e7114bc9d6ab3a7eeeb23)
- Bump brace-expansion from 1.1.11 to 1.1.12 by @dependabot[bot] in [#54](https://github.com/nRF24/rf24-rs/pull/54)
- Bump the npm group across 1 directory with 7 updates by @dependabot[bot] in [#60](https://github.com/nRF24/rf24-rs/pull/60)
- Migrate to napi-rs v3 by @2bndy5 in [#64](https://github.com/nRF24/rf24-rs/pull/64)
- Bump `rf24-rs` to v0.3.1 by @2bndy5 in [`70bdf19`](https://github.com/nRF24/rf24-rs/commit/70bdf197249712bab51fb59b34397598ba2fa86f)
- Bump `rf24ble-rs` to v0.1.4 by @2bndy5 in [`0ff3ffa`](https://github.com/nRF24/rf24-rs/commit/0ff3ffa3b96c19cf63537603260a468cc1d8d286)
- Bump `rf24-node` to v0.3.1 by @2bndy5 in [`fbda27e`](https://github.com/nRF24/rf24-rs/commit/fbda27e6885b590aa2eca9625527afc81e4d46ca)

### <!-- 8 --> 📝 Documentation

- Revise docs about `RF24::tx_delay` property. by @2bndy5 in [`e7b29c1`](https://github.com/nRF24/rf24-rs/commit/e7b29c14ac33cdebd4437ef64f89d2a340f89613)
- Switch to `nur` by @2bndy5 in [#50](https://github.com/nRF24/rf24-rs/pull/50)
- Use typedoc default HTML output by @2bndy5 in [#59](https://github.com/nRF24/rf24-rs/pull/59)

### <!-- 9 --> 🗨️ Changed

- Correct typo in `napi prepublish` command by @2bndy5 in [`e9e26a4`](https://github.com/nRF24/rf24-rs/commit/e9e26a4dc60a8e114eca897d81b7cf091f9b822f)

[rf24-node/0.3.1]: https://github.com/nRF24/rf24-rs/compare/rf24-node/0.3.0...rf24-node/0.3.1

Full commit diff: [`rf24-node/0.3.0...rf24-node/0.3.1`][rf24-node/0.3.1]

## [rf24-node/0.3.0] - 2025-05-04

### <!-- 6 --> 📦 Dependency updates

- Bump the npm group with 2 updates by @dependabot[bot] in [#35](https://github.com/nRF24/rf24-rs/pull/35)
- Bump the npm group across 1 directory with 6 updates by @dependabot[bot] in [#39](https://github.com/nRF24/rf24-rs/pull/39)
- Bump `rf24-rs` to v0.3.0 by @2bndy5 in [`abb8fda`](https://github.com/nRF24/rf24-rs/commit/abb8fdab9575ef30fa3445067aca11f21f07dfbb)
- Bump `rf24-node` to v0.3.0 by @2bndy5 in [`2bff30c`](https://github.com/nRF24/rf24-rs/commit/2bff30cc7500ca556acaa84ed9a239bdcb4b8061)

### <!-- 7 -->🚦 Tests

- Use concrete error type by @2bndy5 in [#42](https://github.com/nRF24/rf24-rs/pull/42)
- Replace `open_tx_pipe()` with `as_tx(Option<address>)` by @2bndy5 in [#41](https://github.com/nRF24/rf24-rs/pull/41)

### <!-- 9 --> 🗨️ Changed

- Remove `_` prefix from private members by @2bndy5 in [`39d8287`](https://github.com/nRF24/rf24-rs/commit/39d8287461777bbf9d8a1c1a92636b46b29669d0)

[rf24-node/0.3.0]: https://github.com/nRF24/rf24-rs/compare/rf24-node/0.2.2...rf24-node/0.3.0

Full commit diff: [`rf24-node/0.2.2...rf24-node/0.3.0`][rf24-node/0.3.0]

## [rf24-node/0.2.2] - 2025-04-08

### <!-- 6 --> 📦 Dependency updates

- Upgrade to yarn modern by @2bndy5 in [#33](https://github.com/nRF24/rf24-rs/pull/33)
- Bump the npm group across 1 directory with 2 updates by @dependabot[bot] in [#34](https://github.com/nRF24/rf24-rs/pull/34)
- Update defmt requirement from 0.3.10 to 1.0.1 in the cargo group by @dependabot[bot] in [#31](https://github.com/nRF24/rf24-rs/pull/31)
- Bump `rf24ble-rs` to v0.1.2 by @2bndy5 in [`33fe013`](https://github.com/nRF24/rf24-rs/commit/33fe0130101feb42aaa49aa5b88ac928034ec261)
- Bump `rf24-node` to v0.2.2 by @2bndy5 in [`9673628`](https://github.com/nRF24/rf24-rs/commit/96736283d11aced8c58cb1e871c71829d57a50bf)

### <!-- 8 --> 📝 Documentation

- Update `rf24-node` README by @2bndy5 in [`85ce48a`](https://github.com/nRF24/rf24-rs/commit/85ce48a87aadb16189bd08285e96b767c410a8f1)

### <!-- 9 --> 🗨️ Changed

- Regenerate change logs by @2bndy5 in [`581751a`](https://github.com/nRF24/rf24-rs/commit/581751af27d074797b4749572f05e9f8b3548e21)

[rf24-node/0.2.2]: https://github.com/nRF24/rf24-rs/compare/rf24-node/0.2.1...rf24-node/0.2.2

Full commit diff: [`rf24-node/0.2.1...rf24-node/0.2.2`][rf24-node/0.2.2]

## [rf24-node/0.2.1] - 2025-04-06

### <!-- 4 --> 🛠️ Fixed

- Use const for max BLE payload size by @2bndy5 in [#30](https://github.com/nRF24/rf24-rs/pull/30)

### <!-- 6 --> 📦 Dependency updates

- Bump `rf24-rs` to v0.2.1 by @2bndy5 in [`f266b96`](https://github.com/nRF24/rf24-rs/commit/f266b9695f1c492cce1ea7720a6df4fde298c338)
- Bump `rf24ble-rs` to v0.1.0 by @2bndy5 in [`1513ada`](https://github.com/nRF24/rf24-rs/commit/1513ada7aa678588ef153cbe1511021efeb7b286)
- Bump `rf24ble-rs` to v0.1.1 by @2bndy5 in [`3094968`](https://github.com/nRF24/rf24-rs/commit/3094968d17f63dea1594b0438534319f3aac5e89)
- Bump `rf24-node` to v0.2.1 by @2bndy5 in [`75c57d3`](https://github.com/nRF24/rf24-rs/commit/75c57d3b2f140ab15e95f6de76b771aaa24a8591)

[rf24-node/0.2.1]: https://github.com/nRF24/rf24-rs/compare/rf24-node/0.2.0...rf24-node/0.2.1

Full commit diff: [`rf24-node/0.2.0...rf24-node/0.2.1`][rf24-node/0.2.1]

## [rf24-node/0.2.0] - 2025-04-06

### <!-- 1 --> 🚀 Added

- Add fake BLE API for nRF24L01 by @2bndy5 in [#25](https://github.com/nRF24/rf24-rs/pull/25)

### <!-- 10 --> 💥 Breaking Changes

- Rename EsbConfig to RadioConfig by @2bndy5 in [#19](https://github.com/nRF24/rf24-rs/pull/19)

### <!-- 6 --> 📦 Dependency updates

- Bump `rf24-rs` to v0.1.1 by @2bndy5 in [`8ca278b`](https://github.com/nRF24/rf24-rs/commit/8ca278bbbff72514c8c84001bbd3480d4ba7d1d9)
- Bump `rf24-rs` to v0.1.2 by @2bndy5 in [`81dd350`](https://github.com/nRF24/rf24-rs/commit/81dd350634880a4a76f3817e0e85d8099490fb37)
- Bump `rf24-rs` to v0.2.0 by @2bndy5 in [`5ce9ac4`](https://github.com/nRF24/rf24-rs/commit/5ce9ac456ec1e1bb00613e433ec8636919c58495)
- Bump the npm group with 9 updates by @dependabot[bot] in [#29](https://github.com/nRF24/rf24-rs/pull/29)
- Bump `rf24-node` to v0.2.0 by @2bndy5 in [`4ccb8aa`](https://github.com/nRF24/rf24-rs/commit/4ccb8aa09a06b154efeb9007ba1b3a88ea7c5682)

### <!-- 7 -->🚦 Tests

- Improve ``rf24-rs`` tests by @2bndy5 in [#26](https://github.com/nRF24/rf24-rs/pull/26)

### <!-- 8 --> 📝 Documentation

- Update API docs for bindings by @2bndy5 in [#20](https://github.com/nRF24/rf24-rs/pull/20)
- Various doc updates by @2bndy5 in [#22](https://github.com/nRF24/rf24-rs/pull/22)

### <!-- 9 --> 🗨️ Changed

- Reassess min supported rust version by @2bndy5 in [`a9ca278`](https://github.com/nRF24/rf24-rs/commit/a9ca278b3ed38a682bba54bbf32de2b874ae9097)
- Dev workflow improvements by @2bndy5 in [`c493356`](https://github.com/nRF24/rf24-rs/commit/c493356b8044655f5a1930e7ef240243fa990d34)
- Reorganize bindings' sources by @2bndy5 in [`3383200`](https://github.com/nRF24/rf24-rs/commit/33832000723857bf7b09a94c4ab892adc9cc66bf)
- Improve readability in `rf24-rs` sources by @2bndy5 in [`f468315`](https://github.com/nRF24/rf24-rs/commit/f4683153d72bd67b0a7707a3a922a0d03b852164)

[rf24-node/0.2.0]: https://github.com/nRF24/rf24-rs/compare/rf24-node/0.1.2...rf24-node/0.2.0

Full commit diff: [`rf24-node/0.1.2...rf24-node/0.2.0`][rf24-node/0.2.0]

## [rf24-node/0.1.2] - 2025-03-09

### <!-- 6 --> 📦 Dependency updates

- Bump `rf24-node` to v0.1.2 by @2bndy5 in [`48c6631`](https://github.com/nRF24/rf24-rs/commit/48c663146c989e05e2506eda67fcbe0795e68c98)

### <!-- 9 --> 🗨️ Changed

- Run `npm pkg fix` on node bindings by @2bndy5 in [`917b7c1`](https://github.com/nRF24/rf24-rs/commit/917b7c138d0f809c4959b0aee860d2cfb7f15fac)

[rf24-node/0.1.2]: https://github.com/nRF24/rf24-rs/compare/rf24-node/0.1.1...rf24-node/0.1.2

Full commit diff: [`rf24-node/0.1.1...rf24-node/0.1.2`][rf24-node/0.1.2]

## [rf24-node/0.1.1] - 2025-03-09

### <!-- 6 --> 📦 Dependency updates

- Bump `rf24-node` to v0.1.1 by @2bndy5 in [`217f51c`](https://github.com/nRF24/rf24-rs/commit/217f51cde5c33451b9fbf845ec0a2ca89902e990)

[rf24-node/0.1.1]: https://github.com/nRF24/rf24-rs/compare/rf24-node/0.1.0...rf24-node/0.1.1

Full commit diff: [`rf24-node/0.1.0...rf24-node/0.1.1`][rf24-node/0.1.1]

## [rf24-node/0.1.0] - 2025-03-09

### <!-- 1 --> 🚀 Added

- Implement nRF24L01 driver by @2bndy5 in [#1](https://github.com/nRF24/rf24-rs/pull/1)

### <!-- 6 --> 📦 Dependency updates

- Specify repo metadata in package.json by @2bndy5 in [`3e2ad68`](https://github.com/nRF24/rf24-rs/commit/3e2ad68e1de85e1622a3c474c4853c2620e5f103)

### <!-- 9 --> 🗨️ Changed

- Prepare release CI by @2bndy5 in [#14](https://github.com/nRF24/rf24-rs/pull/14)

[rf24-node/0.1.0]: https://github.com/nRF24/rf24-rs/compare/f8863cc36d66708bfa0fb2fb1a219c7b2f97f7d6...rf24-node/0.1.0

Full commit diff: [`f8863cc...rf24-node/0.1.0`][rf24-node/0.1.0]

<!-- generated by git-cliff -->

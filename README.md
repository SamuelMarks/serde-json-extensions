serde-json-extensions
=====================
[![License](https://img.shields.io/badge/license-Apache--2.0%20OR%20MIT-blue.svg)](https://opensource.org/licenses/Apache-2.0)

A few useful extensions that the [`serde_json`](https://github.com/serde-rs/json) people probably don't want. Including:

  - `ValueNoObjOrArr` [`serde_json::value::Value`](https://docs.rs/serde_json/latest/serde_json/value/enum.Value.html) without object or array discriminant
  - `ScalarOrArrayValue` [`serde_json::value::Value`](https://docs.rs/serde_json/latest/serde_json/value/enum.Value.html) without object discriminant

FYI: Code is [heavily] taken from [`serde_json`](https://github.com/serde-rs/json) @ [`27a4ca9`](https://github.com/serde-rs/json/tree/27a4ca9d7a62394fe8f0103f3d91de59f055a4c4).

---

## Development guide

### Install Rust

Follow the [official alt-guide](https://forge.rust-lang.org/infra/other-installation-methods.html#other-ways-to-install-rustup) or alternatively run one of the following:

#### Non-Windows
```sh
$ curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh sh -s -- --default-toolchain nightly
```

#### Windows
```cmd
> curl --proto '=https' --tlsv1.2 -sSfO https://static.rust-lang.org/rustup/dist/i686-pc-windows-gnu/rustup-init.exe
> rustup-init --default-toolchain nightly
```

### Build project
```sh
$ cargo build
```

## Contribution guide
Ensure all tests are passing [`cargo test`](https://doc.rust-lang.org/cargo/commands/cargo-test.html) and [`rustfmt`](https://github.com/rust-lang/rustfmt) has been run. This can be with [`cargo make`](https://github.com/sagiegurari/cargo-make); installable with:

```sh
$ cargo install --force cargo-make
```

Then run:
```sh
$ cargo make
```

Finally, we recommend [feature-branches](https://martinfowler.com/bliki/FeatureBranch.html) with an accompanying [pull-request](https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/proposing-changes-to-your-work-with-pull-requests/about-pull-requests).

---

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

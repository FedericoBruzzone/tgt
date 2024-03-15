[crates-io]: https://crates.io/crates/tgt
[crates-io-shield]: https://img.shields.io/crates/v/tgt
[github-ci-linux]: https://github.com/FedericoBruzzone/tgt/actions/workflows/build-linux.yml
[github-ci-linux-shield]: https://github.com/FedericoBruzzone/tgt/actions/workflows/build-linux.yml/badge.svg
[github-ci-windows]: https://github.com/FedericoBruzzone/tgt/actions/workflows/build-windows.yml
[github-ci-windows-shield]: https://github.com/FedericoBruzzone/tgt/actions/workflows/build-windows.yml/badge.svg
[github-ci-macos]: https://github.com/FedericoBruzzone/tgt/actions/workflows/build-macos.yml
[github-ci-macos-shield]: https://github.com/FedericoBruzzone/tgt/actions/workflows/build-macos.yml/badge.svg
[github-license]: https://github.com/FedericoBruzzone/tgt/blob/main/LICENSE
[github-license-shield]: https://img.shields.io/github/license/FedericoBruzzone/tgt
[total-lines]: https://github.com/FedericoBruzzone/tgt
[total-lines-shield]: https://tokei.rs/b1/github/FedericoBruzzone/tgt?type=Rust,Python
[creates-io-downloads]: https://crates.io/crates/tgt
[creates-io-downloads-shield]: https://img.shields.io/crates/d/tgt.svg

> [!WARNING]
> This project is still in development and is not ready for use.

<p align="center">
    <img src="https://github.com/FedericoBruzzone/tgt/raw/main/imgs/logo.png" alt="logo" />
</p>
<p align="center">
    <b>A simple TUI for Telegram</b>
</p>

[![Crates.io][crates-io-shield]][crates-io]
[![GitHub CI Linux][github-ci-linux-shield]][github-ci-linux]
[![GitHub CI Windows][github-ci-windows-shield]][github-ci-windows]
[![GitHub CI macOS][github-ci-macos-shield]][github-ci-macos]
[![GitHub License][github-license-shield]][github-license]
[![Crates.io Downloads][creates-io-downloads-shield]][creates-io-downloads]
[![][total-lines-shield]][total-lines]

## About

`tgt` is a terminal user interface for Telegram, written in Rust.

## Contributing

Contributions to this project are welcome! If you have any suggestions, improvements, or bug fixes, feel free to submit a pull request.
For more information, do not hesitate to contact us (see the [Contact](#contact) section).

The [CONFIGURE_TDLIB.md](https://github.com/FedericoBruzzone/tgt/blob/main/docs/configuration/CONFIGURE_TDLIB.md) file contains the steps to configure the `tdlib` in your local environment, starting from the compilation to the configuration of the environment variables.

### Road Map

You can find the road map of the project [here](https://github.com/FedericoBruzzone/tg-tui/issues/1) (in the pinned issues).

## Building

You can use `just`, `make` or `cargo`,  as build tools.
If you want to use `cargo`, please make sure to read the `Justfile` or the `Makefile` to understand the flags used for each command.
Here are the available commands:

```text
just COMMAND
make COMMAND

COMMAND:
  all    # fmt, clippy, test, build
  build  # Build the project
  run    # Run the project
  fmt    # Format the code
  clippy # Run clippy
  test   # Run the tests
  clean  # Clean the project
  help   # Print the help message
```

## License

This repository is licensed under the [Apache License 2.0](https://www.apache.org/licenses/LICENSE-2.0). Please review the license file provided in the repository for more information regarding the terms and conditions of the license.

## Contact

- Email:
  - [federico.bruzzone.i@gmail.com]
  - [federico.bruzzone@studenti.unimi.it]
  - [andrea.longoni3@studenti.unimi.it]
- GitHub:
  - [FedericoBruzzone](https://github.com/FedericoBruzzone)
  - [Andreal2000](https://github.com/Andreal2000)

<!-- [docs-rs]: https://docs.rs/tgt -->
<!-- [docs-rs-shield]: https://docs.rs/tgt/badge.svg -->
<!-- [![Docs.rs][docs-rs-shield]][docs-rs] -->

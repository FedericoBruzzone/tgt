[crates-io]: https://crates.io/crates/tgt
[crates-io-shield]: https://img.shields.io/crates/v/tgt
[github-ci-linux]: https://github.com/FedericoBruzzone/tgt/actions/workflows/ci-linux.yml
[github-ci-linux-shield]: https://github.com/FedericoBruzzone/tgt/actions/workflows/ci-linux.yml/badge.svg
[github-ci-windows]: https://github.com/FedericoBruzzone/tgt/actions/workflows/ci-windows.yml
[github-ci-windows-shield]: https://github.com/FedericoBruzzone/tgt/actions/workflows/ci-windows.yml/badge.svg
[github-ci-macos]: https://github.com/FedericoBruzzone/tgt/actions/workflows/ci-macos.yml
[github-ci-macos-shield]: https://github.com/FedericoBruzzone/tgt/actions/workflows/ci-macos.yml/badge.svg
[github-ci-android]: https://github.com/FedericoBruzzone/tgt/actions/workflows/ci-android.yml
[github-ci-android-shield]: https://github.com/FedericoBruzzone/tgt/actions/workflows/ci-android.yml/badge.svg
[github-license-mit]: https://github.com/FedericoBruzzone/tgt/blob/main/LICENSE-MIT
[github-license-apache]: https://github.com/FedericoBruzzone/tgt/blob/main/LICENSE-APACHE
[github-license-shield]: https://img.shields.io/github/license/FedericoBruzzone/tgt
[total-lines]: https://github.com/FedericoBruzzone/tgt
[total-lines-shield]: https://tokei.rs/b1/github/FedericoBruzzone/tgt?type=Rust,Python
[creates-io-downloads]: https://crates.io/crates/tgt
[creates-io-downloads-shield]: https://img.shields.io/crates/d/tgt.svg

⚠️ Note that this is the first release of `tgt`. Please consider to open an issue if you find any bug or if you have any suggestion. ⚠️

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
[![GitHub CI Android][github-ci-android-shield]][github-ci-android]

<!-- [![GitHub License][github-license-shield]][github-license-apache] -->

![license](https://img.shields.io/crates/l/tgt)
[![Crates.io Downloads][creates-io-downloads-shield]][creates-io-downloads]
[![][total-lines-shield]][total-lines]

## About

`tgt` is a terminal user interface for Telegram, written in Rust.

<p align="center">
  <img src="./imgs/example_movie.gif" alt="animated" />
</p>

<!-- <p align="center"> -->
<!--     <img src="https://github.com/FedericoBruzzone/tgt/raw/main/imgs/example.png" alt="example"/> -->
<!-- </p> -->

## Usage

**From crates.io**

```bash
cargo install tgt
```

**From source downloading the tdlib**

```bash
cargo build --release --features download-tdlib
```

After the installation, you can run `tgt` with the following command:

```bash
tgt --help
```

### Features

Build features can be combined (e.g. `cargo build --release --features download-tdlib,chafa-dyn`).

| Feature | Description |
|--------|-------------|
| `default` | Enables `download-tdlib` and `voice-message`. |
| `download-tdlib` | Download and use TDLib automatically (recommended for most users). |
| `local-tdlib` | Use TDLib from path in `LOCAL_TDLIB_PATH`. |
| `pkg-config` | Find TDLib via pkg-config. |
| `static` | Statically link `tdjson` (use with `download-tdlib` or `local-tdlib`). No runtime `tdjson` dependency needed. |
| `voice-message` | Play Telegram voice notes (OGG Opus) and other audio (e.g. MP3). **Requires CMake** to build the Opus dependency. Enabled by default; use `--no-default-features` and then add back only the features you need (e.g. `--features download-tdlib`) to disable voice. |
| `chafa-dyn` | Enable [chafa](https://github.com/hpjansson/chafa)-based image rendering in the photo viewer (dynamic linking). Requires the chafa library installed on the system. Not supported on Windows ARM. |
| `chafa-static` | Same as `chafa-dyn` but links chafa statically. Not supported on Windows ARM. |

**Voice messages**  
If you build with the default features (or with `voice-message` enabled), you must have **CMake** installed so the Opus library can be built. Other audio formats (e.g. MP3) use rodio only; only Telegram voice notes (Opus) need this. If you build with `--no-default-features` and do not enable `voice-message`, voice playback is disabled and the app will show a message when you try to play a voice note.

**Installation methods for CMake (when using voice-message)**

- **macOS** — Homebrew: `brew install cmake`
- **Linux** — use your package manager, e.g. `sudo apt install cmake` (Debian/Ubuntu), `sudo dnf install cmake` (Fedora), `sudo pacman -S cmake` (Arch)
- **Windows** — [CMake installer](https://cmake.org/download/) or e.g. `winget install Kitware.CMake`

**Chafa (image rendering)**  
The `chafa-dyn` and `chafa-static` features use the [chafa](https://github.com/hpjansson/chafa) library to display images in the terminal. You must have chafa installed to use these features.

**Installation methods for chafa**

- **Linux (Debian/Ubuntu/Kali, Fedora, Arch, openSUSE, etc.)** — use your package manager:
  - `sudo apt install chafa` (Debian/Ubuntu/Kali)
  - `sudo dnf install chafa` (Fedora)
  - `sudo pacman -S chafa` (Arch Linux)
  - `sudo zypper in chafa` (openSUSE)
  - `sudo emerge media-gfx/chafa` (Gentoo)
- **macOS** — Homebrew or MacPorts:
  - `brew install chafa`
  - `sudo port install chafa`
- **Windows** — Scoop or Windows Package Manager (winget):
  - `scoop install chafa`
  - `winget install hpjansson.Chafa`

### Arch Linux

Thanks to [x-leehe](https://github.com/x-leehe) for creating the [AUR package](https://aur.archlinux.org/packages/tgt-client-git). You can install `tgt` from the AUR:

```bash
yay -S tgt-client-git
```

### NixOS

**From `flake.nix`**

Config directories are created automatically on first run, or you can generate initial config with `tgt init-config` (see [CONFIG.md](CONFIG.md)). Then you have two installation options:

1. Run directly with `nix run`:

```bash
nix run github:FedericoBruzzone/tgt
```

2. Add `tgt` to your system packages:

Add the following to your `flake.nix`:

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable"
    tgt.url = "github:FedericoBruzzone/tgt";
    tgt.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { nixpkgs, tgt, ... }: { /* ... */ }
}
```

Then add it to your `environment.systemPackages`:

```nix
{pkgs, tgt, ...}: {
  environment = {
    systemPackages = [
        (tgt.packages.${pkgs.system}.default)
    ];
  };
}
```

To use a specific version of the program, override the `src` attribute:

```nix
{pkgs, tgt, ...}: {
  environment = {
    systemPackages = [
      (tgt.packages.${pkgs.system}.default.overrideAttrs (old: {
        src = pkgs.fetchFromGitHub {
          owner = old.src.owner;
          repo = old.src.repo;
          rev = "00000000000000000000000000000000000000";
          sha256 = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
        };
        cargoHash = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
      }))
    ];
  };
}
```

### Docker build

- The Docker image is built using `rust:1.91-trixie` as the base.

- Run Docker image in interactive mode to open a bash shell. Specify a container name to reuse when required.

```bash
docker run -it --name <container_name> ghcr.io/FedericoBruzzone/tgt:<version>
```

- Run `tgt` command from bash shell.

```bash
tgt
```

- for reusing the same container for next boot cycles, start the container

```bash
docker container start <container_name>
```

- run `tgt` by opening interactive shell from the container

```bash
docker exec -it <container_name> bash
```

### Configuration

`tgt` is fully customizable. Config uses **XDG-style paths** (e.g. `~/.config/tgt` on Linux) with **backwards compatibility** for the legacy `~/.tgt` folder. The app **creates config directories** at startup if missing, and **bundles default configs** so it works out of the box after `cargo install`. Configs are **versioned** and the program adds missing keybindings on upgrade. Use `tgt init-config` to generate initial config files and `tgt clear --config` (or `--data` / `--logs` / `--all`) to remove them for a fresh start.

For config locations, versioning, CLI commands, keybindings, and per-file details, see **[CONFIG.md](CONFIG.md)**.

## Contributing

Contributions to this project are welcome! If you have any suggestions, improvements, or bug fixes, feel free to submit a pull request.
For more information, do not hesitate to contact us (see the [Contact](#contact) section).

**Build instructions**

There are three ways to build `tgt`:

1. Using the `download-tdlib` feature of [tdlib-rs](https://github.com/FedericoBruzzone/tdlib-rs) you do not need to set any environment variable. Then you can compile `tgt` using `cargo build --features download-tdlib`.
2. By default, `tgt` assumes that you have the tdlib built and the `LOCAL_TDLIB_PATH` environment variable set to the path of the `tdlib` directory. You can set the environment variable with the following command: `export LOCAL_TDLIB_PATH="/path/to/tdlib"`. Then you can compile `tgt` using `cargo build` or `cargo build --feature default`.
3. You can use `pkg-config` to find the path of the library. In this case see the [CONTRIBUTING.md](https://github.com/FedericoBruzzone/tgt/blob/main/CONTRIBUTING.md) file for more information. Then you can compile `tgt` using `cargo build --features pkg-config`.

You can also add `static` to statically link `tdjson`, so the final binary does not require `tdjson` installed at runtime (e.g. `cargo build --features download-tdlib,static`).

The [CONTRIBUTING.md](https://github.com/FedericoBruzzone/tgt/blob/main/CONTRIBUTING.md) file contains information for building `tgt` and the steps to configure the `tdlib` in your local environment, starting from the compilation to the configuration of the environment variables.

### Road Map

You can find the road map of the project [here](https://github.com/FedericoBruzzone/tgt/issues/37) (in the pinned issues).

## Commands

You can use `make` or `cargo`, as build tools.
If you want to use `cargo`, please make sure to read the the `Makefile` to understand the flags used for each command.
Here are the available commands:

```text
make COMMAND

COMMAND:
  all            # Run fmt, clippy and test
  build          # Build the project
  run            # Run the project
  test           # Run the tests
  clippy         # Run clippy
  fmt            # Run rustfmt
  clean          # Clean the project
```

## License

This repository is licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE][github-license-apache] or http://www.apache.org/licenses/LICENSE-2.0)

- MIT license ([LICENSE-MIT][github-license-mit] or http://opensource.org/licenses/MIT)

at your option.

Please review the license file provided in the repository for more information regarding the terms and conditions of the license.

## Contact

If you have any questions, suggestions, or feedback, do not hesitate to [contact me](https://federicobruzzone.github.io/).

Maintainers:

- [FedericoBruzzone](https://github.com/FedericoBruzzone)
- [Andreal2000](https://github.com/Andreal2000)

<!-- [docs-rs]: https://docs.rs/tgt -->
<!-- [docs-rs-shield]: https://docs.rs/tgt/badge.svg -->
<!-- [![Docs.rs][docs-rs-shield]][docs-rs] -->

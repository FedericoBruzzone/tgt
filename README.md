[crates-io]: https://crates.io/crates/tgt
[crates-io-shield]: https://img.shields.io/crates/v/tgt
[github-ci-linux]: https://github.com/FedericoBruzzone/tgt/actions/workflows/ci-linux.yml
[github-ci-linux-shield]: https://github.com/FedericoBruzzone/tgt/actions/workflows/ci-linux.yml/badge.svg
[github-ci-windows]: https://github.com/FedericoBruzzone/tgt/actions/workflows/ci-windows.yml
[github-ci-windows-shield]: https://github.com/FedericoBruzzone/tgt/actions/workflows/ci-windows.yml/badge.svg
[github-ci-macos]: https://github.com/FedericoBruzzone/tgt/actions/workflows/ci-macos.yml
[github-ci-macos-shield]: https://github.com/FedericoBruzzone/tgt/actions/workflows/ci-macos.yml/badge.svg
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

### Arch Linux

Thanks to [x-leehe](https://github.com/x-leehe) for creating the [AUR package](https://aur.archlinux.org/packages/tgt-client-git). You can install `tgt` from the AUR:

```bash
yay -S tgt-client-git
```

### NixOS

**From `flake.nix`**

First, create the required TOML configuration files in `~/.tgt/config` using these commands:

```bash
git clone https://github.com/FedericoBruzzone/tgt ~/tgt
mkdir -p ~/.tgt/config
cp ~/tgt/config/* ~/.tgt/config
```

After setting up the configuration files, you have two installation options:

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

Note that `tgt` is fully customizable.
For more information about the **configuration**, please look at [here](https://github.com/FedericoBruzzone/tgt/tree/main/docs/configuration).
The configuration files are manyfold.

1. [`app.toml`](https://github.com/FedericoBruzzone/tgt/tree/main/config/app.toml): contains general configurations about the application (see [`app.toml.md`](https://github.com/FedericoBruzzone/tgt/tree/main/docs/configuration/app.toml.md) for explanations about each field).
2. [`keymap.toml`](https://github.com/FedericoBruzzone/tgt/tree/main/config/keymap.toml): contains the keybindings used in the application (see [`keymap.toml.md`](https://github.com/FedericoBruzzone/tgt/tree/main/docs/configuration/keymap.toml.md) for explanations about each field).
3. [`logger.toml`](https://github.com/FedericoBruzzone/tgt/tree/main/config/logger.toml): contains the logging configurations (see [`logger.toml.md`](https://github.com/FedericoBruzzone/tgt/tree/main/docs/configuration/logger.toml.md) for explanations about each field).
4. [`telegram.toml`](https://github.com/FedericoBruzzone/tgt/tree/main/config/telegram.toml): contains the Telegram API and TDLib configurations (see [`telegram.toml.md`](https://github.com/FedericoBruzzone/tgt/tree/main/docs/configuration/telegram.toml.md) for explanations about each field).
5. [`theme.toml`](https://github.com/FedericoBruzzone/tgt/tree/main/config/theme.toml): contains the color theme configurations (see [`theme.toml.md`](https://github.com/FedericoBruzzone/tgt/tree/main/docs/configuration/theme.toml.md) for explanations about each field).

> [!NOTE]
> The theme switcher only persists the selected theme in **release** mode. In debug mode, the chosen theme is not saved between sessions.

> [!WARNING]
> Generally, end-users are expected to supply their own api_id and api_hash.
> While some open-source clients, including `tgt`, may provide "default" credentials to make the "out-of-the-box" experience smoother, shipping shared credentials carries risks.
> The [`telegram.toml.md`](https://github.com/FedericoBruzzone/tgt/tree/main/docs/configuration/telegram.toml.md) file contains more information about this topic.

**Default keybindings**:

_None state_:

```bash
esc:               to the "None" state
alt+1 | alt+left:  Focus on the chat list
alt+2 | alt+right: Focus on the chat
alt+3 | alt+down:  Focus on the prompt
alt+h | alt+l:     Resize the chat list
alt+j | alt+k:     Resize the prompt
alt+n:             Toggle chat list
q | ctrl+c:        Quit
```

_Chat List_

```bash
up | down:     Move selection
enter | right: Open the chat
left:          Unselect chat

esc:               Return to the "None" state
alt+1 | alt+left:  Focus on the chat list
alt+2 | alt+right: Focus on the chat
alt+3 | alt+down:  Focus on the prompt
```

_Chat_

```bash
up | down: Scroll the messages
left:      Unselect message
y:         Copy the message
e:         Edit the message
r:         Reply to the message
d:         Delete the message for everyone
D:         Delete the message for me

esc:               Return to the "None" state
alt+1 | alt+left:  Focus on the chat list
alt+2 | alt+right: Focus on the chat
alt+3 | alt+down:  Focus on the prompt
```

_Prompt_

Note that when the prompt is focused, you can **NOT** use `q` or `ctrl+c` to quit the application, you need to press `esc` to return to the "None" state.

```bash
alt+enter:                        Send the message

left | right | up | down:         Move the cursor
ctrl+left | ctrl+b:               Move the cursor to the previous word
ctrl+right | ctrl+f:              Move the cursor to the next word
ctrl+alt+left | ctrl+a | home:    Move the cursor to the beginning of the line (also ctrl+left+b | shift+super+left | shift+super+b)
ctrl+alt+right | ctrl+e | end:    Move the cursor to the end of the line (also ctrl+right+f | shift+super+right | shift+super+f)

shift+left:                       Move the cursor left and select the text
shift+right:                      Move the cursor right and select the text
shift+up:                         Move the cursor up and select the text
shift+down:                       Move the cursor down and select the text
shift+ctrl+left:                  Select the text before the cursor
shift+ctrl+right:                 Select the text after the cursor

ctrl+c:                           Copy the selected text
ctrl+v:                           Paste the copied text

ctrl+w | ctrl+backspace | ctrl+h: Delete the word before the cursor

esc:               Return to the "None" state
alt+1 | alt+left:  Focus on the chat list
alt+2 | alt+right: Focus on the chat
alt+3 | alt+down:  Focus on the prompt
```

## Contributing

Contributions to this project are welcome! If you have any suggestions, improvements, or bug fixes, feel free to submit a pull request.
For more information, do not hesitate to contact us (see the [Contact](#contact) section).

**Build instructions**

There are three ways to build `tgt`:

1. Using the `download-tdlib` feature of [tdlib-rs](https://github.com/FedericoBruzzone/tdlib-rs) you do not need to set any environment variable. Then you can compile `tgt` using `cargo build --features download-tdlib`.
2. By default, `tgt` assumes that you have the tdlib built and the `LOCAL_TDLIB_PATH` environment variable set to the path of the `tdlib` directory. You can set the environment variable with the following command: `export LOCAL_TDLIB_PATH="/path/to/tdlib"`. Then you can compile `tgt` using `cargo build` or `cargo build --feature default`.
3. You can use `pkg-config` to find the path of the library. In this case see the [CONTRIBUTING.md](https://github.com/FedericoBruzzone/tgt/blob/main/CONTRIBUTING.md) file for more information. Then you can compile `tgt` using `cargo build --features pkg-config`.

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

# Configuration

`tgt` by default reads its default configurations from the `config` directory in the root of the project. After this, it reads configurations from the following directories (in order of precedence) and overwrites the fields specified in the default configuration:

- `$TGT_CONFIG_HOME` (if set)
- `$HOME/.config/tgt` (for Linux and macOS) and `C:\Users\<name>\AppData\Roaming\tgt` (for Windows)

Note that after the finding the first configuration file, `tgt` stops looking for more configurations, it is short-circuited.

## Configuration Files

In the configuration directory, `tgt` looks for the following files:

- `app.toml` for theme configuration (see [App Configuration](https://github.com/FedericoBruzzone/tgt/blob/main/docs/configuration/users/app.toml.md))
- `keymap.toml` for keymap configuration (see [Keymap Configuration](https://github.com/FedericoBruzzone/tgt/blob/main/docs/configuration/users/keymap.toml.md))
- `theme.toml` for theme configuration (see [Theme Configuration](https://github.com/FedericoBruzzone/tgt/blob/main/docs/configuration/users/theme.toml.md))
- `logger.toml` for logger configuration (see [Logger Configuration](https://github.com/FedericoBruzzone/tgt/blob/main/docs/configuration/users/logger.toml.md))


## Arch Linux Users

If you are an Arch Linux user, make sure to have installed:
```bash
sudo pacman -S clang14
sudo pacman -S openssl
sudo pacman -S libc++abi
sudo pacman -S libc++
sudo pacman -S libunwind
```

(OPTIONAL) and then create the following symbolic link:

```bash
sudo ln -s /usr/lib/libunwind.so.8.1.0 /usr/lib/libunwind.so.1
sudo ln -s /usr/lib/llvm14/bin/clang++ /usr/bin/clang++-14
```

and export the following environment variable:

```bash
export PATH=$PATH:/usr/lib/llvm14/bin
```


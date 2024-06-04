# Configure the project

First of all, you need to set the `API_HASH` and `API_ID` environment variables with the values of your Telegram application (you can get them from [here](https://my.telegram.org/)) or for the development phase you can use the following values:
```bash
export API_HASH="a3406de8d171bb422bb6ddf3bbd800e2"
export API_ID="94575"
```

## Build/Run using download-tdlib feature of tdlib-rs

Using the `download-tdlib` feature of [tdlib-rs](https://github.com/FedericoBruzzone/tdlib-rs) you do not need to set any environment variable.
Thanks to [tdlib-rs](https://github.com/FedericoBruzzone/tdlib-rs) you can also compile `tgt` using that feature downloading the `tdlib` automatically and build the project.
```bash
cargo build --features download-tdlib
```

Note that this way is supported only for the following platforms:
- Linux x86_64
- Windows x86_64
- MacOS x86_64
- MacOS aarch64

## Build/Run using your local TDLib

By default `tgt` assume that you have the tdlib built and the `LOCAL_TDLIB_PATH` environment variable set to the path of the `tdlib` directory.

You can set the `LOCAL_TDLIB_PATH` environment variable in the `.bashrc` or `.zshrc` file:
```bash
export LOCAL_TDLIB_PATH="/path/to/tdlib"
```

To compile the tdlib, you can see the instructions in the [Build TDLib](#build-tdlib) section.

## Using pkg-config

If you have the `tdlib` installed in your system, you can use the `pkg-config` to find the path of the library.

You can set the `PKG_CONFIG_PATH` environment variable in the `.bashrc` or `.zshrc` file:
```bash
export PKG_CONFIG_PATH="/path/to/tdlib/lib/pkgconfig:$PKG_CONFIG_PATH"
```

and then you need to tell linker where to find the library:
If you are using Linux, you can set the `LD_LIBRARY_PATH` environment variable in the `.bashrc` or `.zshrc` file:
```bash
export LD_LIBRARY_PATH="/path/to/tdlib/lib:$LD_LIBRARY_PATH"
```
If you are using MacOS, you can set the `DYLD_LIBRARY_PATH` environment variable in the `.bashrc` or `.zshrc` file:
```bash
export DYLD_LIBRARY_PATH="/path/to/tdlib/lib:$DYLD_LIBRARY_PATH"
```
If you are using Windows, you can set the `PATH` environment variable in the `.bashrc` or `.zshrc` file:
```bash
export PATH="/path/to/tdlib/bin:$PATH"
```

## Build TDLib

The steps to build TDLib can be found [here](https://tdlib.github.io/td/build.html?language=Rust), for other info check the official repository of [TDLib](https://github.com/tdlib/td).

About the `api_id` you can get one form [https://my.telegram.org](https://my.telegram.org), for other info check the official [documentation](https://core.telegram.org/api/obtaining_api_id).

Current supported TDLib version: [1.8.19](https://github.com/tdlib/td/commit/2589c3fd46925f5d57e4ec79233cd1bd0f5d0c09).

---

# Build TDLib

### MacOS (Intel)

```bash
xcode-select --install
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
brew install gperf cmake openssl
git clone https://github.com/tdlib/td.git
cd td
git checkout 2589c3fd46925f5d57e4ec79233cd1bd0f5d0c09
rm -rf build
mkdir build
cd build
cmake -DCMAKE_BUILD_TYPE=Release -DOPENSSL_ROOT_DIR=/usr/local/opt/openssl/ -DCMAKE_INSTALL_PREFIX:PATH=../tdlib ..
cmake --build . --target install
cd ..
cd ..
ls -l td/tdlib
```

Step 1:

In order to use TDLib in your rust project, copy the td/tdlib directory to the parent folder:

```bash
cp -r ~/WHERE_IS_TD/td/tdlib ~/WHERE_IS_TD
```

Step 2:

Add to the `.bashrc`:

```bash
# Note that this path is there you moved the tdlib directory in the step 1
export PKG_CONFIG_PATH=~/WHERE_IS_TDLIB/tdlib/lib/pkgconfig/:$PKG_CONFIG_PATH
export DYLD_LIBRARY_PATH=~/WHERE_IS_TDLIB/tdlib/lib/:$DYLD_LIBRARY_PATH

# Not correct
# export PKG_CONFIG_PATH=~/WHERE_IS_TD/td/build/pkgconfig/:$PKG_CONFIG_PATH
```

Step 3:

Add to the `.bashrc`:

```bash
# Warning: The API_HASH and API_ID are takern from the Telegram API
export API_HASH="a3406de8d171bb422bb6ddf3bbd800e2"
export API_ID="94575"
```

### Windows

- Note that Windows Subsystem for Linux (WSL) and Cygwin are not Windows environments, so you need to use instructions for Linux for them instead.
- Download and install Microsoft Visual Studio. Enable C++ support while installing.
- Download and install CMake; choose "Add CMake to the system PATH" option while installing.
- Download and install Git.
- Download and install pkg-config.
- Close and re-open PowerShell if the PATH environment variable was changed.

Run these commands in PowerShell to build TDLib and to install it to td/tdlib:

```powershell
git clone https://github.com/tdlib/td.git
cd td
git checkout 2589c3fd46925f5d57e4ec79233cd1bd0f5d0c09
git clone https://github.com/Microsoft/vcpkg.git
cd vcpkg
git checkout cd5e746ec203c8c3c61647e0886a8df8c1e78e41
./bootstrap-vcpkg.bat
./vcpkg.exe install gperf:x64-windows openssl:x64-windows zlib:x64-windows
cd ..
rm -rf build
mkdir build
cd build
cmake -A x64 -DCMAKE_INSTALL_PREFIX:PATH=../tdlib -DCMAKE_TOOLCHAIN_FILE:FILEPATH=../vcpkg/scripts/buildsystems/vcpkg.cmake ..
cmake --build . --target install --config Release
cd ..
cd ..
ls -l td/tdlib
```

Step 1:

```powershell
$env:PATH = $env:PATH + ";/WHERE_IS_TDLIB/tdlib/bin"
$env:PKG_CONFIG_PATH="/WHERE_IS_TDLIB/tdlib/lib/pkgconfig"
```

Step 2:

```powershell
# Warning: The API_HASH and API_ID are takern from the Telegram API
$env:API_HASH="a3406de8d171bb422bb6ddf3bbd800e2"
$env:API_ID="94575"
```

### Linux Ubuntu22 (using clang)

```bash
sudo apt-get update
sudo apt-get upgrade
sudo apt-get install make git zlib1g-dev libssl-dev gperf php-cli cmake clang-14 libc++-dev libc++abi-dev
git clone https://github.com/tdlib/td.git
cd td
git checkout 2589c3fd46925f5d57e4ec79233cd1bd0f5d0c09
rm -rf build
mkdir build
cd build
CXXFLAGS="-stdlib=libc++" CC=/usr/bin/clang-14 CXX=/usr/bin/clang++-14 cmake -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX:PATH=../tdlib ..
cmake --build . --target install
cd ..
cd ..
ls -l td/tdlib
```

Step 1:

In order to use TDLib in your rust project, copy the td/tdlib directory to the parent folder:

```bash
cp -r ~/WHERE_IS_TD/td/tdlib ~/WHERE_IS_TD
```

Step 2:

Add to the `.bashrc`:

```bash
# Note that this path is there you moved the tdlib directory in the step 1
export PKG_CONFIG_PATH=~/WHERE_IS_TDLIB/tdlib/lib/pkgconfig/:$PKG_CONFIG_PATH
export LD_LIBRARY_PATH=~/WHERE_IS_TDLIB/tdlib/lib/:$LD_LIBRARY_PATH
```

Step 3:

Add to the `.bashrc`:

```bash
# Warning: The API_HASH and API_ID are takern from the Telegram API
export API_HASH="a3406de8d171bb422bb6ddf3bbd800e2"
export API_ID="94575"
```

### Linux Other (using clang)

- Install Git, clang >= 3.4, libc++, make, CMake >= 3.0.2, OpenSSL-dev, zlib-dev, gperf, PHP using your package manager. For example, on Arch Linux, you can run: `sudo pacman -S git clang make cmake openssl libc++abi libc++ zlib gperf php`.

```bash
git clone https://github.com/tdlib/td.git
cd td
git checkout 2589c3fd46925f5d57e4ec79233cd1bd0f5d0c09
rm -rf build
mkdir build
cd build
CXXFLAGS="-stdlib=libc++" CC=/usr/bin/clang CXX=/usr/bin/clang++ cmake -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX:PATH=../tdlib ..
cmake --build . --target install
cd ..
cd ..
ls -l td/tdlib
```

Step 1:

In order to use TDLib in your rust project, copy the td/tdlib directory to the parent folder:

```bash
cp -r ~/WHERE_IS_TD/td/tdlib ~/WHERE_IS_TD
```

Step 2:

Add to the `.bashrc`:

```bash
# Note that this path is there you moved the tdlib directory in the step 1
export PKG_CONFIG_PATH=~/WHERE_IS_TDLIB/tdlib/lib/pkgconfig/:$PKG_CONFIG_PATH
export LD_LIBRARY_PATH=~/WHERE_IS_TDLIB/tdlib/lib/:$LD_LIBRARY_PATH
```

Step 3:

Add to the `.bashrc`:

```bash
# Warning: The API_HASH and API_ID are takern from the Telegram API
export API_HASH="a3406de8d171bb422bb6ddf3bbd800e2"
export API_ID="94575"
```

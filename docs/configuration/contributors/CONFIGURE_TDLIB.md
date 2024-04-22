# Configure the project

The steps to build TDLib can be found [here](https://tdlib.github.io/td/build.html?language=Rust), for other info check the official repository of [TDLib](https://github.com/tdlib/td).

About the `api_id` you can get one form [https://my.telegram.org](https://my.telegram.org), for other info check the official [documentation](https://core.telegram.org/api/obtaining_api_id).

Current supported TDLib version: [1.8.19](https://github.com/tdlib/td/commit/2589c3fd46925f5d57e4ec79233cd1bd0f5d0c09).

## Build TDLib

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

Step 4 (not always necessary):

**!The version of the library may change!**

After `cargo build`, if it fails, you may need to move explicitly the `libtdjson.1.8.19.dylib` to the `/usr/local/lib`, `/usr/lib`:

```bash
cp ~/WHERE_IS_TDLIB/tdlib/lib/libtdjson.1.8.19.dylib '/usr/local/lib/'
cp ~/WHERE_IS_TDLIB/tdlib/lib/libtdjson.1.8.19.dylib '/usr/lib/'
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

# Not correct
# export PATH=~/WHERE_IS_TDLIB/tdlib/lib/:$PATH
# export PKG_CONFIG_PATH=~/WHERE_IS_TD/td/build/pkgconfig/:$PKG_CONFIG_PATH
```

Step 3:

Add to the `.bashrc`:

```bash
# Warning: The API_HASH and API_ID are takern from the Telegram API
export API_HASH="a3406de8d171bb422bb6ddf3bbd800e2"
export API_ID="94575"
```

Step 4 (not always necessary):

**!The version of the library may change!**

After `cargo build`, if it fails, you may need to move explicitly the `libtdjson.so.1.8.19` to the `/usr/lib`:

```bash
cp ~/WHERE_IS_TDLIB/tdlib/lib/libtdjson.so.1.8.19 '/usr/lib/'
```

### Linux Other (using clang)

- Install Git, clang >= 3.4, libc++, make, CMake >= 3.0.2, OpenSSL-dev, zlib-dev, gperf, PHP using your package manager.

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

# Not correct
# export PATH=~/WHERE_IS_TDLIB/tdlib/lib/:$PATH
# export PKG_CONFIG_PATH=~/WHERE_IS_TD/td/build/pkgconfig/:$PKG_CONFIG_PATH
```

Step 3:

Add to the `.bashrc`:

```bash
# Warning: The API_HASH and API_ID are takern from the Telegram API
export API_HASH="a3406de8d171bb422bb6ddf3bbd800e2"
export API_ID="94575"
```

Step 4 (not always necessary):

**!The version of the library may change!**

After `cargo build`, if it fails, you may need to move explicitly the `libtdjson.so.1.8.19` to the `/usr/lib`:

```bash
cp ~/WHERE_IS_TDLIB/tdlib/lib/libtdjson.so.1.8.19 '/usr/lib/'
```

# Configure the project

## Build TDLib

### MacOS (Intel)

```bash
xcode-select --install
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
brew install gperf cmake openssl
git clone https://github.com/tdlib/td.git
cd td
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
cp td/tdlib ../
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

Step 3 (not always necessary):

**!The version of the library may change!**

After `cargo build`, if it fails, you may need to move explicitly the `libtdjson.1.8.25.dylib` to the `/usr/local/lib`, `/usr/lib` or `/Users/NAME/lib`:
```bash
cp ~/WHERE_IS_TDLIB/tdlib/lib/libtdjson.1.8.25.dylib '/Users/NAME/lib/'
cp ~/WHERE_IS_TDLIB/tdlib/lib/libtdjson.1.8.25.dylib'/usr/local/lib/'
cp ~/WHERE_IS_TDLIB/tdlib/lib/libtdjson.1.8.25.dylib '/usr/lib/libtd'
```

### Windows

- Note that Windows Subsystem for Linux (WSL) and Cygwin are not Windows environments, so you need to use instructions for Linux for them instead.
- Download and install Microsoft Visual Studio. Enable C++ support while installing.
- Download and install CMake; choose "Add CMake to the system PATH" option while installing.
- Download and install Git.
- Download and unpack PHP. Add the path to php.exe to the PATH environment variable.
- Close and re-open PowerShell if the PATH environment variable was changed.

Run these commands in PowerShell to build TDLib and to install it to td/tdlib:
```powershell
git clone https://github.com/tdlib/td.git
cd td
git clone https://github.com/Microsoft/vcpkg.git
cd vcpkg
git checkout cd5e746ec203c8c3c61647e0886a8df8c1e78e41
./bootstrap-vcpkg.bat
./vcpkg.exe install gperf:x86-windows openssl:x86-windows zlib:x86-windows
cd ..
Remove-Item build -Force -Recurse -ErrorAction SilentlyContinue
mkdir build
cd build
cmake -A Win32 -DCMAKE_INSTALL_PREFIX:PATH=../tdlib -DCMAKE_TOOLCHAIN_FILE:FILEPATH=../vcpkg/scripts/buildsystems/vcpkg.cmake ..
cmake --build . --target install --config Release
cd ..
cd ..
dir td/tdlib
```

### Linux Ubuntu22 (using clang)

```bash
sudo apt-get update
sudo apt-get upgrade
sudo apt-get install make git zlib1g-dev libssl-dev gperf php-cli cmake clang-14 libc++-dev libc++abi-dev
git clone https://github.com/tdlib/td.git
cd td
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
cp td/tdlib ../
```

Step 2:

Add to the `.bashrc`:
```bash
# Note that this path is there you moved the tdlib directory in the step 1
export PKG_CONFIG_PATH=~/WHERE_IS_TDLIB/tdlib/lib/pkgconfig/:$PKG_CONFIG_PATH
export PATH=~/WHERE_IS_TDLIB/tdlib/lib/:$PATH # (Not always necessary)

# Not correct
# export PKG_CONFIG_PATH=~/WHERE_IS_TD/td/build/pkgconfig/:$PKG_CONFIG_PATH
```

Step 3 (not always necessary):

**!The version of the library may change!**

After `cargo build`, if it fails, you may need to move explicitly the `libtdjson.1.8.25.dylib` to the `/usr/local/lib`, `/usr/lib` or `/Users/NAME/lib`:
```bash
cp ~/WHERE_IS_TDLIB/tdlib/lib/libtdjson.1.8.25.dylib '/Users/NAME/lib/'
cp ~/WHERE_IS_TDLIB/tdlib/lib/libtdjson.1.8.25.dylib'/usr/local/lib/'
cp ~/WHERE_IS_TDLIB/tdlib/lib/libtdjson.1.8.25.dylib '/usr/lib/libtd'
```

### Linux Other (using clang)

- Install Git, clang >= 3.4, libc++, make, CMake >= 3.0.2, OpenSSL-dev, zlib-dev, gperf, PHP using your package manager.

```bash
git clone https://github.com/tdlib/td.git
cd td
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
cp td/tdlib ../
```

Step 2:

Add to the `.bashrc`:
```bash
# Note that this path is there you moved the tdlib directory in the step 1
export PKG_CONFIG_PATH=~/WHERE_IS_TDLIB/tdlib/lib/pkgconfig/:$PKG_CONFIG_PATH
export PATH=~/WHERE_IS_TDLIB/tdlib/lib/:$PATH # (Not always necessary)

# Not correct
# export PKG_CONFIG_PATH=~/WHERE_IS_TD/td/build/pkgconfig/:$PKG_CONFIG_PATH
```

Step 3 (not always necessary):

**!The version of the library may change!**

After `cargo build`, if it fails, you may need to move explicitly the `libtdjson.1.8.25.dylib` to the `/usr/local/lib`, `/usr/lib` or `/Users/NAME/lib`:
```bash
cp ~/WHERE_IS_TDLIB/tdlib/lib/libtdjson.1.8.25.dylib '/Users/NAME/lib/'
cp ~/WHERE_IS_TDLIB/tdlib/lib/libtdjson.1.8.25.dylib'/usr/local/lib/'
cp ~/WHERE_IS_TDLIB/tdlib/lib/libtdjson.1.8.25.dylib '/usr/lib/libtd'
```


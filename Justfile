# vim: set ft=make :
set windows-powershell := true
export RUST_BACKTRACE := "1"
project_name := "tgt"

_default:
  just --list --justfile {{justfile()}}

# Run fmt, clippy, test
all: fmt clippy test

# Build the project using cargo; you need to have setup the LOCAL_TDLIB_PATH environment variable
build_local:
  cargo build

# Build the project using cargo; it will download the tdlib library thanks to the tdlib-rs crate
build_download:
  cargo build --features download-tdlib

# Run the project using cargo; you need to have setup the LOCAL_TDLIB_PATH environment variable
# Example: just run_local BIN="bin" BIN_NAME="get_me"
run_local BIN="" BIN_NAME="":
  cargo run {{BIN}} {{BIN_NAME}}

# Run the project using cargo; it will download the tdlib library thanks to the tdlib-rs crate
# Example: just run_download BIN="bin" BIN_NAME="get_me"
run_download BIN="" BIN_NAME="":
  cargo run --features download-tdlib {{BIN}} {{BIN_NAME}}

# Format the code using cargo
fmt:
  cargo fmt
  cargo fmt -- --check

# Format the code using cargo nightly
fmt_nightly:
  cargo +nightly fmt
  cargo +nightly fmt -- --check

# Run clippy using cargo
clippy:
  cargo clippy --all-targets --all-features -- -D warnings

# Run tests using cargo
test:
  cargo test -- --nocapture --test-threads=1

# Clean the project using cargo
clean:
  cargo clean

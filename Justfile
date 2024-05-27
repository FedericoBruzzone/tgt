# vim: set ft=make :
set windows-powershell := true
export RUST_BACKTRACE := "1"
project_name := "tgt"


# Usage: just <command> <feature> <bin_name>
#
# Avaialble commands:
#   all
#   build
#   run
#   test
#   clippy
#   fmt
#   clean
#
# Available features:
#   default
#   download-tdlib
#   pkg-config
#
# Available bin_name:
#   tgt
#   example
#   telegram
#   get_me

_default:
	just --list --justfile {{justfile()}}

# Run fmt, clippy, test
all: fmt clippy test

# Build the project
build FEATURES="default" BIN_NAME="tgt":
  cargo build --verbose --features {{FEATURES}} --bin {{BIN_NAME}}

# Run the project
run FEATURES="default" BIN_NAME="tgt":
  cargo run --features {{FEATURES}} --bin {{BIN_NAME}}

# Run the tests
test FEATURES="default":
  cargo test --verbose --features {{FEATURES}} -- --nocapture --test-threads=1

# Run clippy
clippy FEATURES="default":
	cargo clippy --all-targets --features {{FEATURES}} -- -D warnings

# Run rustfmt
fmt:
  cargo fmt --all
  cargo fmt --all -- --check

# Clean the project using cargo
clean:
  cargo clean

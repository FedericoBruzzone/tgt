# vim: set ft=make :
set windows-powershell := true
export RUST_BACKTRACE := "1"
project_name := "tgt"

_default:
  just --list --justfile {{justfile()}}

# All
all: fmt clippy test build

# Build the project using cargo
build:
  cargo build

# Run the project using cargo
run BIN="" BIN_NAME="":
  cargo run {{BIN}} {{BIN_NAME}}

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

_help:
  @echo "Usage: just [recipe]"
  @echo ""
  @echo "Available recipes:"
  @echo "  build       # Build the project using cargo"
  @echo "  run         # Run the project using cargo"
  @echo "  fmt_nightly # Format the code using cargo nightly"
  @echo "  fmt         # Format the code using cargo"
  @echo "  clippy      # Run clippy using cargo"
  @echo "  test        # Run tests using cargo"
  @echo "  clean       # Clean the project using cargo"
  @echo "  help        # Display this help message"

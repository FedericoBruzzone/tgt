# vim: set ft=make :
export RUST_BACKTRACE := "1"
project_name := "tgt"

_default:
  just --list --justfile {{justfile()}}

# Build the project using cargo
build:
  cargo build

# Run the project using cargo
run BIN="" BIN_NAME="":
  cargo run {{BIN}} {{BIN_NAME}}

# Format the code using cargo nightly
fmt:
  cargo +nightly fmt -- --check

# Run tests using cargo
test:
  cargo test -- --test-threads=1

# Clean the project using cargo
clean:
  cargo clean

_help:
  @echo "Usage: just [recipe]"
  @echo ""
  @echo "Available recipes:"
  @echo "  build # Build the project using cargo"
  @echo "  run   # Run the project using cargo"
  @echo "  fmt   # Format the code using cargo nightly"
  @echo "  test  # Run tests using cargo"
  @echo "  clean # Clean the project using cargo"
  @echo "  help  # Display this help message"

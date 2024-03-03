export RUST_BACKTRACE := "1"

_default:
  just --list --justfile {{justfile()}}

# Build the project using cargo
build:
  cargo build

# Run the project using cargo
run:
  cargo run

# Format the code using cargo nightly
fmt:
  cargo +nightly fmt

# Run tests using cargo
test:
  cargo test

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

export RUST_BACKTRACE := 1

# Usage: make <command> ARGS="--features <feature> --bin <bin_name>"
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
#   local-tdlib
#   download-tdlib
#   pkg-config
#
# Available bin_name:
#   tgt
#   example
#   telegram
#   get_me

all:
	$(MAKE) fmt
	$(MAKE) clippy # ARGS="--features download-tdlib"
	$(MAKE) test # ARGS="--features download-tdlib"

run_local:
	cargo run --no-default-features --features local-tdlib

build_local:
	cargo build --no-default-features --features local-tdlib

# Example 1: make build ARGS="--features download-tdlib"
# Example 2: make build ARGS="--features download-tdlib --example telegram"
build:
	cargo build --no-default-features $(ARGS)

# Example 1: make run ARGS="--features download-tdlib"
# Example 2: make run ARGS="--features download-tdlib --example telegram"
run:
	cargo run --no-default-features $(ARGS)

test:
	cargo test --no-default-features --verbose $(ARGS) -- --nocapture --test-threads=1

clippy:
	cargo clippy --no-default-features --all-targets $(ARGS) -- -D warnings

fmt:
	cargo fmt --all
	cargo fmt --all -- --check

clean:
	cargo clean

help:
	@echo "Usage: make [target]"
	@echo ""
	@echo "Available targets:"
	@echo "  all            # Run fmt, clippy and test"
	@echo "  build          # Build the project"
	@echo "  run            # Run the project"
	@echo "  test           # Run the tests"
	@echo "  clippy         # Run clippy"
	@echo "  fmt            # Run rustfmt"
	@echo "  clean          # Clean the project"
	@echo "  help           # Display this help message"

# Each entry of .PHONY is a target that is not a file
.PHONY: build run test clean

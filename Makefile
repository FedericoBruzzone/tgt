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
#   default
#   download-tdlib
#   pkg-config
#
# Available bin_name:
#   tgt
#   example
#   telegram
#   get_me

all: fmt clippy test

build:
	cargo build --verbose $(ARGS)

run:
	cargo run $(ARGS)

test:
	cargo test --verbose $(ARGS) -- --nocapture --test-threads=1

clippy:
	cargo clippy --all-targets $(ARGS) -- -D warnings

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


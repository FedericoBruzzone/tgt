export RUST_BACKTRACE := 1


all: fmt clippy test

build_local:
	cargo build

build_download:
	cargo build --features download-tdlib

# Example: make run_local ARGS="--bin <bin_name>"
run_local:
	cargo run $(ARGS)

# Example: make run_download ARGS="--bin <bin_name>"
run_download:
	cargo run --features download-tdlib $(ARGS)

fmt:
	cargo fmt
	cargo fmt -- --check

fmt_nightly:
	cargo +nightly fmt
	cargo +nightly fmt -- --check

clippy:
	cargo clippy --all-targets --all-features -- -D warnings

test:
	cargo test -- --nocapture --test-threads=1

clean:
	cargo clean

help:
	@echo "Usage: make [target]"
	@echo ""
	@echo "Available targets:"
	@echo "  all            # Run fmt, clippy and test"
	@echo "  build_local    # Build the project using cargo; you need to have setup the LOCAL_TDLIB_PATH environment variable"
	@echo "  build_download # Build the project using cargo; it will download the tdlib library thanks to the tdlib-rs crate"
	@echo "  run_local	    # Run the project using cargo; you need to have setup the LOCAL_TDLIB_PATH environment variable"
	@echo "  run_download   # Run the project using cargo; it will download the tdlib library thanks to the tdlib-rs crate"
	@echo "  fmt            # Format the code using cargo"
	@echo "  fmt_nightly    # Format the code using nightly cargo"
	@echo "  clippy         # Run clippy using cargo"
	@echo "  test           # Run tests using cargo"
	@echo "  clean          # Clean the project using cargo"
	@echo "  help           # Display this help message"

# Each entry of .PHONY is a target that is not a file
.PHONY: build run test clean


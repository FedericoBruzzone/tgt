export RUST_BACKTRACE := 1

build:
	cargo build

# Example: make run ARGS="--bin <bin_name>"
run:
	cargo run $(ARGS)

fmt:
	cargo +nightly fmt -- --check

test:
	cargo test -- --test-threads=1

clean:
	cargo clean

help:
	@echo "Usage: make [target]"
	@echo ""
	@echo "Available targets:"
	@echo "  build # Build the project using cargo"
	@echo "  run   # Run the project using cargo"
	@echo "  fmt   # Format the code using cargo nightly"
	@echo "  test  # Run tests using cargo"
	@echo "  clean # Clean the project using cargo"
	@echo "  help  # Display this help message"

# Each entry of .PHONY is a target that is not a file
.PHONY: build run test clean


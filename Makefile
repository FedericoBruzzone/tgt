# Makefile for rust projects

build:
	cargo build

run:
	cargo run

fmt:
	cargo +nightly fmt

test:
	cargo test

clean:
	cargo clean

# Each entry of .PHONY is a target that is not a file
.PHONY: build run test clean


# Makefile for ghostsync

BINARY_NAME := ghostsync

.PHONY: all build run install clean

all: build

build:
	cargo build --release

run:
	cargo run -- $(ARGS)

install:
	cargo install --path .

clean:
	cargo clean


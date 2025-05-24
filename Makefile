build:
	cargo build

run:
	cargo run

test:
	cargo test

fmt:
	cargo fmt

grpcui:
	cargo run &
	sleep 2
	grpcui -plaintext localhost:50051

all: fmt build test
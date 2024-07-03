args := "--workspace"

[private]
@default:
	just --list

check:
	cargo c {{args}}
	cargo clippy {{args}}
	cargo fmt --check

format:
	cargo fmt --all

# build static
build:
	cargo b --release --target=x86_64-unknown-linux-musl

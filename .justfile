args := "--workspace"

[private]
@default:
	just --list

check:
	cargo c {{args}}
	cargo clippy {{args}}

format:
	cargo fmt --all

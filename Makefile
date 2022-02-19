.PHONY: configure-rust
configure-rust:
	rustup install 1.53.0
	rustup default 1.53.0
	rustup toolchain install nightly-2021-05-21
	rustup target add wasm32-unknown-unknown --toolchain nightly-2021-05-21
	rustup component add clippy

.PHONY: init
init:
	make configure-rust
	git submodule update --init --recursive

.PHONY: release
release:
	make configure-rust
	rm -rf target/
	cargo build --manifest-path node/Cargo.toml --features with-ethereum-compatibility --release

.PHONY: build
build:
	cargo build --manifest-path node/Cargo.toml --features runtime-benchmarks,with-ethereum-compatibility --release

.PHONY: wasm
wasm:
	cargo build -p setheum-runtime --features with-ethereum-compatibility --release

.PHONY: genesis
genesis:
	make release
	./target/release/setheum-node build-spec --chain testnet-new > resources/chain_spec_testnet.json
	./target/release/setheum-node build-spec --chain mainnet-new > resources/chain_spec_mainnet.json
	./target/release/setheum-node build-spec --chain testnet-new --raw > resources/chain_spec_testnet_raw.json
	./target/release/setheum-node build-spec --chain mainnet-new --raw > resources/chain_spec_mainnet_raw.json

.PHONY: check
check:
	SKIP_WASM_BUILD=1 cargo check

.PHONY: clippy
clippy:
	SKIP_WASM_BUILD=1 cargo clippy -- -D warnings -A clippy::from-over-into -A clippy::unnecessary-cast -A clippy::identity-op -A clippy::upper-case-acronyms

.PHONY: watch
watch:
	SKIP_WASM_BUILD=1 cargo watch -c -x build

.PHONY: test
test:
	SKIP_WASM_BUILD=1 cargo test --all

.PHONY: debug
debug:
	cargo build && RUST_LOG=debug RUST_BACKTRACE=1 rust-gdb --args target/debug/setheum-node --dev --tmp -lruntime=debug

.PHONY: run
run:
	RUST_BACKTRACE=1 cargo run --manifest-path node/Cargo.toml --features with-ethereum-compatibility  -- --dev --tmp

.PHONY: log
log:
	RUST_BACKTRACE=1 RUST_LOG=debug cargo run --manifest-path node/Cargo.toml --features with-ethereum-compatibility  -- --dev --tmp

.PHONY: noeth
noeth:
	RUST_BACKTRACE=1 cargo run -- --dev --tmp

.PHONY: bench
bench:
	SKIP_WASM_BUILD=1 cargo test --manifest-path node/Cargo.toml --features runtime-benchmarks,with-ethereum-compatibility benchmarking

.PHONY: doc
doc:
	SKIP_WASM_BUILD=1 cargo doc --open

.PHONY: cargo-update
cargo-update:
	cargo update
	cargo update --manifest-path node/Cargo.toml
	make test

.PHONY: fork
fork:
	npm i --prefix fork fork
ifeq (,$(wildcard fork/data))
	mkdir fork/data
endif
	cp target/release/setheum-node fork/data/binary
	cp target/release/wbuild/setheum-runtime/setheum_runtime.compact.wasm fork/data/runtime.wasm
	cp resources/types.json fork/data/schema.json
	cp resources/chain_spec_$(chain)_raw.json fork/data/genesis.json
	cd fork && npm start && cd ..

.PHONY: generate-tokens
generate-tokens:
	./scripts/generate-tokens-and-predeploy-contracts.sh

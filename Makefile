.PHONY: toolchain
toolchain:
	./scripts/init.sh

.PHONY: init
init:
	make toolchain
	git submodule update --init --recursive

.PHONY: release
release:
	make toolchain
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
	make build
	./target/release/setheum-node build-spec --chain testnet-new > resources/chain_spec_testnet.json
	./target/release/setheum-node build-spec --chain mainnet-new > resources/chain_spec_mainnet.json
	./target/release/setheum-node build-spec --chain testnet-new --raw > resources/chain_spec_testnet_raw.json
	./target/release/setheum-node build-spec --chain mainnet-new --raw > resources/chain_spec_mainnet_raw.json

.PHONY: check
check:
	SKIP_WASM_BUILD=1 cargo check

.PHONY: check-all
check-all: check-runtime check-benchmarks

.PHONY: check-runtime
check-runtime:
	SKIP_WASM_BUILD= cargo check --features with-ethereum-compatibility --tests --all

.PHONY: clippy
clippy:
	SKIP_WASM_BUILD=1 cargo clippy -- -D warnings -A clippy::from-over-into -A clippy::unnecessary-cast -A clippy::identity-op -A clippy::upper-case-acronyms

.PHONY: watch
watch:
	SKIP_WASM_BUILD=1 cargo watch -c -x build

.PHONY: test
test:
	SKIP_WASM_BUILD=1 cargo test --features with-ethereum-compatibility --all

.PHONY: check-tests
check-tests:
	SKIP_WASM_BUILD= cargo check --features with-ethereum-compatibility --tests --all

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

.PHONY: check-benchmarks
check-benchmarks:
	SKIP_WASM_BUILD= cargo check --features runtime-benchmarks --no-default-features --target=wasm32-unknown-unknown -p setheum-runtime

.PHONY: test-benchmarking
test-benchmarking:
	cargo test --features bench --package module-evm
	cargo test --features runtime-benchmarks --features with-ethereum-compatibility --features --all benchmarking

.PHONY: bench
bench:
	SKIP_WASM_BUILD=1 cargo test --manifest-path node/Cargo.toml --features runtime-benchmarks,with-ethereum-compatibility benchmarking

.PHONY: benchmark
benchmark:
	 cargo run --release --features=runtime-benchmarks --features=with-ethereum-compatibility -- benchmark --chain=dev --steps=50 --repeat=20 '--pallet=*' '--extrinsic=*' --execution=wasm --wasm-execution=compiled --heap-pages=4096 --template=./templates/runtime-weight-template.hbs --output=./runtime/src/weights/

.PHONY: doc
doc:
	SKIP_WASM_BUILD=1 cargo doc --open

.PHONY: cargo-update
cargo-update:
	cargo update
	cargo update --manifest-path node/Cargo.toml
	make test

.PHONY: purge
purge: target/debug/setheum-node
	target/debug/setheum-node purge-chain --dev -y

.PHONY: restart
restart: purge run

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
	cargo test -p setheum-primitives -- --ignored
	cd lib-serml/sevm/predeploy-contracts && yarn && yarn run generate-bytecode

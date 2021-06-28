.PHONY: run
run: githooks
	cargo run --manifest-path node/setheum-dev/Cargo.toml -- --dev -lruntime=debug --instant-sealing

.PHONY: run-sevm
run: githooks
	cargo run --manifest-path node/setheum-dev/Cargo.toml --features with-sevm -- --dev -lruntime=debug -levm=debug --instant-sealing

.PHONY: toolchain
toolchain:
	./scripts/init.sh

.PHONY: build-full
build-full: githooks
	cargo build

.PHONY: build-all
build-all: build-dev build-setheum

.PHONY: build-dev
build-dev:
	cargo build --manifest-path node/setheum-dev/Cargo.toml --locked

.PHONY: build-setheum
build-setheum:
	cargo build --manifest-path node/setheum/Cargo.toml --locked --features with-all-runtime

.PHONY: check
check: githooks
	SKIP_WASM_BUILD= cargo check

.PHONY: check-tests
check-tests: githooks
	SKIP_WASM_BUILD= cargo check --tests --all

.PHONY: check-all
check-all: check-dev check-setheum

.PHONY: check-runtimes
check-runtimes:
	SKIP_WASM_BUILD= cargo check --tests --all --features with-all-runtime

.PHONY: check-dev
check-dev:
	SKIP_WASM_BUILD= cargo check --manifest-path node/setheum-dev/Cargo.toml --tests --all

.PHONY: check-setheum
check-setheum:
	SKIP_WASM_BUILD= cargo check --manifest-path node/setheum/Cargo.toml --tests --all --features with-all-runtime

.PHONY: check-debug
check-debug:
	RUSTFLAGS="-Z macro-backtrace" SKIP_WASM_BUILD= cargo +nightly check

.PHONY: check-try-runtime
check-try-runtime:
	SKIP_WASM_BUILD= cargo check --features try-runtime --features with-all-runtime

.PHONY: test
test: githooks
	SKIP_WASM_BUILD= cargo test --all

.PHONY: test-sevm
test: githooks
	SKIP_WASM_BUILD= cargo test --all --features with-sevm test_setheum_evm
	SKIP_WASM_BUILD= cargo test --all --features with-sevm should_not_kill_contract_on_transfer_all
	SKIP_WASM_BUILD= cargo test --all --features with-sevm schedule_call_precompile_should_work
	SKIP_WASM_BUILD= cargo test --all --features with-sevm schedule_call_precompile_should_handle_invalid_input

.PHONY: test-all
test-all: test-dev test-sevm test-setheum test-benchmarking

.PHONY: test-dev
test-dev:
	SKIP_WASM_BUILD= cargo test --manifest-path node/setheum-dev/Cargo.toml --all

.PHONY: test-setheum
test-setheum:
	SKIP_WASM_BUILD= cargo test --manifest-path node/setheum/Cargo.toml --all --features with-all-runtime

.PHONY: check-benchmarking
test-benchmarking:
	SKIP_WASM_BUILD= cargo check --manifest-path node/setheum-dev/Cargo.toml --features runtime-benchmarks --no-default-features --target=wasm32-unknown-unknown -p newrome-runtime

.PHONY: test-benchmarking
test-benchmarking:
	SKIP_WASM_BUILD= cargo test --manifest-path node/setheum-dev/Cargo.toml --features runtime-benchmarks --no-default-features --target=wasm32-unknown-unknown -p newrome-runtime
	SKIP_WASM_BUILD= cargo test --manifest-path node/setheum/Cargo.toml --features runtime-benchmarks --no-default-features --target=wasm32-unknown-unknown -p neom-runtime


.PHONY: build
build: githooks
	SKIP_WASM_BUILD= cargo build

.PHONY: purge
purge: target/debug/setheum-dev
	target/debug/setheum-dev purge-chain --dev -y

.PHONY: restart
restart: purge run

target/debug/setheum-dev:
	SKIP_WASM_BUILD= cargo build

GITHOOKS_SRC = $(wildcard githooks/*)
GITHOOKS_DEST = $(patsubst githooks/%, .git/hooks/%, $(GITHOOKS_SRC))

.git/hooks:
	mkdir .git/hooks

.git/hooks/%: githooks/%
	cp $^ $@

.PHONY: githooks
githooks: .git/hooks $(GITHOOKS_DEST)

.PHONY: init
init: toolchain submodule build-full

.PHONY: submodule
submodule:
	git submodule update --init --recursive

.PHONY: update-orml
update-orml:
	cd lib-openrml && git checkout master && git pull
	git add lib-openrml

.PHONY: update
update: update-orml cargo-update check-all

.PHONY: cargo-update
cargo-update:
	cargo update

.PHONY: build-wasm-newrome
build-wasm-newrome:
	./scripts/build-only-wasm.sh -p newrome-runtime --features=with-sevm

.PHONY: build-wasm-neom
build-wasm-newrome:
	./scripts/build-only-wasm.sh -p neom-runtime --features=on-chain-release-build

.PHONY: srtool-build-wasm-neom
srtool-build-wasm-neom:
	PACKAGE=neom-runtime BUILD_OPTS="--features on-chain-release-build" ./scripts/srtool-build.sh

.PHONY: generate-tokens
generate-tokens:
	./scripts/generate-tokens-and-predeploy-contracts.sh

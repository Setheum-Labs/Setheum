.PHONY: run
run:
	cargo run --features with-newrome-runtime -- --dev -lruntime=debug --instant-sealing

.PHONY: run-sevm
run-sevm:
	cargo run --features with-newrome-runtime --features with-sevm -- --dev -lruntime=debug -levm=debug --instant-sealing

.PHONY: run-neom
run-neom:
	cargo run --features with-neom-runtime -- --chain=neom

.PHONY: toolchain
toolchain:
	./scripts/init.sh

.PHONY: build
build: githooks
	SKIP_WASM_BUILD= cargo build --features with-newrome-runtime

.PHONY: build-full
build-full: githooks
	cargo build --features with-newrome-runtime

.PHONY: build-all
build-all:
	cargo build --locked --features with-all-runtime

.PHONY: check
check: githooks
	SKIP_WASM_BUILD= cargo check --features with-newrome-runtime

.PHONY: check
check-neom: githooks
	SKIP_WASM_BUILD= cargo check --features with-neom-runtime

.PHONY: check-tests
check-tests: githooks
	SKIP_WASM_BUILD= cargo check --features with-all-runtime --tests --all

.PHONY: check-all
check-all: check-runtimes check-benchmarks

.PHONY: check-runtimes
check-runtimes:
	SKIP_WASM_BUILD= cargo check --features with-all-runtime --tests --all

.PHONY: check-benchmarks
check-benchmarks:
	SKIP_WASM_BUILD= cargo check --features runtime-benchmarks --no-default-features --target=wasm32-unknown-unknown -p newrome-runtime
	SKIP_WASM_BUILD= cargo check --features runtime-benchmarks --no-default-features --target=wasm32-unknown-unknown -p neom-runtime

.PHONY: check-debug
check-debug:
	RUSTFLAGS="-Z macro-backtrace" SKIP_WASM_BUILD= cargo +nightly check --features with-newrome-runtime

.PHONY: check-try-runtime
check-try-runtime:
	SKIP_WASM_BUILD= cargo check --features try-runtime --features with-all-runtime

.PHONY: test
test: githooks
	SKIP_WASM_BUILD= cargo test --features with-newrome-runtime --all

.PHONY: test-sevm
test: githooks
	SKIP_WASM_BUILD= cargo test --all --features with-sevm test_setheum_evm
	SKIP_WASM_BUILD= cargo test --all --features with-sevm should_not_kill_contract_on_transfer_all
	SKIP_WASM_BUILD= cargo test --all --features with-sevm schedule_call_precompile_should_work
	SKIP_WASM_BUILD= cargo test --all --features with-sevm schedule_call_precompile_should_handle_invalid_input

.PHONY: test-runtimes
test-runtimes:
	SKIP_WASM_BUILD= cargo test --all --features with-all-runtime

.PHONY: test-benchmarking
test-benchmarking:
	cargo test --features runtime-benchmarks --features with-all-runtime --features --all benchmarking

.PHONY: test-all
test-all: test-runtimes test-sevm test-benchmarking

.PHONY: purge
purge: target/debug/setheum
	target/debug/setheum purge-chain --dev -y

.PHONY: restart
restart: purge run

target/debug/setheum:
	SKIP_WASM_BUILD= cargo build --features with-newrome-runtime

.PHONY: build-newrome
build: githooks
	SKIP_WASM_BUILD= cargo build --features with-newrome-runtime

.PHONY: build-neom
build: githooks
	SKIP_WASM_BUILD= cargo build --features with-neom-runtime

.PHONY: build-setheum
build: githooks
	SKIP_WASM_BUILD= cargo build --features with-setheum-runtime

.PHONY: build-all
build: githooks
	SKIP_WASM_BUILD= cargo build --features with-all-runtime

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

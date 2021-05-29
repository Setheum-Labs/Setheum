.PHONY: run
run: githooks
	cargo run -- --dev -lruntime=debug --instant-sealing

.PHONY: toolchain
toolchain:
	./scripts/init.sh

.PHONY: build-full
build-full: githooks
	cargo build

.PHONY: build-setheum
build-setheum:
	cargo build --locked --features with-all-runtime

.PHONY: check
check: githooks
	SKIP_WASM_BUILD= cargo check

.PHONY: check-tests
check-tests: githooks
	SKIP_WASM_BUILD= cargo check --tests --all

.PHONY: check-all
check-all: check-setheum check-benchmarks

.PHONY: check-setheum
check-setheum:
	SKIP_WASM_BUILD= cargo check  --tests --all --features with-all-runtime

.PHONY: check-benchmarks
check-benchmarks:
	SKIP_WASM_BUILD= cargo check --tests --all --features with-all-runtime --features runtime-benchmarks


.PHONY: check-debug
check-debug:
	RUSTFLAGS="-Z macro-backtrace" SKIP_WASM_BUILD= cargo +nightly check

.PHONY: test
test: githooks
	SKIP_WASM_BUILD= cargo test --all

.PHONY: test-all
test-all: test-setheum test-benchmarking

.PHONY: test-setheum
test-setheum:
	SKIP_WASM_BUILD= cargo test  --all --features with-all-runtime

.PHONY: test-benchmarking
test-benchmarking:
	SKIP_WASM_BUILD= cargo test --features runtime-benchmarks --features with-all-runtime --features --all benchmarking

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
	cd orml && git checkout master && git pull
	git add orml

.PHONY: update
update: update-orml cargo-update check-all

.PHONY: cargo-update
cargo-update:
	cargo update

.PHONY: build-wasm-newrome
build-wasm-newrome:
	./scripts/build-only-wasm.sh newrome-runtime

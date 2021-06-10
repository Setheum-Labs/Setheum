.PHONY: run
run: githooks
	cargo run --manifest-path node/setheum-dev/Cargo.toml -- --dev -lruntime=debug --instant-sealing

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

.PHONY: test-all
test-all: test-dev test-setheum

.PHONY: test-dev
test-dev:
	SKIP_WASM_BUILD= cargo test --manifest-path node/setheum-dev/Cargo.toml --all

.PHONY: test-setheum
test-setheum:
	SKIP_WASM_BUILD= cargo test --manifest-path node/setheum/Cargo.toml --all --features with-all-runtime

.PHONY: test-benchmarking
test-benchmarking:
	SKIP_WASM_BUILD= cargo test --manifest-path node/setheum-dev/Cargo.toml --features runtime-benchmarks -p newrome-runtime benchmarking

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
update: update-orml
	cargo update
	make check

.PHONY: build-wasm-newrome
build-wasm-newrome:
	./scripts/build-only-wasm.sh newrome-runtime

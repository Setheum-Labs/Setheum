.PHONY: init
init:
	./scripts/init.sh

.PHONY: check
check:
	WASM_BUILD_TOOLCHAIN=nightly-2020-10-05 cargo check

.PHONY: test
test:
	WASM_BUILD_TOOLCHAIN=nightly-2020-10-05 cargo test --all

.PHONY: run
run:
	WASM_BUILD_TOOLCHAIN=nightly-2020-10-05 cargo run --release -- --dev --tmp

.PHONY: build
build:
	WASM_BUILD_TOOLCHAIN=nightly-2020-10-05 cargo build --release

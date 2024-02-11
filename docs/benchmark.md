# Benchmarking docs

## Generate Module weights

### Pallet Module weights

```bash
    cargo run --release --features=runtime-benchmarks \
    --features=with-ethereum-compatibility \
    -- benchmark \
    --chain=dev \
    --steps=50 \
    --repeat=20 \
    '--pallet={module_name}' \
    '--extrinsic=*' \
    --execution=wasm \
    --wasm-execution=compiled \
    --heap-pages=4096 \
    --template=.maintain/runtime-weight-template.hbs \
    --output=./modules/{dir/module-inner-directory}/src/weights/
```

for example, this is the command for generating the `airdrop` module weights:

```bash
    cargo run --release --features=runtime-benchmarks \
    --features=with-ethereum-compatibility \
    -- benchmark \
    --chain=dev \
    --steps=50 \
    --repeat=20 \
    '--pallet=airdrop' \
    '--extrinsic=*' \
    --execution=wasm \
    --wasm-execution=compiled \
    --heap-pages=4096 \
    --template=.maintain/runtime-weight-template.hbs \
    --output=./modules/airdrop/src/weights/
```

### Runtime Module weights

```bash
   make benchmark
```

Or for a specific module:

```bash
    cargo run --release --features=runtime-benchmarks \
    --features=with-ethereum-compatibility \
    -- benchmark \
    --chain=dev \
    --steps=50 \
    --repeat=20 \
    '--pallet=airdrop' \
    '--extrinsic=*' \
    --execution=wasm \
    --wasm-execution=compiled \
    --heap-pages=4096 \
    --template=.maintain/runtime-weight-template.hbs \
    --output=./runtime/src/weights/
```

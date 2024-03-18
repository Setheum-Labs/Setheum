# API for [Setheum](https://github.com/Setheum-Labs/Setheum) chain.

This crate provides a Rust application interface for submitting transactions to `setheum-node` chain.
Most of the [pallets](https://docs.substrate.io/reference/frame-pallets/) are common to any
[Substrate](https://github.com/paritytech/substrate) chain, but there are some unique to `setheum-node`,
e.g. [`pallets::elections::ElectionsApi`](./src/pallets/elections.rs).

## Build

Just use `cargo build` or `cargo build --release`, depends on your usecase.

## Contributions

All contributions are welcome, e.g. adding new API for pallets in `setheum-node`. 

## Metadata

`setheum-client` uses [`subxt`](https://github.com/paritytech/subxt) to communicate with a Substrate-based chain which
`setheum` is. In order to provide a strong type safety, it uses a manually generated file [`setheum.rs`](src/setheum.rs)
which refers to top of the `main` branch in the `setheum` repository. See more info [here](docker/README.md).

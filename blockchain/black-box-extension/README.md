# BlackBox chain extension

## Overview

This crate is an implementation of BlackBox chain extension, with both ink! and Substrate counterparts available.

## ink!

### Usage

To use `black-box-extension` with ink!, include it as a dependency and activate `ink-std` 
feature when `std` feature of your contract is enabled:

```toml
black-box-extension = { path = "...", default-features = false, features = ["ink"] }

# ...

[features]
# ...
std = [
    # ...
    "black-box-extension/ink-std"
]
```

Next, simply call `BlackBoxExtension` methods on `black-box_extension::ink::Extension`:

```rust
use black-box_extension::{ink::Extension, BlackBoxExtension};

Extension.store_key(...);
```

### Testing

To test chain extension with `ink` features enabled, you have to ensure that you removed any other mention of `black-box-extension`
with `substrate` feature enabled, otherwise `rustc` will emit errors related to duplicated items.

For example, you can comment out `black-box-extension` mentions from `runtime` crate, then try to run
the necessary checks/tests in `black-box-extension` directory.

## Substrate

### Usage

To use `black-box-extension` with Substrate, add `black-box_extension::substrate::Extension` to `pallet_contracts::Config`'s `ChainExtension` associated type:

```rust
impl pallet_contracts::Config for Runtime {
    // ...
    type ChainExtension = black-box_extension::substrate::Extension;
}
```

### Implementation details

`black-box-extension` introduces several types for you to use during the
chain extension development/usage.

#### `BlackBoxExtension`

The trait, thanks to being marked with `#[obce::definition]`, provides a description
of what chain extension does, as well as contains an automatically generated `obce::codegen::ExtensionDescription`
and `obce::codegen::MethodDescription` trait impls.

#### `BlackBoxError`

`BlackBoxError` is a type that describes all errors that can occur during chain extension calls.

Using `#[obce::error]` attribute, accompanied by `ret_val` variant attributes,
`BlackBoxError` has an automatically implemented common traits (`Copy`, `Clone`, `scale::Encode`, `scale::Decode`, etc.) for the type itself, and an implementation of `TryFrom<BlackBoxError>` for `pallet_contracts::chain_extension::RetVal`. This implementation allows us to automatically convert `BlackBoxError` to `RetVal` if `#[obce::implementation]` methods have `ret_val` attribute on them.

#### `substrate` module

The `substrate` module contains the chain extension implementation itself.

Every method is marked with with `#[obce(weight(expr = "...", pre_charge = true), ret_val)]`,
meaning that they:

1. Pre-charge weight calculated by the provided expression.
2. Return a `Result<T, E>`, that has to be converted to `pallet_contracts::chain_extension::RetVal` if possible (in our case, since `BlackBoxError` has all variants attributed with `#[obce(ret_val = "...")]` we simply convert every error instance to `RetVal`).

An additional `Env: Executor<T>` bound exists on `impl` block to mock pallet calls
with `Executor` trait available at `executor` module.

#### Testing

For mocking chain extension environment there exists a `MockedEnvironment` struct, which
implements both `obce::substrate::ChainExtensionEnvironment` and `Executor` traits,
which change their behaviors depending on the passed const generics values.

Various utility types (like `StoreKeyOkayer` or `VerifyErrorer`) exist to simplify
chain extension testing even further by providing a simple const generics-based interface to
configure testing environment.

Method identifier constants (`STORE_KEY_ID` and `VERIFY_ID`) were acquired by expanding
macros using `cargo expand`, and are depending solely on the method names (thus making them stable between compilations).

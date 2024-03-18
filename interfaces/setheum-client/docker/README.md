This directory contains following files:
### `subxt-integration.Dockerfile`
This is not a main `setheum-client`, rather it is a helper Dockerfile to run on GH, which has `subxt` tool.

It requires:
* a `setheum` chain to be run in the background (ie `127.0.0.1:9944` port must be opened),
* access to `rustfmt.toml`,
* access to current `setheum.rs` file

The docker checks whether a `subxt`-generated runtime metadata is the same as from the current commit. 

It needs to be run only from `setheum-client` directory and in network host mode:
```bash
 docker run --network host --mount type=bind,source="$(pwd)/..",target=/subxt/setheum subxt:latest
```

### `subxt-integration-entrypoint.sh` 
An entrypoint for above Dockerfile

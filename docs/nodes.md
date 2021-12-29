Generate Node Key

```
./target/release/setheum-node key generate-node-key
```


Run Bootnode

```
./target/release/setheum-node \
  --chain testnet \
  --base-path ./setheum/bootnode \
  --port 30333 \
  --node-key <[node-key]> \
  --rpc-cors all \
  --name TestnetBootnode
```


Run Full Node (RPC Node)

```
./target/release/setheum-node \
  --chain testnet \
  --base-path ./setheum/fullnode \
  --pruning=archive \
  --port 30334 \
  --ws-port 9944 \
  --rpc-port 9933 \
  --rpc-methods Auto \
  --rpc-cors all \
  --rpc-external \
  --ws-external \
  --name TestnetRPCNode
```


Insert Session Keys (Babe)

```
// Insert babe session keys
./target/release/setheum-node key insert --base-path /tmp/node01 \
--chain testnet \
--suri <[babe-key]> \
--password-interactive \
--key-type babe
```


Insert Session Keys (Grandpa)

```
// Insert grandpa session keys
./target/release/setheum-node key insert --base-path /tmp/node01 \
--chain testnet \
--suri <[grandpa-key]> \
--password-interactive \
--key-type gran
```

Verify keys are in keystore

```
ls /tmp/node01/chains/setheum_testnet/keystore
```


Run Validator Node

```
./target/release/setheum-node \
  --validator \
  --chain testnet \
  --base-path ./setheum/validator \
  --execution=wasm \
  --port 30335 \
  --ws-port 9945 \
  --rpc-port 9934 \
  --rpc-methods Unsafe \
  --no-mdns \
  --no-private-ipv4 \
  --no-prometheus \
  --no-telemetry \
  --name TestnetValidatorNode1
```

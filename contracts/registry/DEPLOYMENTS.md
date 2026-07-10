# Registry Contract — Deployment Log

## Testnet

| Field              | Value                                                              |
|--------------------|--------------------------------------------------------------------|
| Network            | Stellar Testnet (`Test SDF Network ; September 2015`)              |
| Contract ID        | _pending — see note below_                                         |
| Deployer address   | _pending — see note below_                                         |
| Deployment date    | _pending — see note below_                                         |
| WASM artifact      | `contracts/registry/target/wasm32-unknown-unknown/release/contracts_registry.wasm` |

### Deployment note

The `stellar-cli` binary could not be installed in the CI environment used
for this issue because the system dependency `libdbus-1-dev` was not present
(`cargo install --locked stellar-cli` exits with a `libdbus-sys` build
failure).  All local validation checks pass:

```
cargo test -p contracts-registry                                   ✅ 37 tests pass
cargo test -p contracts-registry --test integration_test           ✅ 2 tests pass
cargo build --target wasm32-unknown-unknown --release -p contracts-registry  ✅
cargo clippy --target wasm32-unknown-unknown -p contracts-registry -- -D warnings  ✅
cargo fmt -p contracts-registry -- --check                         ✅
```

To complete the deployment, run the following on a machine where
`stellar-cli` is available (install via `cargo install --locked stellar-cli`):

```bash
# 1. Generate a dedicated testnet deployer identity (one-time)
stellar keys generate cascade-deployer --network testnet --fund

# 2. Record the deployer's public key (add to this file — never commit the secret)
stellar keys address cascade-deployer

# 3. Configure the testnet network (one-time)
stellar network add testnet \
  --rpc-url https://soroban-testnet.stellar.org:443 \
  --network-passphrase "Test SDF Network ; September 2015"

# 4. Build the optimized WASM (if not already built)
cd contracts
cargo build --target wasm32-unknown-unknown --release -p contracts-registry

# 5. Deploy
stellar contract deploy \
  --wasm contracts/registry/target/wasm32-unknown-unknown/release/contracts_registry.wasm \
  --source cascade-deployer \
  --network testnet
```

The command prints a contract ID on success.  Update the table above with:
- the contract ID (the `C…` address printed by `stellar contract deploy`)
- the deployer's public key (`stellar keys address cascade-deployer`)
- the deployment date (ISO 8601, e.g. `2026-07-10`)

**Do not commit any secret key material.**  Only the public key (`G…`) belongs
in this file.

---

## Mainnet

Not applicable — mainnet deployment is explicitly out of scope for the
flat-registry MVP phase and requires its own dedicated review process.

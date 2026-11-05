# contracts/

`alpha-keys/` is the NEAR contract: trader registry, Viewing-Key NFT (NEP-171/177/178/181/199), and PnL attestation records. The interface is in [../docs/ARCHITECTURE.md §2](../docs/ARCHITECTURE.md). The tasks are C-1 to C-6 and A-6.

```bash
cd contracts/alpha-keys
cargo test
cargo near build non-reproducible-wasm
near deploy <account.testnet> ../../target/near/alpha_keys.wasm
```

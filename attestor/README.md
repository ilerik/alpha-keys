# attestor/

- `pnl-core/`: a pure Rust library that turns fills and account state into metrics (PnL, max drawdown, Sharpe). It is deterministic, uses fixed-point output, and has no I/O. Tasks A-1 to A-3.
- `outlayer-job/`: the WASI (`wasm32-wasip2`) binary that runs inside OutLayer's Intel TDX workers. It reads JSON input on stdin and the sealed `HL_ADDRESS` from env (an OutLayer secret), posts to Hyperliquid with `wasi-http-client`, and writes metrics JSON to stdout. The native build keeps a fetch that refuses the network, so `cargo test` stays offline. Actions: `pnl` and `feed`. Tasks A-4 to A-7 and F-2.

```bash
cargo test -p pnl-core
cargo build -p outlayer-job --target wasm32-wasip2 --release
sha256sum target/wasm32-wasip2/release/outlayer-job.wasm
# outlayer upload … && outlayer deploy …   (see scripts/outlayer-publish.sh, task A-5)
```
Docs: https://app.outlayer.ai/docs/getting-started · https://app.outlayer.ai/llms.txt

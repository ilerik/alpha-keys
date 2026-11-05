//! OutLayer WASI job (STUB). Build: cargo build -p outlayer-job --target wasm32-wasip2 --release
//! Contract: stdin = {"action":"pnl"|"feed", ...}; env HL_ADDRESS (OutLayer secret); stdout = JSON.
//! NEVER print HL_ADDRESS or raw fills. See docs/ARCHITECTURE.md §3, tasks A-4, F-2.
use std::io::Read;

fn main() {
    let mut input = String::new();
    let _ = std::io::stdin().read_to_string(&mut input);
    // TODO(A-4): parse input, fetch HL /info over HTTPS, call pnl_core, emit metrics JSON.
    println!("{{\"status\":\"stub\",\"max_dd_bps\":{}}}", pnl_core::max_drawdown_bps(&[]));
}

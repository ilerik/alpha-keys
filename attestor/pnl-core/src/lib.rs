//! pnl-core (STUB): pure metrics over Hyperliquid fills. See docs/TASKS.md A-1..A-3.
//! All outputs are fixed-point integers so TEE output is deterministic.

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Metrics {
    pub pnl_usd_e6: i64,
    pub max_dd_bps: u32,
    pub sharpe_e4: i32,
    pub n_trades: u32,
    pub equity_usd_e6: u64,
}

/// Max drawdown in basis points over an equity curve (USD e6). Implemented as an example of the style.
pub fn max_drawdown_bps(equity: &[u64]) -> u32 {
    let mut peak = 0u64;
    let mut worst = 0u64;
    for &e in equity {
        peak = peak.max(e);
        if peak > 0 {
            worst = worst.max((peak - e) * 10_000 / peak);
        }
    }
    worst as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn drawdown_basic() {
        assert_eq!(max_drawdown_bps(&[100, 120, 90, 130]), 2500);
        assert_eq!(max_drawdown_bps(&[]), 0);
    }
}

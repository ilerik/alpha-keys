//! Pure Hyperliquid fill metrics. Outputs are fixed-point integers.
//! USD values use 6 decimal places (`e6`). Position sizes use 8 (`e8`).
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const DAY_MS: u64 = 86_400_000;
/// floor(sqrt(365) * 1e8)
const SQRT_365_E8: i128 = 1_910_497_317;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Side {
    #[serde(rename = "B")]
    Buy,
    #[serde(rename = "A")]
    Sell,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fill {
    pub coin: String,
    pub px_e6: i64,
    pub px_display: String,
    pub sz_e8: i64,
    pub side: Side,
    pub time_ms: u64,
    pub closed_pnl_e6: i64,
    pub fee_e6: i64,
    pub start_position_e8: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    pub coin: String,
    #[serde(rename = "szi")]
    pub szi_e8: i64,
    #[serde(rename = "entryPx")]
    pub entry_px_e6: i64,
    #[serde(rename = "unrealizedPnl")]
    pub unrealized_pnl_e6: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct AccountState {
    #[serde(rename = "accountValue")]
    pub account_value_e6: i64,
    pub positions: Vec<Position>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Metrics {
    pub pnl_usd_e6: i64,
    pub max_dd_bps: u32,
    pub sharpe_e4: i32,
    pub n_trades: u32,
    pub equity_usd_e6: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct FillWire {
    coin: String,
    px: String,
    sz: String,
    side: String,
    time: u64,
    #[serde(rename = "closedPnl")]
    closed_pnl: String,
    fee: String,
    #[serde(rename = "startPosition")]
    start_position: String,
}

/// Max drawdown in basis points. Empty or flat curves return 0. The result is at most 10_000.
pub fn max_drawdown_bps(equity: &[u64]) -> u32 {
    let mut peak = 0u64;
    let mut worst = 0u128;
    for &equity_point in equity {
        peak = peak.max(equity_point);
        if peak > 0 && equity_point < peak {
            let drop = (peak - equity_point) as u128 * 10_000 / peak as u128;
            worst = worst.max(drop);
        }
    }
    worst.min(10_000) as u32
}

pub fn compute(fills: &[Fill], state: &AccountState) -> Metrics {
    let mut ordered = fills.to_vec();
    ordered.sort_by_key(|fill| (fill.time_ms, fill.coin.clone()));
    let realized: i64 = ordered.iter().map(net_pnl).sum();
    let unrealized: i64 = state.positions.iter().map(|position| position.unrealized_pnl_e6).sum();
    let pnl = realized.saturating_add(unrealized);
    let base = state.account_value_e6.saturating_sub(unrealized).saturating_sub(realized);
    let mut points: Vec<(u64, i64)> = Vec::new();
    let start = ordered.first().map(|fill| fill.time_ms.saturating_sub(DAY_MS)).unwrap_or(0);
    points.push((start, base));
    let mut running = 0i64;
    for fill in &ordered {
        running = running.saturating_add(net_pnl(fill));
        points.push((fill.time_ms, base.saturating_add(running)));
    }
    if points.last().map(|point| point.1) != Some(state.account_value_e6) {
        let end_time = points.last().map(|point| point.0.saturating_add(1)).unwrap_or(0);
        points.push((end_time, state.account_value_e6));
    }
    let curve: Vec<u64> = points.iter().map(|(_, equity)| nonneg(*equity)).collect();
    let daily = end_of_day(&points);
    Metrics {
        pnl_usd_e6: pnl,
        max_dd_bps: max_drawdown_bps(&curve),
        sharpe_e4: sharpe_e4(&daily),
        n_trades: ordered.len() as u32,
        equity_usd_e6: nonneg(state.account_value_e6),
    }
}

pub fn fills_from_hl_json(bytes: &[u8]) -> Result<Vec<Fill>, String> {
    let value: Value = serde_json::from_slice(bytes).map_err(|err| err.to_string())?;
    let rows = value.as_array().ok_or("fills must be an array")?;
    rows.iter().map(fill_from_value).collect()
}

pub fn account_from_hl_json(bytes: &[u8]) -> Result<AccountState, String> {
    let value: Value = serde_json::from_slice(bytes).map_err(|err| err.to_string())?;
    let account_value = value
        .pointer("/marginSummary/accountValue")
        .ok_or("missing marginSummary.accountValue")?;
    let mut positions = Vec::new();
    if let Some(rows) = value.get("assetPositions").and_then(Value::as_array) {
        for row in rows {
            let position = row.get("position").unwrap_or(row);
            positions.push(Position {
                coin: string_field(position, "coin")?,
                szi_e8: parse_fixed(&string_field(position, "szi")?, 8)?,
                entry_px_e6: parse_fixed(&string_field(position, "entryPx").unwrap_or_else(|_| "0".into()), 6)?,
                unrealized_pnl_e6: parse_fixed(
                    &string_field(position, "unrealizedPnl").unwrap_or_else(|_| "0".into()),
                    6,
                )?,
            });
        }
    }
    Ok(AccountState { account_value_e6: parse_fixed(&as_decimal(account_value)?, 6)?, positions })
}

pub fn fills_to_hl_json(fills: &[Fill]) -> String {
    let wires: Vec<FillWire> = fills.iter().map(Fill::to_wire).collect();
    serde_json::to_string(&wires).expect("fills serialize")
}

impl Fill {
    fn to_wire(&self) -> FillWire {
        FillWire {
            coin: self.coin.clone(),
            px: self.px_display.clone(),
            sz: format_fixed(self.sz_e8, 8),
            side: match self.side {
                Side::Buy => "B".into(),
                Side::Sell => "A".into(),
            },
            time: self.time_ms,
            closed_pnl: format_fixed(self.closed_pnl_e6, 6),
            fee: format_fixed(self.fee_e6, 6),
            start_position: format_fixed(self.start_position_e8, 8),
        }
    }
}

fn fill_from_value(value: &Value) -> Result<Fill, String> {
    let side = match string_field(value, "side")?.as_str() {
        "B" | "Buy" => Side::Buy,
        "A" | "Sell" => Side::Sell,
        other => return Err(format!("bad side {other}")),
    };
    let px = string_field(value, "px")?;
    Ok(Fill {
        coin: string_field(value, "coin")?,
        px_e6: parse_fixed(&px, 6)?,
        px_display: trim_decimal(&px),
        sz_e8: parse_fixed(&string_field(value, "sz")?, 8)?,
        side,
        time_ms: value.get("time").and_then(Value::as_u64).ok_or("missing time")?,
        closed_pnl_e6: parse_fixed(&string_field(value, "closedPnl")?, 6)?,
        fee_e6: parse_fixed(&string_field(value, "fee").unwrap_or_else(|_| "0".into()), 6)?,
        start_position_e8: parse_fixed(&string_field(value, "startPosition").unwrap_or_else(|_| "0".into()), 8)?,
    })
}

fn net_pnl(fill: &Fill) -> i64 {
    fill.closed_pnl_e6.saturating_sub(fill.fee_e6)
}

fn nonneg(value: i64) -> u64 {
    if value > 0 { value as u64 } else { 0 }
}

fn end_of_day(points: &[(u64, i64)]) -> Vec<u64> {
    let mut daily: Vec<(u64, i64)> = Vec::new();
    for (time, equity) in points {
        let day = time / DAY_MS;
        if daily.last().map(|(existing, _)| *existing) == Some(day) {
            daily.last_mut().unwrap().1 = *equity;
        } else {
            daily.push((day, *equity));
        }
    }
    daily.into_iter().map(|(_, equity)| nonneg(equity)).collect()
}

/// Annualized Sharpe on daily equity, times 1e4. Fewer than two returns yields 0.
pub fn sharpe_e4(daily_equity: &[u64]) -> i32 {
    if daily_equity.len() < 3 {
        return 0;
    }
    let mut returns: Vec<i128> = Vec::new();
    for pair in daily_equity.windows(2) {
        if pair[0] == 0 {
            return 0;
        }
        returns.push((pair[1] as i128 - pair[0] as i128) * 100_000_000 / pair[0] as i128);
    }
    let n = returns.len() as i128;
    if n < 2 {
        return 0;
    }
    let mean = returns.iter().sum::<i128>() / n;
    let variance = returns.iter().map(|ret| (ret - mean) * (ret - mean)).sum::<i128>() / (n - 1);
    if variance <= 0 {
        return 0;
    }
    let std = variance as u128;
    let std = std.isqrt();
    if std == 0 {
        return 0;
    }
    let sharpe = mean * SQRT_365_E8 / std as i128 / 10_000;
    sharpe.clamp(i32::MIN as i128, i32::MAX as i128) as i32
}

pub fn parse_fixed(input: &str, scale: u32) -> Result<i64, String> {
    let trimmed = input.trim();
    let negative = trimmed.starts_with('-');
    let body = trimmed.trim_start_matches(['+', '-']);
    if body.is_empty() {
        return Err("empty decimal".into());
    }
    let (whole, frac) = body.split_once('.').unwrap_or((body, ""));
    let whole = if whole.is_empty() { "0" } else { whole };
    if !whole.chars().all(|ch| ch.is_ascii_digit()) || !frac.chars().all(|ch| ch.is_ascii_digit()) {
        return Err(format!("bad decimal {input}"));
    }
    let mut value: i128 = whole.parse::<i128>().map_err(|err| err.to_string())?;
    value *= 10i128.pow(scale);
    let mut fraction: i128 = 0;
    let digits = frac.len().min(scale as usize);
    for ch in frac.chars().take(digits) {
        fraction = fraction * 10 + (ch as i128 - '0' as i128);
    }
    if digits > 0 {
        fraction *= 10i128.pow(scale - digits as u32);
    }
    value += fraction;
    if negative {
        value = -value;
    }
    i64::try_from(value).map_err(|_| "decimal overflow".into())
}

fn format_fixed(value: i64, scale: u32) -> String {
    let negative = value < 0;
    let abs = value.unsigned_abs();
    let factor = 10u64.pow(scale);
    let whole = abs / factor;
    let frac = abs % factor;
    let mut text = if frac == 0 {
        whole.to_string()
    } else {
        let mut frac_text = format!("{frac:0width$}", width = scale as usize);
        while frac_text.ends_with('0') {
            frac_text.pop();
        }
        format!("{whole}.{frac_text}")
    };
    if negative {
        text.insert(0, '-');
    }
    text
}

fn trim_decimal(input: &str) -> String {
    let trimmed = input.trim();
    if let Ok(parsed) = parse_fixed(trimmed, 6) {
        format_fixed(parsed, 6)
    } else {
        trimmed.to_string()
    }
}

fn string_field(value: &Value, key: &str) -> Result<String, String> {
    match value.get(key) {
        Some(item) => as_decimal(item),
        None => Err(format!("missing {key}")),
    }
}

fn as_decimal(value: &Value) -> Result<String, String> {
    match value {
        Value::String(text) => Ok(text.clone()),
        Value::Number(number) => Ok(number.to_string()),
        _ => Err("expected decimal".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drawdown_basic() {
        assert_eq!(max_drawdown_bps(&[100, 120, 90, 130]), 2500);
        assert_eq!(max_drawdown_bps(&[]), 0);
    }

    #[test]
    fn drawdown_stays_within_a_full_loss() {
        let mut seed = 1u64;
        for _ in 0..200 {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            let len = 1 + (seed % 24) as usize;
            let curve: Vec<u64> = (0..len)
                .map(|index| 1 + seed.wrapping_add(index as u64).wrapping_mul(17) % 1_000_000)
                .collect();
            let drawdown = max_drawdown_bps(&curve);
            assert!(drawdown <= 10_000);
        }
        assert_eq!(max_drawdown_bps(&[5, 0, 4]), 10_000);
    }

    #[test]
    fn round_trip_hl_fills() {
        let raw = include_bytes!("../../../fixtures/fills/sample.json");
        let fills = fills_from_hl_json(raw).unwrap();
        let again = fills_from_hl_json(fills_to_hl_json(&fills).as_bytes()).unwrap();
        assert_eq!(fills, again);
    }

    fn load(fills: &str, account: &str) -> Metrics {
        compute(
            &fills_from_hl_json(fills.as_bytes()).unwrap(),
            &account_from_hl_json(account.as_bytes()).unwrap(),
        )
    }

    #[test]
    fn three_fixtures_match_hand_computed_metrics() {
        let win = load(
            include_str!("../../../fixtures/fills/sample.json"),
            include_str!("../../../fixtures/fills/sample-account.json"),
        );
        assert_eq!(win.pnl_usd_e6, 8_000_000);
        assert_eq!(win.max_dd_bps, 10);
        assert_eq!(win.sharpe_e4, 108_098);
        assert_eq!(win.n_trades, 2);
        assert_eq!(win.equity_usd_e6, 1_008_000_000);

        let loss = load(
            include_str!("../../../fixtures/fills/loss.json"),
            include_str!("../../../fixtures/fills/loss-account.json"),
        );
        assert_eq!(loss.pnl_usd_e6, -51_000_000);
        assert_eq!(loss.max_dd_bps, 510);
        assert_eq!(loss.sharpe_e4, 0);
        assert_eq!(loss.n_trades, 1);

        let open = load(
            include_str!("../../../fixtures/fills/open.json"),
            include_str!("../../../fixtures/fills/open-account.json"),
        );
        assert_eq!(open.pnl_usd_e6, 19_000_000);
        assert_eq!(open.max_dd_bps, 10);
        assert_eq!(open.n_trades, 1);
        assert_eq!(open.equity_usd_e6, 1_019_000_000);
    }

    #[test]
    fn parses_clearinghouse_state() {
        let raw = include_bytes!("../../../fixtures/fills/sample-account.json");
        let account = account_from_hl_json(raw).unwrap();
        assert_eq!(account.account_value_e6, 1_008_000_000);
        assert!(account.positions.is_empty());
    }
}

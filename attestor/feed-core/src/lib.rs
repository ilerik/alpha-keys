//! Pure trade-feed sanitizer. No I/O and no account address.
use pnl_core::{Fill, Side};
use serde::{Deserialize, Serialize};

pub const DELAYED_MS: u64 = 60_000;
const ROUND_MS: u64 = 5_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tier {
    Live,
    Delayed60,
}

impl Tier {
    pub fn delay_ms(self) -> u64 {
        match self {
            Tier::Live => 0,
            Tier::Delayed60 => DELAYED_MS,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeEvent {
    pub trader_id: String,
    pub seq: u64,
    pub ts_ms: u64,
    pub coin: String,
    pub side: String,
    pub size_bucket: String,
    pub px: String,
    pub reduce_only: bool,
    pub kind: String,
}

pub fn sanitize(
    fills: &[Fill],
    now_ms: u64,
    tier: Tier,
    subscriber_id: &str,
    trader_id: &str,
) -> Vec<TradeEvent> {
    let cutoff = now_ms.saturating_sub(tier.delay_ms());
    let jitter = jitter_ms(subscriber_id);
    let mut ordered: Vec<&Fill> = fills.iter().filter(|fill| fill.time_ms <= cutoff).collect();
    ordered.sort_by_key(|fill| (fill.time_ms, fill.coin.as_str()));
    ordered
        .into_iter()
        .enumerate()
        .map(|(index, fill)| {
            let rounded = fill.time_ms - fill.time_ms % ROUND_MS;
            let kind = kind_of(fill);
            TradeEvent {
                trader_id: trader_id.to_string(),
                seq: index as u64 + 1,
                ts_ms: rounded.saturating_sub(jitter),
                coin: fill.coin.clone(),
                side: match fill.side {
                    Side::Buy => "B".into(),
                    Side::Sell => "A".into(),
                },
                size_bucket: size_bucket(fill.sz_e8).into(),
                px: fill.px_display.clone(),
                reduce_only: kind == "reduce" || kind == "close",
                kind: kind.into(),
            }
        })
        .collect()
}

pub fn size_bucket(sz_e8: i64) -> &'static str {
    let size = sz_e8.unsigned_abs();
    if size <= 1_000_000 {
        "0.01"
    } else if size <= 10_000_000 {
        "0.1"
    } else if size <= 100_000_000 {
        "1"
    } else if size <= 1_000_000_000 {
        "10"
    } else {
        "10+"
    }
}

fn kind_of(fill: &Fill) -> &'static str {
    let delta = match fill.side {
        Side::Buy => fill.sz_e8,
        Side::Sell => -fill.sz_e8,
    };
    let end = fill.start_position_e8.saturating_add(delta);
    if end == 0 {
        "close"
    } else if fill.start_position_e8 == 0 {
        "open"
    } else if end.abs() < fill.start_position_e8.abs() {
        "reduce"
    } else {
        "add"
    }
}

pub fn jitter_ms(subscriber_id: &str) -> u64 {
    let mut hash: u32 = 2_166_136_261;
    for byte in subscriber_id.as_bytes() {
        hash ^= u32::from(*byte);
        hash = hash.wrapping_mul(1_677_761_9);
    }
    u64::from(hash % 3_000)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pnl_core::{fills_from_hl_json, DAY_MS};

    fn sample() -> Vec<Fill> {
        fills_from_hl_json(include_bytes!("../../../fixtures/fills/sample.json")).unwrap()
    }

    #[test]
    fn delayed_tier_drops_fresh_fills_and_buckets_size() {
        let fills = sample();
        let newest = fills.iter().map(|fill| fill.time_ms).max().unwrap();
        let now = newest + 30_000;
        let delayed = sanitize(&fills, now, Tier::Delayed60, "subscriber-a", "alpha-1");
        assert!(delayed.iter().all(|event| event.ts_ms <= now - DELAYED_MS));
        assert_eq!(delayed.len(), 1);
        assert_eq!(delayed[0].size_bucket, "1");
        assert_eq!(delayed[0].kind, "open");
        let live = sanitize(&fills, now, Tier::Live, "subscriber-a", "alpha-1");
        assert_eq!(live.len(), 2);
        assert!(live.iter().all(|event| event.ts_ms <= now));
        assert_eq!(live[1].kind, "close");
        assert!(live[1].reduce_only);
    }

    #[test]
    fn old_fills_pass_the_delay_and_jitter_is_per_subscriber() {
        let fills = sample();
        let newest = fills.iter().map(|fill| fill.time_ms).max().unwrap();
        let now = newest + DELAYED_MS + DAY_MS;
        let first = sanitize(&fills, now, Tier::Delayed60, "subscriber-a", "alpha-1");
        let second = sanitize(&fills, now, Tier::Delayed60, "subscriber-b", "alpha-1");
        assert_eq!(first.len(), 2);
        assert_ne!(first[0].ts_ms, second[0].ts_ms);
        assert!(first.iter().all(|event| event.ts_ms + jitter_ms("subscriber-a") <= now - DELAYED_MS));
    }
}

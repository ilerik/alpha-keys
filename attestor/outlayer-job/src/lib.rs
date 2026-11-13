//! OutLayer job logic, tested with a fake HTTP transport.
//! The live worker is the only place that should set `HL_ADDRESS`.
use feed_core::{sanitize, Tier, TradeEvent};
use pnl_core::{
    account_from_hl_json, compute, fills_from_hl_json, fills_in_window, spot_usdc_e6, window_ms,
    Metrics,
};
use serde_json::{json, Value};

pub const HYPERLIQUID_INFO_URL: &str = "https://api.hyperliquid.xyz/info";

pub trait Fetch {
    fn post_info(&self, body: &str) -> Result<String, String>;
}

pub struct RejectingFetch;

impl Fetch for RejectingFetch {
    fn post_info(&self, _body: &str) -> Result<String, String> {
        Err("hyperliquid https is not available in this build".into())
    }
}

pub fn execute(input: &str, hl_address: Option<&str>, fetch: &dyn Fetch) -> Result<String, String> {
    let request: Value = serde_json::from_str(input).map_err(|err| err.to_string())?;
    let action = request
        .get("action")
        .and_then(Value::as_str)
        .unwrap_or("pnl");
    let trader_id = request
        .get("trader_id")
        .and_then(Value::as_str)
        .unwrap_or("alpha-1");
    if hl_address.map(str::trim).unwrap_or("").is_empty() {
        return Err("missing HL_ADDRESS".into());
    }
    // Hyperliquid documents addresses as lowercase hex. A checksummed (mixed
    // case) secret is normalized so the info endpoint sees the same account.
    let normalized = hl_address.unwrap().trim().to_ascii_lowercase();
    let address = normalized.as_str();
    let fixture = request
        .get("fixture")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let (fills_json, account) = if fixture {
        (
            include_str!("../../../fixtures/fills/sample.json").to_string(),
            account_from_hl_json(
                include_str!("../../../fixtures/fills/sample-account.json").as_bytes(),
            )?,
        )
    } else if action == "feed" {
        let start_ms = window_start(&request)?;
        let body = json!({ "type": "userFillsByTime", "user": address, "startTime": start_ms })
            .to_string();
        let fills_json = fetch.post_info(&body)?;
        let state_body = json!({ "type": "clearinghouseState", "user": address }).to_string();
        let account = account_from_hl_json(fetch.post_info(&state_body)?.as_bytes())?;
        (fills_json, account)
    } else {
        let start_ms = window_start(&request)?;
        let body = json!({ "type": "userFillsByTime", "user": address, "startTime": start_ms })
            .to_string();
        let fills_json = fetch.post_info(&body)?;
        let state_body = json!({ "type": "clearinghouseState", "user": address }).to_string();
        let mut account = account_from_hl_json(fetch.post_info(&state_body)?.as_bytes())?;
        let spot_body = json!({ "type": "spotClearinghouseState", "user": address }).to_string();
        account.spot_usdc_e6 = spot_usdc_e6(fetch.post_info(&spot_body)?.as_bytes())?;
        (fills_json, account)
    };
    let mut fills = fills_from_hl_json(fills_json.as_bytes())?;
    if let Some(now_ms) = request.get("now_ms").and_then(Value::as_u64) {
        let start_ms = window_start(&request)?;
        fills = fills_in_window(&fills, start_ms, now_ms);
    }
    let output = match action {
        "feed" => {
            let now_ms = request.get("now_ms").and_then(Value::as_u64).unwrap_or(0);
            let tier = match request
                .get("tier")
                .and_then(Value::as_str)
                .unwrap_or("Delayed60")
            {
                "Live" => Tier::Live,
                _ => Tier::Delayed60,
            };
            let subscriber = request
                .get("subscriber_id")
                .and_then(Value::as_str)
                .unwrap_or("public");
            let events: Vec<TradeEvent> = sanitize(&fills, now_ms, tier, subscriber, trader_id);
            serde_json::to_string(&events).map_err(|err| err.to_string())?
        }
        _ => {
            let metrics: Metrics = compute(&fills, &account);
            serde_json::to_string(&metrics).map_err(|err| err.to_string())?
        }
    };
    if output.to_ascii_lowercase().contains(address) {
        return Err("refusing to emit the account address".into());
    }
    Ok(output)
}

fn window_start(request: &Value) -> Result<u64, String> {
    let now_ms = request
        .get("now_ms")
        .and_then(Value::as_u64)
        .ok_or("missing now_ms")?;
    let window = request
        .get("window")
        .and_then(Value::as_str)
        .unwrap_or("7d");
    Ok(now_ms.saturating_sub(window_ms(window)?))
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FixtureFetch;
    impl Fetch for FixtureFetch {
        fn post_info(&self, body: &str) -> Result<String, String> {
            if body.contains("clearinghouseState") {
                Ok(include_str!("../../../fixtures/fills/sample-account.json").into())
            } else {
                Ok(include_str!("../../../fixtures/fills/sample.json").into())
            }
        }
    }

    #[test]
    fn mocked_http_metrics_omit_the_account() {
        let address = "hidden-account";
        let output = execute(
            r#"{"action":"pnl","trader_id":"alpha-1","window":"7d","now_ms":1731200000000}"#,
            Some(address),
            &FixtureFetch,
        )
        .unwrap();
        assert!(!output.contains(address));
        assert!(!output.contains("0x"));
        let metrics: Metrics = serde_json::from_str(&output).unwrap();
        assert_eq!(metrics.pnl_usd_e6, 8_000_000);
        assert_eq!(metrics.max_dd_bps, 10);
    }

    #[test]
    fn fixture_flag_does_not_need_a_network() {
        let output = execute(
            r#"{"action":"pnl","fixture":true,"trader_id":"alpha-1"}"#,
            Some("hidden-account"),
            &RejectingFetch,
        )
        .unwrap();
        assert!(output.contains("8000000") || output.contains("8000000"));
        assert!(!output.contains("hidden-account"));
    }

    struct RecordingFetch {
        seen: std::cell::RefCell<String>,
    }

    impl Fetch for RecordingFetch {
        fn post_info(&self, body: &str) -> Result<String, String> {
            self.seen.borrow_mut().push_str(body);
            self.seen.borrow_mut().push('\n');
            if body.contains("spotClearinghouseState") {
                Ok(r#"{"balances":[]}"#.into())
            } else if body.contains("clearinghouseState") {
                Ok(include_str!("../../../fixtures/fills/sample-account.json").into())
            } else {
                Ok(include_str!("../../../fixtures/fills/sample.json").into())
            }
        }
    }

    #[test]
    fn live_fills_request_uses_the_window_start() {
        let fetch = RecordingFetch {
            seen: std::cell::RefCell::new(String::new()),
        };
        let output = execute(
            r#"{"action":"pnl","trader_id":"alpha-1","window":"7d","now_ms":1731200000000}"#,
            Some("hidden-account"),
            &fetch,
        )
        .unwrap();
        let seen = fetch.seen.borrow();
        let body: Value = serde_json::from_str(seen.lines().next().unwrap()).unwrap();
        assert_eq!(body["type"], "userFillsByTime");
        assert_eq!(body["startTime"], 1731200000000u64 - 7 * 86_400_000);
        assert!(!output.contains("hidden-account"));
    }

    #[test]
    fn feed_fills_request_uses_the_window_start() {
        let fetch = RecordingFetch {
            seen: std::cell::RefCell::new(String::new()),
        };
        let output = execute(
            r#"{"action":"feed","trader_id":"alpha-1","tier":"Delayed60","window":"7d","now_ms":1731200000000}"#,
            Some("hidden-account"),
            &fetch,
        )
        .unwrap();
        let seen = fetch.seen.borrow();
        let body: Value = serde_json::from_str(seen.lines().next().unwrap()).unwrap();
        assert_eq!(body["type"], "userFillsByTime");
        assert_eq!(body["startTime"], 1731200000000u64 - 7 * 86_400_000);
        assert!(!output.contains("hidden-account"));
    }

    #[test]
    fn missing_address_is_an_error() {
        let error =
            execute(r#"{"action":"pnl","fixture":true}"#, None, &RejectingFetch).unwrap_err();
        assert_eq!(error, "missing HL_ADDRESS");
    }

    #[test]
    fn live_worker_posts_to_the_hyperliquid_info_endpoint() {
        assert_eq!(HYPERLIQUID_INFO_URL, "https://api.hyperliquid.xyz/info");
    }

    #[test]
    fn checksummed_address_is_sent_lowercase_and_never_emitted() {
        let fetch = RecordingFetch {
            seen: std::cell::RefCell::new(String::new()),
        };
        let mixed = format!("0x{}", "AbCdEf0123".repeat(4));
        let mixed = mixed.as_str();
        let output = execute(
            r#"{"action":"pnl","trader_id":"alpha-1","window":"7d","now_ms":1731200000000}"#,
            Some(mixed),
            &fetch,
        )
        .unwrap();
        let seen = fetch.seen.borrow();
        assert_eq!(seen.lines().count(), 3);
        for line in seen.lines() {
            let body: Value = serde_json::from_str(line).unwrap();
            assert_eq!(body["user"], mixed.to_ascii_lowercase());
        }
        assert!(!output.to_ascii_lowercase().contains(&mixed.to_ascii_lowercase()));
    }
}

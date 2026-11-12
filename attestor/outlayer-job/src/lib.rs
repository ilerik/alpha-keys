//! OutLayer job logic, tested with a fake HTTP transport.
//! The live worker is the only place that should set `HL_ADDRESS`.
use feed_core::{sanitize, Tier, TradeEvent};
use pnl_core::{account_from_hl_json, compute, fills_from_hl_json, Metrics};
use serde_json::{json, Value};

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
    let action = request.get("action").and_then(Value::as_str).unwrap_or("pnl");
    let trader_id = request.get("trader_id").and_then(Value::as_str).unwrap_or("alpha-1");
    if hl_address.map(str::trim).unwrap_or("").is_empty() {
        return Err("missing HL_ADDRESS".into());
    }
    let address = hl_address.unwrap();
    let (fills_json, account_json) = if request.get("fixture").and_then(Value::as_bool).unwrap_or(false) {
        (
            include_str!("../../../fixtures/fills/sample.json").to_string(),
            include_str!("../../../fixtures/fills/sample-account.json").to_string(),
        )
    } else {
        let body = json!({ "type": "userFillsByTime", "user": address }).to_string();
        let fills_json = fetch.post_info(&body)?;
        let state_body = json!({ "type": "clearinghouseState", "user": address }).to_string();
        let account_json = fetch.post_info(&state_body)?;
        (fills_json, account_json)
    };
    let fills = fills_from_hl_json(fills_json.as_bytes())?;
    let account = account_from_hl_json(account_json.as_bytes())?;
    let output = match action {
        "feed" => {
            let now_ms = request.get("now_ms").and_then(Value::as_u64).unwrap_or(0);
            let tier = match request.get("tier").and_then(Value::as_str).unwrap_or("Delayed60") {
                "Live" => Tier::Live,
                _ => Tier::Delayed60,
            };
            let subscriber = request.get("subscriber_id").and_then(Value::as_str).unwrap_or("public");
            let events: Vec<TradeEvent> = sanitize(&fills, now_ms, tier, subscriber, trader_id);
            serde_json::to_string(&events).map_err(|err| err.to_string())?
        }
        _ => {
            let metrics: Metrics = compute(&fills, &account);
            serde_json::to_string(&metrics).map_err(|err| err.to_string())?
        }
    };
    if output.contains(address) {
        return Err("refusing to emit the account address".into());
    }
    Ok(output)
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

    #[test]
    fn missing_address_is_an_error() {
        let error = execute(r#"{"action":"pnl","fixture":true}"#, None, &RejectingFetch).unwrap_err();
        assert_eq!(error, "missing HL_ADDRESS");
    }
}

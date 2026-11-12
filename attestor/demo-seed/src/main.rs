//! Offline judge path: fixture fills -> metrics -> delayed events.
//! Writes `fixtures/feed/alpha-1.jsonl` and prints both JSON documents to stdout.
use feed_core::{sanitize, Tier};
use pnl_core::{account_from_hl_json, compute, fills_from_hl_json};
use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let root = repo_root();
    let fills_path = root.join("fixtures/fills/sample.json");
    let account_path = root.join("fixtures/fills/sample-account.json");
    let fills = fills_from_hl_json(&fs::read(&fills_path).expect("fills fixture")).expect("parse fills");
    let account = account_from_hl_json(&fs::read(&account_path).expect("account fixture")).expect("parse account");
    let metrics = compute(&fills, &account);
    let newest = fills.iter().map(|fill| fill.time_ms).max().unwrap_or(0);
    let now_ms = newest + 120_000;
    let events = sanitize(&fills, now_ms, Tier::Delayed60, "demo-subscriber", "alpha-1");
    let feed_path = root.join("fixtures/feed/alpha-1.jsonl");
    fs::create_dir_all(feed_path.parent().unwrap()).ok();
    let lines: Vec<String> = events.iter().map(|event| serde_json::to_string(event).unwrap()).collect();
    fs::write(&feed_path, lines.join("\n") + "\n").expect("write feed");
    println!("{}", serde_json::to_string(&metrics).unwrap());
    println!("{}", serde_json::to_string(&events).unwrap());
}

fn repo_root() -> PathBuf {
    if let Ok(root) = env::var("ALPHA_KEYS_ROOT") {
        return PathBuf::from(root);
    }
    let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    dir.pop();
    dir.pop();
    dir
}

use anyhow::Result;
use near_workspaces::types::NearToken;
use serde_json::json;
use std::process::Command;

fn contract_wasm() -> Vec<u8> {
    let target = std::env::temp_dir().join("alpha-keys-wasm-target");
    let status = Command::new("cargo")
        .args(["build", "-p", "alpha-keys", "--release", "--target", "wasm32-unknown-unknown"])
        .env("CARGO_TARGET_DIR", &target)
        .status()
        .expect("cargo build");
    assert!(status.success(), "wasm build failed");
    let path = target.join("wasm32-unknown-unknown/release/alpha_keys.wasm");
    std::fs::read(&path).unwrap_or_else(|_| panic!("missing wasm at {}", path.display()))
}

#[tokio::test]
async fn sandbox_mints_transfers_and_pays_royalty() -> Result<()> {
    for key in ["http_proxy", "https_proxy", "HTTP_PROXY", "HTTPS_PROXY", "ALL_PROXY", "all_proxy"] {
        std::env::remove_var(key);
    }
    let worker = near_workspaces::sandbox().await?;
    let wasm = contract_wasm();
    let contract = worker.dev_deploy(&wasm).await?;
    let owner = contract.id();
    let init = contract
        .call("new")
        .args_json(json!({
            "owner_id": owner,
            "outlayer_id": "outlayer.testnet",
            "attestor_code_hash": "hash-1"
        }))
        .transact()
        .await?;
    assert!(init.is_success(), "{init:?}");

    let trader = worker.dev_create_account().await?;
    let buyer = worker.dev_create_account().await?;
    let registered = trader
        .call(contract.id(), "register_trader")
        .args_json(json!({
            "id": "alpha-1",
            "display_name": "Alpha",
            "price_live": "1000000000000000000000000",
            "price_delayed": "500000000000000000000000",
            "royalty_bps": 1000,
            "max_keys": 10
        }))
        .deposit(NearToken::from_near(1))
        .transact()
        .await?;
    assert!(registered.is_success(), "{registered:?}");

    let bought = buyer
        .call(contract.id(), "buy_key")
        .args_json(json!({"trader_id": "alpha-1", "tier": "Delayed60"}))
        .deposit(NearToken::from_near(2))
        .transact()
        .await?;
    assert!(bought.is_success(), "{bought:?}");

    let owned = buyer
        .call(contract.id(), "nft_tokens_for_owner")
        .args_json(json!({"account_id": buyer.id()}))
        .view()
        .await?;
    let tokens: Vec<serde_json::Value> = owned.json()?;
    assert_eq!(tokens.len(), 1);
    let token_id = tokens[0]["token_id"].as_str().unwrap().to_string();

    let payout = contract
        .call("nft_payout")
        .args_json(json!({"token_id": token_id, "balance": "10000", "max_len_payout": 10}))
        .view()
        .await?;
    let payout: serde_json::Value = payout.json()?;
    assert_eq!(payout["payout"][trader.id().as_str()], "1000");
    assert_eq!(payout["payout"][buyer.id().as_str()], "9000");

    let other = worker.dev_create_account().await?;
    let transferred = buyer
        .call(contract.id(), "nft_transfer")
        .args_json(json!({"receiver_id": other.id(), "token_id": token_id}))
        .deposit(NearToken::from_yoctonear(1))
        .transact()
        .await?;
    assert!(transferred.is_success(), "{transferred:?}");

    let access = contract
        .call("has_access")
        .args_json(json!({"account_id": buyer.id(), "trader_id": "alpha-1"}))
        .view()
        .await?;
    let access: Option<String> = access.json()?;
    assert_eq!(access, None);
    Ok(())
}

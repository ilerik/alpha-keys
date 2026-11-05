//! Alpha Keys NEAR contract (STUB). See docs/ARCHITECTURE.md §2 and docs/TASKS.md C-1..C-6.
//! Implementation intentionally omitted; this file pins the public surface for agents.
use near_sdk::json_types::U128;
use near_sdk::{near, AccountId, PanicOnDefault};

pub type TraderId = String;

#[near(serializers = [json, borsh])]
#[derive(Clone, Debug, PartialEq)]
pub enum Tier {
    Live,
    Delayed60,
}

#[near(serializers = [json, borsh])]
#[derive(Clone, Debug)]
pub struct Trader {
    pub id: TraderId,
    pub owner: AccountId,
    pub display_name: String,
    pub price_live: U128,
    pub price_delayed: U128,
    pub royalty_bps: u16,
    pub max_keys: u32,
    pub keys_sold: u32,
    pub active: bool,
}

#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct AlphaKeys {
    owner_id: AccountId,
    outlayer_id: AccountId,
    attestor_code_hash: String,
    // TODO(C-2..C-6): traders, pnl records, NonFungibleToken, minters, key metadata
}

#[near]
impl AlphaKeys {
    #[init]
    pub fn new(owner_id: AccountId, outlayer_id: AccountId, attestor_code_hash: String) -> Self {
        Self { owner_id, outlayer_id, attestor_code_hash }
    }

    pub fn get_config(&self) -> (AccountId, AccountId, String) {
        (self.owner_id.clone(), self.outlayer_id.clone(), self.attestor_code_hash.clone())
    }

    // TODO(C-2) register_trader / get_trader / list_traders
    // TODO(C-3) NEP-171/177/178/181/199 via near_contract_standards
    // TODO(C-4) buy_key / ft_on_transfer / mint_key
    // TODO(C-5) has_access(account_id, trader_id) -> Option<Tier>
    // TODO(C-6, A-6) request_pnl_update -> outlayer.request_execution -> on_pnl_result; submit_pnl fallback
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_sets_config() {
        let c = AlphaKeys::new("owner.testnet".parse().unwrap(), "outlayer.testnet".parse().unwrap(), "sha256:TODO".into());
        assert_eq!(c.get_config().1.as_str(), "outlayer.testnet");
    }
}

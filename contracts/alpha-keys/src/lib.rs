//! Alpha Keys NEAR contract: trader registry, viewing-key NFT, attested PnL records.
//! NFT core, metadata, approval, and enumeration come from `near-contract-standards`.
//! NEP-199 `nft_payout` is implemented here because crate 5.30 has no royalty module.
use near_contract_standards::non_fungible_token::events::NftMint;
use near_contract_standards::non_fungible_token::metadata::{
    NFTContractMetadata, NonFungibleTokenMetadataProvider, TokenMetadata, NFT_METADATA_SPEC,
};
use near_contract_standards::non_fungible_token::{
    approval::NonFungibleTokenApproval, core::NonFungibleTokenCore,
    core::NonFungibleTokenResolver, enumeration::NonFungibleTokenEnumeration, Token, TokenId,
    NonFungibleToken,
};
use near_sdk::collections::{LazyOption, LookupMap, UnorderedMap, UnorderedSet, Vector};
use near_sdk::json_types::{U128, U64};
use near_sdk::{
    env, ext_contract, near, require, AccountId, BorshStorageKey, Gas, NearToken, PanicOnDefault,
    Promise, PromiseError, PromiseOrValue,
};
use std::collections::HashMap;

pub type TraderId = String;

#[near(serializers = [json, borsh])]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Tier {
    Live,
    Delayed60,
}

impl Tier {
    fn rank(&self) -> u8 {
        match self {
            Tier::Delayed60 => 1,
            Tier::Live => 2,
        }
    }
}

#[near(serializers = [json, borsh])]
#[derive(Clone, Debug, PartialEq)]
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

#[near(serializers = [json, borsh])]
#[derive(Clone, Debug, PartialEq)]
pub struct KeyMeta {
    pub trader_id: TraderId,
    pub tier: Tier,
    pub expires_at_ns: Option<U64>,
    pub purchase_ref: Option<String>,
}

#[near(serializers = [json, borsh])]
#[derive(Clone, Debug, PartialEq)]
pub enum AttestSource {
    OutLayerCallback { request_id: Option<String> },
    OutLayerHttps { attestation_ref: String, relayer: AccountId },
}

#[near(serializers = [json, borsh])]
#[derive(Clone, Debug, PartialEq)]
pub struct PnlRecord {
    pub ts_ns: U64,
    pub window: String,
    pub pnl_usd_e6: i64,
    pub max_dd_bps: u32,
    pub sharpe_e4: i32,
    pub n_trades: u32,
    pub equity_usd_e6: u64,
    pub code_hash: String,
    pub input_hash: String,
    pub source: AttestSource,
}

#[near(serializers = [json])]
#[derive(Clone, Debug, PartialEq)]
pub struct Payout {
    pub payout: HashMap<AccountId, U128>,
}

#[near(serializers = [json, borsh])]
#[derive(Clone, Debug)]
struct PnlOutput {
    pnl_usd_e6: i64,
    max_dd_bps: u32,
    sharpe_e4: i32,
    n_trades: u32,
    equity_usd_e6: u64,
}

#[derive(BorshStorageKey)]
#[near]
enum StorageKey {
    NonFungibleToken,
    Metadata,
    TokenMetadata,
    Enumeration,
    Approval,
    Traders,
    KeyMeta,
    Minters,
    Pnl,
}

#[derive(PanicOnDefault)]
#[near(contract_state)]
pub struct AlphaKeys {
    owner_id: AccountId,
    outlayer_id: AccountId,
    attestor_code_hash: String,
    attestor_wasm_url: String,
    secrets_profile: String,
    secrets_account_id: AccountId,
    next_token_id: u64,
    tokens: NonFungibleToken,
    metadata: LazyOption<NFTContractMetadata>,
    traders: UnorderedMap<TraderId, Trader>,
    key_meta: LookupMap<TokenId, KeyMeta>,
    minters: UnorderedSet<AccountId>,
    pnl: LookupMap<TraderId, Vector<PnlRecord>>,
}

#[allow(dead_code)]
#[ext_contract(ext_outlayer)]
trait OutLayer {
    fn request_execution(
        &mut self,
        source: near_sdk::serde_json::Value,
        resource_limits: Option<near_sdk::serde_json::Value>,
        input_data: Option<String>,
        secrets_ref: Option<near_sdk::serde_json::Value>,
        response_format: Option<String>,
        payer_account_id: Option<AccountId>,
        params: Option<near_sdk::serde_json::Value>,
    );
}

#[allow(dead_code)]
#[ext_contract(ext_self)]
trait ExtSelf {
    fn on_pnl_result(
        &mut self,
        trader_id: TraderId,
        window: String,
        input_hash: String,
        code_hash: String,
    );
}

const CALLBACK_GAS: Gas = Gas::from_tgas(40);
/// Floor for `outlayer.request_execution`; it also gets the unused gas.
const OUTLAYER_MIN_GAS: Gas = Gas::from_tgas(100);
/// How far `now_ms` may sit from the block time.
const MAX_CLOCK_SKEW_MS: u64 = 600_000;
/// The least prepaid gas `request_pnl_update` accepts.
const MIN_REQUEST_GAS: Gas = Gas::from_tgas(200);

#[near]
impl AlphaKeys {
    #[init]
    pub fn new(owner_id: AccountId, outlayer_id: AccountId, attestor_code_hash: String) -> Self {
        require!(!env::state_exists(), "Already initialized");
        let metadata = NFTContractMetadata {
            spec: NFT_METADATA_SPEC.to_string(),
            name: "Alpha Keys".to_string(),
            symbol: "AKEY".to_string(),
            icon: None,
            base_uri: None,
            reference: None,
            reference_hash: None,
        };
        metadata.assert_valid();
        Self {
            tokens: NonFungibleToken::new(
                StorageKey::NonFungibleToken,
                owner_id.clone(),
                Some(StorageKey::TokenMetadata),
                Some(StorageKey::Enumeration),
                Some(StorageKey::Approval),
            ),
            metadata: LazyOption::new(StorageKey::Metadata, Some(&metadata)),
            owner_id,
            outlayer_id,
            attestor_code_hash,
            attestor_wasm_url: String::new(),
            secrets_profile: String::new(),
            secrets_account_id: "outlayer.testnet".parse().unwrap(),
            next_token_id: 0,
            traders: UnorderedMap::new(StorageKey::Traders),
            key_meta: LookupMap::new(StorageKey::KeyMeta),
            minters: UnorderedSet::new(StorageKey::Minters),
            pnl: LookupMap::new(StorageKey::Pnl),
        }
    }

    pub fn get_config(&self) -> (AccountId, AccountId, String) {
        (self.owner_id.clone(), self.outlayer_id.clone(), self.attestor_code_hash.clone())
    }

    #[payable]
    pub fn register_trader(
        &mut self,
        id: TraderId,
        display_name: String,
        price_live: U128,
        price_delayed: U128,
        royalty_bps: u16,
        max_keys: u32,
    ) {
        require!(royalty_bps <= 10_000, "royalty_bps");
        require!(self.traders.get(&id).is_none(), "duplicate trader");
        let before = env::storage_usage();
        let trader = Trader {
            id: id.clone(),
            owner: env::predecessor_account_id(),
            display_name,
            price_live,
            price_delayed,
            royalty_bps,
            max_keys,
            keys_sold: 0,
            active: true,
        };
        self.traders.insert(&id, &trader);
        refund_deposit(env::storage_usage() - before);
    }

    #[payable]
    pub fn update_trader(
        &mut self,
        id: TraderId,
        display_name: String,
        price_live: U128,
        price_delayed: U128,
        royalty_bps: u16,
        max_keys: u32,
        active: bool,
    ) {
        require!(royalty_bps <= 10_000, "royalty_bps");
        let mut trader = self.traders.get(&id).unwrap_or_else(|| env::panic_str("unknown trader"));
        require!(trader.owner == env::predecessor_account_id(), "owner only");
        require!(max_keys >= trader.keys_sold, "max_keys below sold");
        let before = env::storage_usage();
        trader.display_name = display_name;
        trader.price_live = price_live;
        trader.price_delayed = price_delayed;
        trader.royalty_bps = royalty_bps;
        trader.max_keys = max_keys;
        trader.active = active;
        self.traders.insert(&id, &trader);
        let after = env::storage_usage();
        if after >= before {
            refund_deposit(after - before);
        }
    }

    pub fn get_trader(&self, trader_id: TraderId) -> Option<Trader> {
        self.traders.get(&trader_id)
    }

    pub fn list_traders(&self, from: u64, limit: u64) -> Vec<Trader> {
        self.traders.iter().skip(from as usize).take(limit as usize).map(|(_, trader)| trader).collect()
    }

    #[payable]
    pub fn add_minter(&mut self, account_id: AccountId) {
        self.require_owner();
        let before = env::storage_usage();
        self.minters.insert(&account_id);
        refund_deposit(env::storage_usage() - before);
    }

    pub fn set_attestor_code_hash(&mut self, code_hash: String) {
        self.require_owner();
        self.attestor_code_hash = code_hash;
    }

    pub fn set_attestor_source(
        &mut self,
        wasm_url: String,
        secrets_profile: String,
        secrets_account_id: AccountId,
    ) {
        self.require_owner();
        self.attestor_wasm_url = wasm_url;
        self.secrets_profile = secrets_profile;
        self.secrets_account_id = secrets_account_id;
    }

    #[payable]
    pub fn buy_key(&mut self, trader_id: TraderId, tier: Tier) -> TokenId {
        let price = self.price_of(&trader_id, &tier);
        if env::attached_deposit().as_yoctonear() < price {
            env::panic_str("underpayment");
        }
        let before = env::storage_usage();
        let token_id = self.mint_to(env::predecessor_account_id(), &trader_id, tier.clone(), None, None);
        let used = env::storage_usage().saturating_sub(before);
        let storage = env::storage_byte_cost().saturating_mul(u128::from(used)).as_yoctonear();
        let trader = self.traders.get(&trader_id).unwrap();
        self.settle_sale(price, storage, trader.owner);
        token_id
    }

    #[payable]
    pub fn mint_key(
        &mut self,
        receiver_id: AccountId,
        trader_id: TraderId,
        tier: Tier,
        purchase_ref: String,
        expires_at_ns: Option<U64>,
    ) -> TokenId {
        require!(self.minters.contains(&env::predecessor_account_id()), "not a minter");
        let before = env::storage_usage();
        let token_id =
            self.mint_to(receiver_id, &trader_id, tier, Some(purchase_ref), expires_at_ns);
        refund_deposit(env::storage_usage() - before);
        token_id
    }

    pub fn has_access(&self, account_id: AccountId, trader_id: TraderId) -> Option<Tier> {
        let tokens = self.tokens.nft_tokens_for_owner(account_id, None, Some(100));
        let now = env::block_timestamp();
        let mut best: Option<Tier> = None;
        for token in tokens {
            let Some(meta) = self.key_meta.get(&token.token_id) else { continue };
            if meta.trader_id != trader_id {
                continue;
            }
            if let Some(exp) = meta.expires_at_ns {
                if exp.0 <= now {
                    continue;
                }
            }
            best = Some(match best {
                Some(current) if current.rank() >= meta.tier.rank() => current,
                _ => meta.tier,
            });
        }
        best
    }

    pub fn get_key(&self, token_id: TokenId) -> Option<KeyMeta> {
        self.key_meta.get(&token_id)
    }

    pub fn nft_payout(&self, token_id: TokenId, balance: U128, max_len_payout: u32) -> Payout {
        let token = self.tokens.nft_token(token_id.clone()).unwrap_or_else(|| env::panic_str("no token"));
        let mut payout: HashMap<AccountId, U128> = HashMap::new();
        let amount = balance.0;
        let royalty_bps = self
            .key_meta
            .get(&token_id)
            .and_then(|meta| self.traders.get(&meta.trader_id))
            .map(|trader| trader.royalty_bps)
            .unwrap_or(0);
        let royalty = amount.saturating_mul(u128::from(royalty_bps)) / 10_000;
        let seller_share = amount.saturating_sub(royalty);
        if royalty > 0 {
            if let Some(meta) = self.key_meta.get(&token_id) {
                if let Some(trader) = self.traders.get(&meta.trader_id) {
                    payout.insert(trader.owner, U128(royalty));
                }
            }
        }
        if seller_share > 0 {
            let entry = payout.entry(token.owner_id).or_insert(U128(0));
            entry.0 = entry.0.saturating_add(seller_share);
        }
        require!((payout.len() as u32) <= max_len_payout, "payout too long");
        Payout { payout }
    }

    #[payable]
    pub fn nft_transfer_payout(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        approval_id: Option<u64>,
        memo: Option<String>,
        balance: U128,
        max_len_payout: u32,
    ) -> Payout {
        let payout = self.nft_payout(token_id.clone(), balance, max_len_payout);
        self.nft_transfer(receiver_id, token_id, approval_id, memo);
        payout
    }

    pub fn get_pnl_history(&self, trader_id: TraderId, from: u64, limit: u64) -> Vec<PnlRecord> {
        let Some(history) = self.pnl.get(&trader_id) else { return Vec::new() };
        history.iter().skip(from as usize).take(limit as usize).collect()
    }

    #[payable]
    pub fn request_pnl_update(&mut self, trader_id: TraderId, window: String, now_ms: u64) -> Promise {
        require!(self.traders.get(&trader_id).is_some(), "unknown trader");
        require!(!self.attestor_wasm_url.is_empty(), "attestor wasm url not set");
        // The window ends now. A caller may not pick an older, better week.
        let block_ms = env::block_timestamp() / 1_000_000;
        require!(now_ms.abs_diff(block_ms) <= MAX_CLOCK_SKEW_MS, "now_ms must be within 10 minutes of the block time");
        let input = near_sdk::serde_json::json!({
            "action": "pnl",
            "trader_id": trader_id,
            "window": window,
            "now_ms": now_ms,
        })
        .to_string();
        let input_hash = hex_encode(&env::sha256(input.as_bytes()));
        let code_hash = self.attestor_code_hash.clone();
        let source = near_sdk::serde_json::json!({
            "WasmUrl": {
                "url": self.attestor_wasm_url,
                "hash": code_hash,
                "build_target": "wasm32-wasip2",
            }
        });
        let limits = near_sdk::serde_json::json!({
            "max_instructions": 50_000_000_000u64,
            "max_memory_mb": 128,
            "max_execution_seconds": 60,
        });
        let secrets = if self.secrets_profile.is_empty() {
            None
        } else {
            Some(near_sdk::serde_json::json!({
                "profile": self.secrets_profile,
                "account_id": self.secrets_account_id,
            }))
        };
        // OutLayer gets a fixed floor plus every unused gas unit. Reserving
        // `prepaid - callback - 5 Tgas` as static gas left no room for the two
        // promises this call creates, so the runtime refused it (GasExceeded).
        require!(env::prepaid_gas() >= MIN_REQUEST_GAS, "attach at least 200 Tgas");
        ext_outlayer::ext(self.outlayer_id.clone())
            .with_attached_deposit(env::attached_deposit())
            .with_static_gas(OUTLAYER_MIN_GAS)
            .with_unused_gas_weight(1)
            .request_execution(source, Some(limits), Some(input), secrets, Some("Text".into()), Some(env::predecessor_account_id()), None)
            .then(
                ext_self::ext(env::current_account_id())
                    .with_static_gas(CALLBACK_GAS)
                    .with_unused_gas_weight(0)
                    .on_pnl_result(trader_id, window, input_hash, code_hash),
            )
    }

    #[private]
    pub fn on_pnl_result(
        &mut self,
        trader_id: TraderId,
        window: String,
        input_hash: String,
        code_hash: String,
        #[callback_result] result: Result<Option<String>, PromiseError>,
    ) {
        require!(env::predecessor_account_id() == env::current_account_id(), "private callback");
        if code_hash != self.attestor_code_hash {
            self.reject_pnl(&trader_id, "code hash");
            return;
        }
        let text = match result {
            Ok(Some(text)) => text,
            Ok(None) => {
                self.reject_pnl(&trader_id, "empty output");
                return;
            }
            Err(_) => {
                self.reject_pnl(&trader_id, "execution failed");
                return;
            }
        };
        let output: PnlOutput = match near_sdk::serde_json::from_str(&text) {
            Ok(output) => output,
            Err(_) => {
                self.reject_pnl(&trader_id, "malformed output");
                return;
            }
        };
        self.store_pnl(PnlRecord {
            ts_ns: U64(env::block_timestamp()),
            window,
            pnl_usd_e6: output.pnl_usd_e6,
            max_dd_bps: output.max_dd_bps,
            sharpe_e4: output.sharpe_e4,
            n_trades: output.n_trades,
            equity_usd_e6: output.equity_usd_e6,
            code_hash,
            input_hash,
            source: AttestSource::OutLayerCallback { request_id: None },
        }, &trader_id);
    }

    pub fn submit_pnl(
        &mut self,
        trader_id: TraderId,
        window: String,
        pnl_usd_e6: i64,
        max_dd_bps: u32,
        sharpe_e4: i32,
        n_trades: u32,
        equity_usd_e6: u64,
        code_hash: String,
        input_hash: String,
        attestation_ref: String,
    ) {
        let relayer = env::predecessor_account_id();
        require!(self.minters.contains(&relayer), "not a minter");
        require!(code_hash == self.attestor_code_hash, "code hash");
        require!(self.traders.get(&trader_id).is_some(), "unknown trader");
        self.store_pnl(
            PnlRecord {
                ts_ns: U64(env::block_timestamp()),
                window,
                pnl_usd_e6,
                max_dd_bps,
                sharpe_e4,
                n_trades,
                equity_usd_e6,
                code_hash,
                input_hash,
                source: AttestSource::OutLayerHttps { attestation_ref, relayer },
            },
            &trader_id,
        );
    }
}

impl AlphaKeys {
    fn require_owner(&self) {
        require!(env::predecessor_account_id() == self.owner_id, "owner only");
    }

    fn price_of(&self, trader_id: &str, tier: &Tier) -> u128 {
        let trader = self.traders.get(&trader_id.to_string()).unwrap_or_else(|| env::panic_str("unknown trader"));
        require!(trader.active, "inactive");
        require!(trader.keys_sold < trader.max_keys, "max_keys");
        match tier {
            Tier::Live => trader.price_live.0,
            Tier::Delayed60 => trader.price_delayed.0,
        }
    }

    fn mint_to(
        &mut self,
        receiver: AccountId,
        trader_id: &str,
        tier: Tier,
        purchase_ref: Option<String>,
        expires_at_ns: Option<U64>,
    ) -> TokenId {
        let mut trader =
            self.traders.get(&trader_id.to_string()).unwrap_or_else(|| env::panic_str("unknown trader"));
        require!(trader.active, "inactive");
        require!(trader.keys_sold < trader.max_keys, "max_keys");
        let token_id = self.next_token_id.to_string();
        self.next_token_id += 1;
        trader.keys_sold += 1;
        self.traders.insert(&trader_id.to_string(), &trader);
        let metadata = TokenMetadata {
            title: Some(format!("Alpha Key {trader_id}")),
            description: Some(match tier {
                Tier::Live => "Live viewing key".to_string(),
                Tier::Delayed60 => "Delayed viewing key".to_string(),
            }),
            media: None,
            media_hash: None,
            copies: Some(1),
            issued_at: Some(env::block_timestamp().to_string()),
            expires_at: expires_at_ns.map(|t| t.0.to_string()),
            starts_at: None,
            updated_at: None,
            extra: None,
            reference: None,
            reference_hash: None,
        };
        let token = self.tokens.internal_mint_with_refund(token_id.clone(), receiver, Some(metadata), None);
        NftMint { owner_id: &token.owner_id, token_ids: &[&token.token_id], memo: None }.emit();
        self.key_meta.insert(
            &token_id,
            &KeyMeta { trader_id: trader_id.to_string(), tier, expires_at_ns, purchase_ref },
        );
        token_id
    }

    fn settle_sale(&self, price: u128, storage: u128, trader_owner: AccountId) {
        let attached = env::attached_deposit().as_yoctonear();
        let (to_trader, refund) = split_payment(attached, price, storage).unwrap_or_else(|msg| env::panic_str(msg));
        if to_trader > 0 {
            Promise::new(trader_owner).transfer(NearToken::from_yoctonear(to_trader)).detach();
        }
        if refund > 1 {
            Promise::new(env::predecessor_account_id()).transfer(NearToken::from_yoctonear(refund)).detach();
        }
    }

    fn store_pnl(&mut self, record: PnlRecord, trader_id: &str) {
        let mut history = self.pnl.get(&trader_id.to_string()).unwrap_or_else(|| {
            Vector::new(pnl_prefix(trader_id))
        });
        history.push(&record);
        self.pnl.insert(&trader_id.to_string(), &history);
    }

    fn reject_pnl(&self, trader_id: &str, reason: &str) {
        env::log_str(&format!(
            "EVENT_JSON:{{\"standard\":\"alpha-keys\",\"event\":\"pnl_rejected\",\"data\":{{\"trader_id\":\"{trader_id}\",\"reason\":\"{reason}\"}}}}"
        ));
    }
}

pub fn split_payment(attached: u128, price: u128, storage: u128) -> Result<(u128, u128), &'static str> {
    let need = price.saturating_add(storage);
    if attached < need {
        return Err("underpayment");
    }
    Ok((price, attached - need))
}

fn refund_deposit(storage_used: u64) {
    let required = env::storage_byte_cost().saturating_mul(u128::from(storage_used));
    let attached = env::attached_deposit();
    require!(required <= attached, "insufficient storage deposit");
    let refund = attached.saturating_sub(required);
    if refund.as_yoctonear() > 1 {
        Promise::new(env::predecessor_account_id()).transfer(refund).detach();
    }
}

fn pnl_prefix(trader_id: &str) -> Vec<u8> {
    let mut prefix = b"pnl:".to_vec();
    prefix.extend(env::sha256(trader_id.as_bytes()));
    prefix
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0xf) as usize] as char);
    }
    out
}

#[near]
impl NonFungibleTokenCore for AlphaKeys {
    #[payable]
    fn nft_transfer(&mut self, receiver_id: AccountId, token_id: TokenId, approval_id: Option<u64>, memo: Option<String>) {
        self.tokens.nft_transfer(receiver_id, token_id, approval_id, memo);
    }

    #[payable]
    fn nft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        approval_id: Option<u64>,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<bool> {
        self.tokens.nft_transfer_call(receiver_id, token_id, approval_id, memo, msg)
    }

    fn nft_token(&self, token_id: TokenId) -> Option<Token> {
        self.tokens.nft_token(token_id)
    }
}

#[near]
impl NonFungibleTokenResolver for AlphaKeys {
    #[private]
    fn nft_resolve_transfer(
        &mut self,
        previous_owner_id: AccountId,
        receiver_id: AccountId,
        token_id: TokenId,
        approved_account_ids: Option<HashMap<AccountId, u64>>,
    ) -> bool {
        self.tokens.nft_resolve_transfer(previous_owner_id, receiver_id, token_id, approved_account_ids)
    }
}

#[near]
impl NonFungibleTokenApproval for AlphaKeys {
    #[payable]
    fn nft_approve(&mut self, token_id: TokenId, account_id: AccountId, msg: Option<String>) -> Option<Promise> {
        self.tokens.nft_approve(token_id, account_id, msg)
    }

    #[payable]
    fn nft_revoke(&mut self, token_id: TokenId, account_id: AccountId) {
        self.tokens.nft_revoke(token_id, account_id);
    }

    #[payable]
    fn nft_revoke_all(&mut self, token_id: TokenId) {
        self.tokens.nft_revoke_all(token_id);
    }

    fn nft_is_approved(&self, token_id: TokenId, approved_account_id: AccountId, approval_id: Option<u64>) -> bool {
        self.tokens.nft_is_approved(token_id, approved_account_id, approval_id)
    }
}

#[near]
impl NonFungibleTokenEnumeration for AlphaKeys {
    fn nft_total_supply(&self) -> U128 {
        self.tokens.nft_total_supply()
    }

    fn nft_tokens(&self, from_index: Option<U128>, limit: Option<u64>) -> Vec<Token> {
        self.tokens.nft_tokens(from_index, limit)
    }

    fn nft_supply_for_owner(&self, account_id: AccountId) -> U128 {
        self.tokens.nft_supply_for_owner(account_id)
    }

    fn nft_tokens_for_owner(&self, account_id: AccountId, from_index: Option<U128>, limit: Option<u64>) -> Vec<Token> {
        self.tokens.nft_tokens_for_owner(account_id, from_index, limit)
    }
}

#[near]
impl NonFungibleTokenMetadataProvider for AlphaKeys {
    fn nft_metadata(&self) -> NFTContractMetadata {
        self.metadata.get().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::testing_env;

    const ONE_NEAR: u128 = 1_000_000_000_000_000_000_000_000;

    fn account(name: &str) -> AccountId {
        name.parse().unwrap()
    }

    fn ctx(predecessor: &str, deposit_yocto: u128, now_ns: u64) -> AlphaKeys {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(account("alpha.testnet"))
            .signer_account_id(account(predecessor))
            .predecessor_account_id(account(predecessor))
            .attached_deposit(NearToken::from_yoctonear(deposit_yocto))
            .block_timestamp(now_ns)
            .prepaid_gas(Gas::from_tgas(300));
        testing_env!(builder.build());
        AlphaKeys::new(account("owner.testnet"), account("outlayer.testnet"), "hash-1".into())
    }

    fn as_pred(predecessor: &str, deposit_yocto: u128, now_ns: u64) {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(account("alpha.testnet"))
            .signer_account_id(account(predecessor))
            .predecessor_account_id(account(predecessor))
            .attached_deposit(NearToken::from_yoctonear(deposit_yocto))
            .block_timestamp(now_ns)
            .storage_usage(env::storage_usage())
            .prepaid_gas(Gas::from_tgas(300));
        testing_env!(builder.build());
    }

    fn register(contract: &mut AlphaKeys, id: &str, max_keys: u32) {
        as_pred("trader.testnet", 2 * ONE_NEAR, 1_000);
        contract.register_trader(id.into(), "Alpha".into(), U128(ONE_NEAR), U128(ONE_NEAR / 2), 1_000, max_keys);
    }

    #[test]
    fn init_sets_config() {
        let contract = ctx("owner.testnet", 0, 1);
        assert_eq!(contract.get_config().1, account("outlayer.testnet"));
    }

    #[test]
    fn register_get_and_page() {
        let mut contract = ctx("owner.testnet", 0, 1);
        register(&mut contract, "alpha-1", 10);
        as_pred("other.testnet", 2 * ONE_NEAR, 1);
        contract.register_trader("alpha-2".into(), "Beta".into(), U128(1), U128(1), 0, 5);
        let page = contract.list_traders(0, 1);
        assert_eq!(page.len(), 1);
        assert_eq!(page[0].id, "alpha-1");
        assert_eq!(contract.list_traders(1, 10).len(), 1);
        assert_eq!(contract.get_trader("alpha-1".into()).unwrap().owner, account("trader.testnet"));
    }

    #[test]
    #[should_panic(expected = "duplicate trader")]
    fn duplicate_trader_panics() {
        let mut contract = ctx("owner.testnet", 0, 1);
        register(&mut contract, "alpha-1", 10);
        register(&mut contract, "alpha-1", 10);
    }

    #[test]
    #[should_panic(expected = "owner only")]
    fn update_rejects_non_owner() {
        let mut contract = ctx("owner.testnet", 0, 1);
        register(&mut contract, "alpha-1", 10);
        as_pred("buyer.testnet", ONE_NEAR, 1);
        contract.update_trader("alpha-1".into(), "nope".into(), U128(1), U128(1), 0, 10, true);
    }

    #[test]
    fn update_by_trader_owner() {
        let mut contract = ctx("owner.testnet", 0, 1);
        register(&mut contract, "alpha-1", 10);
        as_pred("trader.testnet", ONE_NEAR, 1);
        contract.update_trader("alpha-1".into(), "Renamed".into(), U128(3), U128(2), 500, 10, false);
        let trader = contract.get_trader("alpha-1".into()).unwrap();
        assert_eq!(trader.display_name, "Renamed");
        assert!(!trader.active);
    }

    #[test]
    #[should_panic(expected = "underpayment")]
    fn underpay_is_rejected() {
        let mut contract = ctx("owner.testnet", 0, 1);
        register(&mut contract, "alpha-1", 10);
        as_pred("buyer.testnet", 1, 5_000);
        contract.buy_key("alpha-1".into(), Tier::Delayed60);
    }

    #[test]
    fn buy_mints_and_grants_access() {
        let mut contract = ctx("owner.testnet", 0, 1);
        register(&mut contract, "alpha-1", 10);
        as_pred("buyer.testnet", 3 * ONE_NEAR, 5_000);
        let token_id = contract.buy_key("alpha-1".into(), Tier::Live);
        assert_eq!(contract.get_trader("alpha-1".into()).unwrap().keys_sold, 1);
        assert_eq!(contract.has_access(account("buyer.testnet"), "alpha-1".into()), Some(Tier::Live));
        let owned = contract.nft_tokens_for_owner(account("buyer.testnet"), None, None);
        assert_eq!(owned.len(), 1);
        assert_eq!(owned[0].token_id, token_id);
    }

    #[test]
    #[should_panic(expected = "max_keys")]
    fn max_keys_enforced() {
        let mut contract = ctx("owner.testnet", 0, 1);
        register(&mut contract, "alpha-1", 1);
        as_pred("buyer.testnet", 3 * ONE_NEAR, 5_000);
        contract.buy_key("alpha-1".into(), Tier::Delayed60);
        as_pred("buyer.testnet", 3 * ONE_NEAR, 6_000);
        contract.buy_key("alpha-1".into(), Tier::Delayed60);
    }

    #[test]
    fn overpay_split_refunds_the_excess() {
        assert_eq!(split_payment(5, 2, 1), Ok((2, 2)));
        assert_eq!(split_payment(2, 2, 0), Ok((2, 0)));
        assert!(split_payment(1, 2, 0).is_err());
    }

    #[test]
    #[should_panic(expected = "not a minter")]
    fn non_minter_cannot_mint() {
        let mut contract = ctx("owner.testnet", 0, 1);
        register(&mut contract, "alpha-1", 10);
        as_pred("buyer.testnet", 2 * ONE_NEAR, 5_000);
        contract.mint_key(account("buyer.testnet"), "alpha-1".into(), Tier::Delayed60, "ref".into(), None);
    }

    #[test]
    fn minter_stores_purchase_ref() {
        let mut contract = ctx("owner.testnet", 0, 1);
        register(&mut contract, "alpha-1", 10);
        as_pred("owner.testnet", ONE_NEAR, 2);
        contract.add_minter(account("minter.testnet"));
        as_pred("minter.testnet", 2 * ONE_NEAR, 5_000);
        let token_id = contract.mint_key(
            account("buyer.testnet"),
            "alpha-1".into(),
            Tier::Delayed60,
            "quote-1".into(),
            None,
        );
        assert_eq!(contract.get_key(token_id).unwrap().purchase_ref.unwrap(), "quote-1");
    }

    #[test]
    fn access_follows_ownership_tier_and_expiry() {
        let mut contract = ctx("owner.testnet", 0, 1);
        register(&mut contract, "alpha-1", 10);
        as_pred("owner.testnet", ONE_NEAR, 2);
        contract.add_minter(account("minter.testnet"));
        assert_eq!(contract.has_access(account("buyer.testnet"), "alpha-1".into()), None);

        as_pred("minter.testnet", 2 * ONE_NEAR, 1_000);
        contract.mint_key(account("buyer.testnet"), "alpha-1".into(), Tier::Delayed60, "d".into(), None);
        as_pred("minter.testnet", 2 * ONE_NEAR, 1_000);
        contract.mint_key(account("buyer.testnet"), "alpha-1".into(), Tier::Live, "l".into(), None);
        assert_eq!(contract.has_access(account("buyer.testnet"), "alpha-1".into()), Some(Tier::Live));

        as_pred("minter.testnet", 2 * ONE_NEAR, 1_000);
        let expired = contract.mint_key(
            account("other.testnet"),
            "alpha-1".into(),
            Tier::Live,
            "old".into(),
            Some(U64(50)),
        );
        as_pred("other.testnet", 1, 100);
        assert_eq!(contract.has_access(account("other.testnet"), "alpha-1".into()), None);
        assert!(contract.get_key(expired).is_some());

        let owned = contract.nft_tokens_for_owner(account("buyer.testnet"), None, None);
        assert_eq!(owned.len(), 2);
        as_pred("buyer.testnet", 1, 200);
        for token in owned {
            contract.nft_transfer(account("other.testnet"), token.token_id, None, None);
        }
        assert!(contract.nft_tokens_for_owner(account("buyer.testnet"), None, None).is_empty());
        assert_eq!(contract.has_access(account("buyer.testnet"), "alpha-1".into()), None);
        assert_eq!(contract.has_access(account("other.testnet"), "alpha-1".into()), Some(Tier::Live));
    }

    #[test]
    fn payout_splits_royalty() {
        let mut contract = ctx("owner.testnet", 0, 1);
        register(&mut contract, "alpha-1", 10);
        as_pred("buyer.testnet", 3 * ONE_NEAR, 5_000);
        let token_id = contract.buy_key("alpha-1".into(), Tier::Delayed60);
        let payout = contract.nft_payout(token_id, U128(10_000), 10);
        assert_eq!(payout.payout.get(&account("trader.testnet")), Some(&U128(1_000)));
        assert_eq!(payout.payout.get(&account("buyer.testnet")), Some(&U128(9_000)));
    }

    #[test]
    #[should_panic(expected = "not a minter")]
    fn submit_pnl_rejects_strangers() {
        let mut contract = ctx("owner.testnet", 0, 1);
        register(&mut contract, "alpha-1", 10);
        as_pred("buyer.testnet", 0, 9);
        contract.submit_pnl("alpha-1".into(), "7d".into(), 1, 2, 3, 4, 5, "hash-1".into(), "in".into(), "ref".into());
    }

    #[test]
    fn submit_pnl_stores_relayed_record() {
        let mut contract = ctx("owner.testnet", 0, 1);
        register(&mut contract, "alpha-1", 10);
        as_pred("owner.testnet", ONE_NEAR, 2);
        contract.add_minter(account("minter.testnet"));
        as_pred("minter.testnet", 0, 9_000);
        contract.submit_pnl(
            "alpha-1".into(),
            "7d".into(),
            8_000_000,
            10,
            108098,
            2,
            1_008_000_000,
            "hash-1".into(),
            "abc".into(),
            "relayed-demo".into(),
        );
        let history = contract.get_pnl_history("alpha-1".into(), 0, 10);
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].pnl_usd_e6, 8_000_000);
        assert!(matches!(history[0].source, AttestSource::OutLayerHttps { .. }));
    }

    #[test]
    fn malformed_callback_does_not_store() {
        let mut contract = ctx("owner.testnet", 0, 1);
        register(&mut contract, "alpha-1", 10);
        as_pred("alpha.testnet", 0, 9);
        contract.on_pnl_result(
            "alpha-1".into(),
            "7d".into(),
            "in".into(),
            "hash-1".into(),
            Ok(Some("not-json".into())),
        );
        assert!(contract.get_pnl_history("alpha-1".into(), 0, 10).is_empty());
    }

    #[test]
    fn callback_stores_valid_metrics() {
        let mut contract = ctx("owner.testnet", 0, 1);
        register(&mut contract, "alpha-1", 10);
        as_pred("alpha.testnet", 0, 9);
        contract.on_pnl_result(
            "alpha-1".into(),
            "7d".into(),
            "in".into(),
            "hash-1".into(),
            Ok(Some(
                "{\"pnl_usd_e6\":8,\"max_dd_bps\":10,\"sharpe_e4\":1,\"n_trades\":2,\"equity_usd_e6\":9}".into(),
            )),
        );
        assert_eq!(contract.get_pnl_history("alpha-1".into(), 0, 10)[0].pnl_usd_e6, 8);
    }

    #[test]
    fn callback_rejects_rotated_code_hash() {
        let mut contract = ctx("owner.testnet", 0, 1);
        register(&mut contract, "alpha-1", 10);
        as_pred("owner.testnet", 0, 2);
        contract.set_attestor_code_hash("hash-2".into());
        as_pred("alpha.testnet", 0, 9);
        contract.on_pnl_result(
            "alpha-1".into(),
            "7d".into(),
            "in".into(),
            "hash-1".into(),
            Ok(Some(
                "{\"pnl_usd_e6\":8,\"max_dd_bps\":10,\"sharpe_e4\":1,\"n_trades\":2,\"equity_usd_e6\":9}".into(),
            )),
        );
        assert!(contract.get_pnl_history("alpha-1".into(), 0, 10).is_empty());
    }

    #[test]
    #[should_panic(expected = "private callback")]
    fn callback_is_private() {
        let mut contract = ctx("owner.testnet", 0, 1);
        as_pred("buyer.testnet", 0, 9);
        contract.on_pnl_result("alpha-1".into(), "7d".into(), "in".into(), "hash-1".into(), Ok(None));
    }

    fn ready_for_request(contract: &mut AlphaKeys) {
        register(contract, "alpha-1", 10);
        as_pred("owner.testnet", 0, 2);
        contract.set_attestor_code_hash("hash-1".into());
        contract.set_attestor_source(
            "https://test.fastfs.io/owner.testnet/outlayer.testnet/hash-1.wasm".into(),
            "pnl".into(),
            account("owner.testnet"),
        );
    }

    #[test]
    fn request_fits_in_300_tgas() {
        let mut contract = ctx("owner.testnet", 0, 1);
        ready_for_request(&mut contract);
        as_pred("owner.testnet", ONE_NEAR / 2, 1_731_200_000_000 * 1_000_000);
        let _ = contract.request_pnl_update("alpha-1".into(), "7d".into(), 1_731_200_000_000);
        assert!(env::used_gas() < env::prepaid_gas());
    }

    #[test]
    #[should_panic(expected = "now_ms must be within 10 minutes")]
    fn request_refuses_a_stale_window() {
        let mut contract = ctx("owner.testnet", 0, 1);
        ready_for_request(&mut contract);
        as_pred("owner.testnet", ONE_NEAR / 2, 1_731_200_000_000 * 1_000_000);
        let _ = contract.request_pnl_update("alpha-1".into(), "7d".into(), 1_731_200_000_000 - 86_400_000);
    }

    #[test]
    #[should_panic(expected = "attach at least 200 Tgas")]
    fn request_refuses_too_little_gas() {
        let mut contract = ctx("owner.testnet", 0, 1);
        ready_for_request(&mut contract);
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(account("alpha.testnet"))
            .signer_account_id(account("owner.testnet"))
            .predecessor_account_id(account("owner.testnet"))
            .attached_deposit(NearToken::from_yoctonear(ONE_NEAR / 2))
            .block_timestamp(1_731_200_000_000 * 1_000_000)
            .storage_usage(env::storage_usage())
            .prepaid_gas(Gas::from_tgas(100));
        testing_env!(builder.build());
        let _ = contract.request_pnl_update("alpha-1".into(), "7d".into(), 1_731_200_000_000);
    }

    #[test]
    fn callback_record_reads_as_outlayer_callback_json() {
        let mut contract = ctx("owner.testnet", 0, 1);
        register(&mut contract, "alpha-1", 10);
        as_pred("alpha.testnet", 0, 9);
        contract.on_pnl_result(
            "alpha-1".into(),
            "7d".into(),
            "in".into(),
            "hash-1".into(),
            Ok(Some(
                "{\"pnl_usd_e6\":-3,\"max_dd_bps\":500,\"sharpe_e4\":-1,\"n_trades\":12,\"equity_usd_e6\":9}\n".into(),
            )),
        );
        let row = &contract.get_pnl_history("alpha-1".into(), 0, 10)[0];
        let json = serde_json::to_value(row).unwrap();
        assert!(json["source"].get("OutLayerCallback").is_some());
        assert_eq!(json["code_hash"], "hash-1");
        assert_eq!(contract.get_config().2, "hash-1");
    }
}

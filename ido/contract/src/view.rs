use crate::*;
use near_sdk::{
    near_bindgen, AccountId,
};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::json_types::{U128};

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
pub struct Stats {
    pub owner_id: AccountId,
    pub total_ido: U128,
    pub staking_contract_id: AccountId,
    pub payment_token_id: AccountId,
    pub dangel_token_id: AccountId,
    pub ido_price: U128,
    pub total_purchased: U128,
    pub register_start_timestamp: TimestampSec,
    pub register_end_timestamp: TimestampSec,
    pub sale_start_timestamp: TimestampSec,
    pub sale_end_timestamp: TimestampSec,
    pub min_tier_cap_rate: u32,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
pub struct AccountOutput {
    pub purchased_amount: U128,
    pub tier: Tier,
}

#[near_bindgen]
impl Contract {
    pub fn get_stats(&self) -> Stats {
        Stats {
            owner_id: self.owner_id.clone(),
            total_ido: self.total_ido.into(),
            staking_contract_id: self.staking_contract_id.clone(),
            payment_token_id: self.payment_token_id.clone(),
            dangel_token_id: self.dangel_token_id.clone(),
            total_purchased: self.total_purchased.into(),
            ido_price: self.ido_price.into(),
            register_start_timestamp: self.register_start_timestamp,
            register_end_timestamp: self.register_end_timestamp,
            sale_start_timestamp: self.sale_start_timestamp,
            sale_end_timestamp: self.sale_end_timestamp,
            min_tier_cap_rate: self.min_tier_cap_rate,
        }
    }

    pub fn get_account(&self, account_id: AccountId) -> Option<AccountOutput> {
        self.accounts.get(&account_id.into())
        .map(|account| AccountOutput {
            purchased_amount: account.purchased_amount.into(),
            tier: account.tier,
        })
    }

    pub fn get_tier_allocation(&self, tier: &Tier) -> U128 {
        self.internal_get_tier_allocation(tier).into()
    }

    pub fn get_min_tier_cap(&self, tier: &Tier) -> U128 {
        (self.internal_get_tier_allocation(&tier) * self.min_tier_cap_rate as u128 / DENO as u128 ).into()
    }

    pub fn get_tier_configs(&self) -> TierConfigsType {
        TierConfig::get_default_tier_configs()
    }
}
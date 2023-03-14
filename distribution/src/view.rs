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
    pub token_account_id: AccountId,
    pub total_claimed: U128
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
pub struct AccountOutput {
    pub total_amount: U128,
    pub claimed_amount: U128,
    pub start_timestamp: u64,
    pub finish_timestamp: u64,
    pub duration: u64,
    pub releases_count: u64,
    pub is_revoked: bool,
    pub is_revocable: bool
}

#[near_bindgen]
impl Contract {
    pub fn get_stats(&self) -> Stats {
        Stats {
            owner_id: self.owner_id.clone(),
            token_account_id: self.token_account_id.clone(),
            total_claimed: self.total_claimed.into(),
        }
    }

    pub fn get_account(&self, account_id: AccountId) -> Option<AccountOutput> {
        self.accounts.get(&account_id.into())
        .map(|account| AccountOutput {
            total_amount: account.total_amount.into(),
            claimed_amount: account.claimed_amount.into(),
            start_timestamp: account.start_timestamp,
            finish_timestamp: account.finish_timestamp,
            duration: account.duration,
            releases_count: account.releases_count,
            is_revoked: account.is_revoked,
            is_revocable: account.is_revocable,
        })
    }

}
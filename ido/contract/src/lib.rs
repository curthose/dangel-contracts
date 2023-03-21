
mod util;
mod view;
mod tier;
pub use crate::tier::*;
pub use crate::util::*;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::collections::LookupMap;
use std::collections::HashMap;
use near_sdk::json_types::{U128};
use near_sdk::{
    env, near_bindgen, AccountId, Balance, BorshStorageKey, assert_one_yocto,
    Gas, PanicOnDefault, ext_contract, PromiseOrValue, log, Promise, Timestamp,
};

#[derive(BorshStorageKey, BorshSerialize)]
pub(crate) enum StorageKey {
    Accounts,
}
pub type TimestampSec = u64;

pub const STAKE_INFO_CB_GAS: Gas = Gas(5_000_000_000_000);
pub const STAKE_INFO_READ_GAS: Gas = Gas(5_000_000_000_000);
pub const ONE_YOCTO: Balance = 1;
pub const NO_DEPOSIT: Balance = 0;
pub const DENO: u32 = 1000;

#[derive(Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct AssetView {
    pub token_id: AccountId,
    pub balance: U128
}

#[derive(Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct AccountDetailedView {
    pub supplied: Vec<AssetView>,
}

#[ext_contract(ext_stakeinfo)]
pub trait ExtStakeInfo {
    fn get_account(account_id: AccountId);
}

#[ext_contract(ext_self)]
pub trait ExtContract {
    fn on_get_account(&mut self, account_id: AccountId, #[callback] account: Option<AccountDetailedView>);
}

#[ext_contract(ext_ft)]
pub trait FT {
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128>;
}


#[derive(BorshDeserialize, BorshSerialize)]
pub struct Account {
    pub purchased_amount: Balance,
    pub tier: Tier,
}


#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    pub owner_id: AccountId,
    pub accounts: LookupMap<AccountId, Account>,
    pub total_ido: Balance,
    pub staking_contract_id: AccountId,
    pub payment_token_id: AccountId,
    pub dangel_token_id: AccountId,
    pub ido_price: Balance,
    pub total_purchased: Balance,
    pub register_start_timestamp: TimestampSec,
    pub register_end_timestamp: TimestampSec,
    pub sale_start_timestamp: TimestampSec,
    pub sale_end_timestamp: TimestampSec,
    // RATE / 1000
    pub min_tier_cap_rate: u32,
    pub tier_configs: TierConfigsType,
}


#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(
        owner_id: AccountId,
        total_ido: U128,
        staking_contract_id: AccountId,
        payment_token_id: AccountId,
        dangel_token_id: AccountId,
        ido_price: U128,
        register_start_timestamp: TimestampSec,
        register_end_timestamp: TimestampSec,
        sale_start_timestamp: TimestampSec,
        sale_end_timestamp: TimestampSec,
        min_tier_cap_rate: u32,
    ) -> Self {
        Self {
            owner_id: owner_id.into(),
            total_ido: total_ido.into(),
            accounts: LookupMap::new(StorageKey::Accounts),
            staking_contract_id: staking_contract_id.into(),
            payment_token_id: payment_token_id.into(),
            dangel_token_id: dangel_token_id.into(),
            ido_price: ido_price.into(),
            total_purchased: 0,
            register_start_timestamp: register_start_timestamp,
            register_end_timestamp: register_end_timestamp,
            sale_start_timestamp: sale_start_timestamp,
            sale_end_timestamp: sale_end_timestamp,
            min_tier_cap_rate: min_tier_cap_rate,
            tier_configs: TierConfig::get_default_tier_configs(),
        }
    }
    

    #[payable]
    pub fn register(&mut self) -> Promise {
        assert_one_yocto();
        let current_timestamp = nano_to_sec(env::block_timestamp());
        assert!(current_timestamp > self.register_start_timestamp && current_timestamp < self.register_end_timestamp , "Register not open");
        let account_id = env::predecessor_account_id();
        assert_eq!(self.accounts.get(&account_id).expect("Account does not exist").tier,Tier::Tier0,"Account already registered");

        ext_stakeinfo::ext(self.staking_contract_id.clone())
        .with_static_gas(STAKE_INFO_READ_GAS)
        .get_account(account_id.clone())
        .then(ext_self::ext(env::current_account_id())
        .with_static_gas(STAKE_INFO_CB_GAS)
        .on_get_account(account_id.clone()))
    }

    fn internal_get_tier_allocation(&self, tier: &Tier) -> u128 {
        let configs = self.tier_configs.iter().map(|a| (*a.1)).collect::<Vec<TierConfig>>();
        let acc_tier= self.tier_configs.get(tier).unwrap();
        let mut total_weight : u128 = 0;
        for config in configs {
            total_weight += (config.number_of_participants * config.pool_weight) as u128;
        }
        (self.total_ido / total_weight) * acc_tier.pool_weight as u128
    }

    fn internal_purchase(&mut self, account_id: &AccountId, purchase_amount: Balance) {
        let current_timestamp = nano_to_sec(env::block_timestamp());
        assert!(current_timestamp > self.sale_start_timestamp && current_timestamp < self.sale_end_timestamp , "Sale not open");
        let mut account = self.accounts.get(&account_id).expect("Account is not whitelisted");
        let max_tier_cap = self.internal_get_tier_allocation(&account.tier);
        let min_tier_cap = max_tier_cap * (self.min_tier_cap_rate / DENO) as u128;
        let amount = (purchase_amount * u128:: pow(10,18)) / self.ido_price;
        assert!(account.purchased_amount + amount > min_tier_cap, "Amount is lower than minimum tier cap");
        assert!(account.purchased_amount + amount < max_tier_cap, "Amount is greater than maximum tier cap");

        account.purchased_amount += amount;
        self.total_purchased += amount;

        self.accounts.insert(&account_id, &account);
    }
 
    // View func get storage balance, return 0 if account need deposit to interact
    pub fn storage_balance_of(&self, account_id: AccountId) -> U128 {
        let account: Option<Account> = self.accounts.get(&account_id);
        if account.is_some() {
            U128(1)
        } else {
            U128(0)
        }
    }

    #[payable]
    pub fn storage_deposit(&mut self, account_id: Option<AccountId>) {
        assert_at_least_one_yocto();
        let account = account_id.unwrap_or_else(|| env::predecessor_account_id());

        let acc: Option<Account> = self.accounts.get(&account);
        if acc.is_some() {
            refund_deposit(0);
        } else {
            let before_storage_usage = env::storage_usage();
            let new_account = Account {
                purchased_amount: 0,
                tier: Tier::Tier0,
            };
            self.accounts.insert(&account, &new_account);
            let after_storage_usage = env::storage_usage();

            refund_deposit(after_storage_usage - before_storage_usage);
        }
    }

    pub fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        assert_eq!(env::predecessor_account_id(),self.payment_token_id,"Wrong Token!");
        let amount: u128 = amount.into();
        if msg.is_empty() {
            self.internal_purchase(&sender_id, amount.into());
        }
        PromiseOrValue::Value(U128(0))
    }

    #[private]
    pub fn on_get_account(&mut self, account_id: AccountId, #[callback] account_stake: AccountDetailedView) -> U128 {
        assert_eq!(account_stake.supplied[0].token_id,self.dangel_token_id,"Wrong Token");
        let stake_amount = account_stake.supplied[0].balance.into();
       let account_tier = self.internal_get_tier(stake_amount);
       assert_ne!(account_tier,Tier::Tier0,"Insufficient Stake Amount");
       let mut account = self.accounts.get(&account_id).unwrap();
       account.tier = account_tier;
       self.accounts.insert(&account_id,&account);

       let mut tier_info  = self.tier_configs.get(&account_tier).map(|&cfg| cfg).unwrap();
       tier_info.number_of_participants += 1;
       self.tier_configs.insert(account_tier, tier_info);
       U128(stake_amount)
    }

}


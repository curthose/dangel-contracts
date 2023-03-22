use near_contract_standards::fungible_token::core::ext_ft_core;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::json_types::{U128};
use near_sdk::{
    env, near_bindgen, AccountId, Balance, BorshStorageKey,
    Gas, PanicOnDefault, log, assert_one_yocto, Promise, PromiseResult, Timestamp,
};
mod view;

#[derive(BorshStorageKey, BorshSerialize)]
pub(crate) enum StorageKey {
    Accounts,
}
pub type TimestampSec = u64;

pub const GAS_FOR_FT_TRANSFER: Gas = Gas(3_000_000_000_000);
pub const GAS_FOR_CB_TRANSFER: Gas = Gas(2_000_000_000_000);
pub const ONE_YOCTO: Balance = 1;
pub const NO_DEPOSIT: Balance = 0;

/// Contains information about vesting schedule.
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Account {
    /// Total amount of tokens to be released at the end of the vesting
    pub total_amount: Balance,
    /// Tracks claimed amount of tokens for each account.
    pub claimed_amount: Balance,
    /// Start timestamp after cliff for vesting in seconds.
    pub start_timestamp: TimestampSec,
    /// Finish timestamp at the end of the vesting.
    pub finish_timestamp: TimestampSec,
    /// Vesting duration in seconds for each release.
    pub duration: TimestampSec,
    /// Release number in the vesting.
    pub releases_count: u64,
    /// true if this vesting schedule is revoked.
    pub is_revoked: bool,
    ///  Whether the vesting is revocable or not.
    pub is_revocable: bool,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    /// Owner Account ID
    pub owner_id: AccountId,
    /// Mapping for all beneficiary account IDs to vesting schedules.
    pub accounts: LookupMap<AccountId, Account>,
    /// dAngel token ID - token.dangelfund.near
    pub token_account_id: AccountId,
    /// Total claimed amount
    pub total_claimed: Balance,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(
        owner_id: AccountId,
        token_account_id: AccountId,
    ) -> Self {
        Self {
            owner_id: owner_id.into(),
            accounts: LookupMap::new(StorageKey::Accounts),
            token_account_id: token_account_id.into(),
            total_claimed: 0,
        }
    }
    
    /// Transfers vested tokens to beneficiary.
    #[payable]
    pub fn claim(&mut self) -> Promise {
        assert_one_yocto();
        let account_id = env::predecessor_account_id();
        assert!(self.accounts.contains_key(&account_id), "Caller not a beneficiary!");
        let mut account = self.accounts.get(&account_id).unwrap();
        assert!(!account.is_revoked, "Caller has revoked!");
        let claimable_amount = self.calculate_vested_amount(&account_id).checked_sub(account.claimed_amount).expect("ERR_INTEGER_OVERFLOW");
        assert!(claimable_amount > 0, "No claimable amount!");
        account.claimed_amount += claimable_amount;
        self.total_claimed += claimable_amount;
        self.accounts.insert(&account_id, &account);
        ext_ft_core::ext(self.token_account_id.clone())
            .with_attached_deposit(ONE_YOCTO)
            .with_static_gas(GAS_FOR_FT_TRANSFER)
            .ft_transfer(
                account_id.clone(),
                U128(claimable_amount),
                None
            ).then(Self::ext(env::current_account_id())
            .with_static_gas(GAS_FOR_CB_TRANSFER)
            .with_attached_deposit(0)
            .callback_claim_transfer(account_id.clone(), U128(claimable_amount)),)
    }

    /// Allows the owner to revoke the vesting. Rest are returned to the owner. 
    #[payable]
    pub fn revoke(&mut self, account_id: AccountId) -> Promise {
        assert_eq!(self.owner_id, env::predecessor_account_id(), "ERR_NOT_OWNER");
        assert_one_yocto();
        assert!(self.accounts.contains_key(&account_id), "Caller not a beneficiary!");
        let mut account = self.accounts.get(&account_id).unwrap();
        assert!(account.is_revocable, "ERR_GRANT_NOT_REVOCABLE");
        assert!(!account.is_revoked, "ERR_ALREADY_REVOKED");

        let remaining_amount: u128 = account.total_amount.checked_sub(account.claimed_amount).expect("Integer underflow");

        account.is_revoked = true;
        self.accounts.insert(&account_id, &account);

        // transfer leftover to owner
        ext_ft_core::ext(self.token_account_id.clone())
        .with_attached_deposit(ONE_YOCTO)
        .with_static_gas(GAS_FOR_FT_TRANSFER)
        .ft_transfer(
            self.owner_id.clone(),
            U128(remaining_amount),
            None
        ).then(Self::ext(env::current_account_id())
        .with_static_gas(GAS_FOR_CB_TRANSFER)
        .with_attached_deposit(0)
        .callback_revoke_transfer(self.owner_id.clone(), U128(remaining_amount)),)
    }

    
    pub fn get_vested_amount(&self, account_id: AccountId) -> U128{
        assert!(self.accounts.contains_key(&account_id), "Caller not a beneficiary!");
        self.calculate_vested_amount(&account_id).into()
    }

    /// Creates a new vesting schedule for a beneficiary.
    #[payable]
    pub fn add_accounts(
        &mut self, 
        accounts: Vec<(AccountId, U128, u64, u64, u64, u64, bool)>,
    ) -> bool {
        assert_eq!(self.owner_id, env::predecessor_account_id(), "ERR_NOT_OWNER");
        assert_one_yocto();
        for (account_id, amount, start, cliff, duration, releases_count, is_revocable ) in accounts {
            assert!(cliff > 0 || releases_count > 0, "INVALID PARAMS");
            if !self.accounts.contains_key(&account_id){
                let account = Account {
                    total_amount: amount.into(),
                    claimed_amount: 0,
                    start_timestamp: start + cliff,
                    finish_timestamp: start + cliff + releases_count * duration,
                    duration: duration,
                    releases_count: releases_count,
                    is_revoked: false,
                    is_revocable: is_revocable,
                };
                self.accounts.insert(&account_id, &account);
            }
        } 
        true
    }

    /// Calculates the amount that has already vested.
    fn calculate_vested_amount(&self, account_id: &AccountId) -> u128 {
        let current_timestamp = nano_to_sec(env::block_timestamp());
        let account = self.accounts.get(&account_id).expect("No Vesting Found!");
        if current_timestamp < account.start_timestamp {
           0
        } else if current_timestamp >= account.finish_timestamp || account.is_revoked  {
           account.total_amount
        } else {
            let available_releases = (current_timestamp - account.start_timestamp) as u128 / account.duration as u128;
            let tokens_per_release = account.total_amount / account.releases_count as u128;
            let vested_amount = available_releases as u128 * tokens_per_release;
            vested_amount
        }
    }

    /// Callback for claim
    #[private]
    pub fn callback_claim_transfer(&mut self, account_id: AccountId, amount: U128) -> U128 {
        assert_eq!(env::promise_results_count(), 1, "ERR_TOO_MANY_RESULTS");
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_val) => {
                log!(format!(
                    "Account claim succeed, account is {}, amount is {}",
                    account_id,
                    amount.0
                ));
            },
            /// if transfer failed, restore the account data.
            PromiseResult::Failed => {
                let mut account = self
                .accounts
                .get(&account_id)
                .expect("The claim is not found");
                account.claimed_amount -= amount.0;
                self.total_claimed -= amount.0;
                self.accounts.insert(&account_id, &account);
            }
        }
        amount.into()
    }

    /// Callback for revoke
    pub fn callback_revoke_transfer(&mut self, account_id: AccountId, amount: U128) -> U128 {
        assert_eq!(env::promise_results_count(), 1, "ERR_TOO_MANY_RESULTS");
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_val) => {
                log!(format!(
                    "Account revoke succeed, account is {}, remaining amount is {}",
                    account_id,
                    amount.0
                ));
            },
            /// if transfer failed, restore the account data.
            PromiseResult::Failed => {
                let mut account = self
                .accounts
                .get(&account_id)
                .expect("The account is not found");
                account.is_revoked = false;
                self.accounts.insert(&account_id, &account);
            }
        }
        amount.into()
    }
}

pub fn nano_to_sec(nano: Timestamp) -> TimestampSec {
    nano as TimestampSec / 1_000_000_000 
}





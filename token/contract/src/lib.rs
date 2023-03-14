/*!
Fungible Token implementation with JSON serialization.
NOTES:
  - The maximum balance value is limited by U128 (2**128 - 1).
  - JSON calls should pass U128 as a base-10 string. E.g. "100".
  - The contract optimizes the inner trie structure by hashing account IDs. It will prevent some
    abuse of deep tries. Shouldn't be an issue, once NEAR clients implement full hashing of keys.
  - The contract tracks the change in storage before and after the call. If the storage increases,
    the contract requires the caller of the contract to attach enough deposit to the function call
    to cover the storage cost.
    This is done to prevent a denial of service attack on the contract by taking all available storage.
    If the storage decreases, the contract will issue a refund for the cost of the released storage.
    The unused tokens from the attached deposit are also refunded, so it's safe to
    attach more deposit than required.
  - To prevent the deployed contract from being modified or deleted, it should not have any access
    keys on its account.
*/
use near_contract_standards::fungible_token::metadata::{
    FungibleTokenMetadata, FungibleTokenMetadataProvider, FT_METADATA_SPEC,
};
use near_contract_standards::fungible_token::FungibleToken;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LazyOption;
use near_sdk::json_types::U128;
use near_sdk::{env, log, near_bindgen, AccountId, Balance, PanicOnDefault, PromiseOrValue};

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    token: FungibleToken,
    metadata: LazyOption<FungibleTokenMetadata>,
}

const SVG_DANGEL_ICON: &str = "data:image/svg+xml,%3Csvg id='l' xmlns='http://www.w3.org/2000/svg' xmlns:xlink='http://www.w3.org/1999/xlink' viewBox='0 0 288 288'%3E%3Cdefs%3E%3Cstyle%3E.cls-1%7Bfill:url(%23linear-gradient);%7D%3C/style%3E%3ClinearGradient id='linear-gradient' x1='56.19' y1='-431.8' x2='231.81' y2='-431.8' gradientTransform='translate(0 -287.8) scale(1 -1)' gradientUnits='userSpaceOnUse'%3E%3Cstop offset='0' stop-color='%23007ffb'/%3E%3Cstop offset='.29' stop-color='%236a3eb3'/%3E%3Cstop offset='.51' stop-color='%23e84dae'/%3E%3Cstop offset='.74' stop-color='%23ea3139'/%3E%3Cstop offset='1' stop-color='%23ffaf00'/%3E%3C/linearGradient%3E%3C/defs%3E%3Cpath id='l-2' class='cls-1' d='m106.33,213.16c-33.97-12.07-51.81-49.47-39.78-83.45,7.82-22.07,32.12-33.65,54.19-25.82l21.46,7.63c2.22,1.05,30.02,13.31,55.89,2.54,6.26-2.62,11.82-6.41,16.7-11.3l-25.06,70.65c-9.49,26.8-34.74,43.54-61.66,43.54-7.23-.04-14.57-1.25-21.74-3.79m95.67-142.06h1.58c1.13.04,2.26.16,3.39.28.73.08,1.49.12,2.22.21,1.25.16,2.5.45,3.75.73.61.12,1.25.21,1.86.36,1.86.45,3.72,1.01,5.56,1.65l3.96,1.41c-6.34,16.06-15.97,26.87-28.69,32.2-19.12,8.03-40.31,1.53-47.9-1.29.21-.45.36-.89.57-1.34.36-.85.77-1.65,1.17-2.46.45-.89.92-1.77,1.41-2.66.45-.77.89-1.53,1.37-2.3.52-.85,1.09-1.65,1.65-2.46.52-.73,1.01-1.41,1.58-2.1.61-.77,1.25-1.53,1.9-2.3.57-.64,1.13-1.29,1.74-1.93.68-.73,1.37-1.41,2.1-2.1.61-.61,1.25-1.17,1.9-1.74.73-.64,1.53-1.25,2.3-1.86.64-.52,1.34-1.05,2.02-1.53.8-.57,1.62-1.09,2.46-1.65.73-.45,1.41-.89,2.14-1.34.85-.49,1.74-.92,2.62-1.41.73-.36,1.49-.77,2.22-1.13.89-.4,1.81-.77,2.74-1.17.77-.33,1.53-.64,2.3-.89.92-.33,1.9-.61,2.87-.89.8-.24,1.58-.49,2.38-.68.97-.24,1.98-.45,2.99-.61.8-.16,1.62-.33,2.42-.45,1.01-.16,2.05-.24,3.11-.33.8-.08,1.62-.21,2.42-.24,1.05-.04,2.14-.04,3.18-.04h.12c.12.04.36.04.61.04m-.92-6.5c-1.17,0-2.3-.04-3.47.04-.92.04-1.86.16-2.83.28-1.09.12-2.22.21-3.31.36-.97.16-1.9.36-2.83.52-1.05.21-2.14.4-3.18.64-.92.24-1.86.52-2.78.8-1.01.33-2.02.61-3.03.92-.92.33-1.81.73-2.74,1.09-.97.4-1.93.77-2.9,1.21-.89.4-1.77.89-2.62,1.34-.92.49-1.86.97-2.78,1.49-.85.49-1.7,1.05-2.5,1.58-.89.57-1.77,1.13-2.62,1.74-.8.57-1.58,1.21-2.38,1.81-.8.64-1.65,1.29-2.42,1.98-.77.64-1.46,1.34-2.18,2.02-.77.73-1.53,1.46-2.26,2.22-.68.73-1.34,1.49-2.02,2.22-.68.8-1.37,1.62-2.02,2.46-.61.8-1.21,1.62-1.77,2.42-.61.89-1.21,1.77-1.81,2.71-.57.85-1.05,1.74-1.58,2.62-.52.97-1.05,1.93-1.58,2.9-.45.92-.89,1.86-1.34,2.78-.24.52-.49,1.01-.73,1.53l-18.6-6.57c-25.46-9.04-53.51,4.36-62.55,29.82-13.12,37.36,6.5,78.52,43.86,91.76,37.36,13.24,78.56-6.38,91.8-43.74l35.69-100.69c.61-1.7-.28-3.59-1.98-4.15l-7.11-2.5c-2.05-.73-4.12-1.34-6.22-1.86-.68-.16-1.41-.28-2.1-.4-1.37-.28-2.74-.61-4.15-.8-.85-.12-1.7-.16-2.54-.24-1.21-.12-2.46-.28-3.67-.33h-.89c-.64-.04-1.25,0-1.9,0'/%3E%3C/svg%3E";
const TOTAL_SUPPLY: Balance = 250_000_000_000_000_000_000_000_000;

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new_dangel_meta(owner_id: AccountId) -> Self {
        Self::new(
            owner_id,
            U128(TOTAL_SUPPLY),
            FungibleTokenMetadata {
                spec: FT_METADATA_SPEC.to_string(),
                name: "dAngel".to_string(),
                symbol: "DANGEL".to_string(),
                icon: Some(SVG_DANGEL_ICON.to_string()),
                reference: None,
                reference_hash: None,
                decimals: 18,
            },
        )
    }

    #[init]
    pub fn new(
        owner_id: AccountId,
        total_supply: U128,
        metadata: FungibleTokenMetadata,
    ) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        metadata.assert_valid();
        let mut this = Self {
            token: FungibleToken::new(b"a".to_vec()),
            metadata: LazyOption::new(b"m".to_vec(), Some(&metadata)),
        };
        this.token.internal_register_account(&owner_id);
        this.token.internal_deposit(&owner_id, total_supply.into());
        near_contract_standards::fungible_token::events::FtMint {
            owner_id: &owner_id,
            amount: &total_supply,
            memo: Some("Initial tokens supply is minted"),
        }
        .emit();
        this
    }

    fn on_account_closed(&mut self, account_id: AccountId, balance: Balance) {
        log!("Closed @{} with {}", account_id, balance);
    }

    fn on_tokens_burned(&mut self, account_id: AccountId, amount: Balance) {
        log!("Account @{} burned {}", account_id, amount);
    }
}

near_contract_standards::impl_fungible_token_core!(Contract, token, on_tokens_burned);
near_contract_standards::impl_fungible_token_storage!(Contract, token, on_account_closed);

#[near_bindgen]
impl FungibleTokenMetadataProvider for Contract {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        self.metadata.get().unwrap()
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::MockedBlockchain;
    use near_sdk::{testing_env, Balance};

    use super::*;

    const TOTAL_SUPPLY: Balance = 1_000_000_000_000_000;

    fn get_context(predecessor_account_id: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(accounts(0))
            .signer_account_id(predecessor_account_id.clone())
            .predecessor_account_id(predecessor_account_id);
        builder
    }

    #[test]
    fn test_new() {
        let mut context = get_context(accounts(1));
        testing_env!(context.build());
        let contract = Contract::new_default_meta(accounts(1).into(), TOTAL_SUPPLY.into());
        testing_env!(context.is_view(true).build());
        assert_eq!(contract.ft_total_supply().0, TOTAL_SUPPLY);
        assert_eq!(contract.ft_balance_of(accounts(1)).0, TOTAL_SUPPLY);
    }

    #[test]
    #[should_panic(expected = "The contract is not initialized")]
    fn test_default() {
        let context = get_context(accounts(1));
        testing_env!(context.build());
        let _contract = Contract::default();
    }

    #[test]
    fn test_transfer() {
        let mut context = get_context(accounts(2));
        testing_env!(context.build());
        let mut contract = Contract::new_default_meta(accounts(2).into(), TOTAL_SUPPLY.into());
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(contract.storage_balance_bounds().min.into())
            .predecessor_account_id(accounts(1))
            .build());
        // Paying for account registration, aka storage deposit
        contract.storage_deposit(None, None);

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(1)
            .predecessor_account_id(accounts(2))
            .build());
        let transfer_amount = TOTAL_SUPPLY / 3;
        contract.ft_transfer(accounts(1), transfer_amount.into(), None);

        testing_env!(context
            .storage_usage(env::storage_usage())
            .account_balance(env::account_balance())
            .is_view(true)
            .attached_deposit(0)
            .build());
        assert_eq!(contract.ft_balance_of(accounts(2)).0, (TOTAL_SUPPLY - transfer_amount));
        assert_eq!(contract.ft_balance_of(accounts(1)).0, transfer_amount);
    }
}
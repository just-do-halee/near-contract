#![cfg(feature = "ft")]
#![allow(dead_code)]
/*!
Fungible Token implementation with JSON serialization.

# NOTES:
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

# EXAMPLE:
```
mod cmn;
use cmn::*;

#[near_bindgen]
#[derive(PanicOnDefault, BorshDeserialize, BorshSerialize)]
pub struct Contract {
    ft: ft::FungibleToken,
}

const DATA_IMAGE_SVG_NEAR_ICON: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 288 288'%3E%3Cg id='l' data-name='l'%3E%3Cpath d='M187.58,79.81l-30.1,44.69a3.2,3.2,0,0,0,4.75,4.2L191.86,103a1.2,1.2,0,0,1,2,.91v80.46a1.2,1.2,0,0,1-2.12.77L102.18,77.93A15.35,15.35,0,0,0,90.47,72.5H87.34A15.34,15.34,0,0,0,72,87.84V201.16A15.34,15.34,0,0,0,87.34,216.5h0a15.35,15.35,0,0,0,13.08-7.31l30.1-44.69a3.2,3.2,0,0,0-4.75-4.2L96.14,186a1.2,1.2,0,0,1-2-.91V104.61a1.2,1.2,0,0,1,2.12-.77l89.55,107.23a15.35,15.35,0,0,0,11.71,5.43h3.13A15.34,15.34,0,0,0,216,201.16V87.84A15.34,15.34,0,0,0,200.66,72.5h0A15.35,15.35,0,0,0,187.58,79.81Z'/%3E%3C/g%3E%3C/svg%3E";

ft::impl_fungible_token_contract!(Contract, ft);

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(owner_id: AccountId, total_supply: U128) -> Self {
        require_init!();
        Self {
            ft: ft::FungibleToken::new(
                owner_id,
                total_supply,
                ft::Metadata {
                    spec: ft::METADATA_SPEC.to_string(),
                    name: "Example NEAR FT".to_string(),
                    symbol: "EXAMPLE".to_string(),
                    icon: Some(DATA_IMAGE_SVG_NEAR_ICON.to_string()),
                    reference: None,
                    reference_hash: None,
                    decimals: 24,
                },
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::test_utils::*;
    use super::*;

    use ft::core::FungibleTokenCore;

    const TOTAL_SUPPLY: Balance = 1_000_000_000_000_000;

    fn get_vm(predecessor: AccountId) -> VMContextBuilder {
        vm!(predecessor)
            .current_account_id("current".parse().unwrap())
            .clone()
    }

    #[test]
    fn test_new() {
        let mut vm = get_vm(accounts(0));
        run_vm(&vm);

        let contract = Contract::new(accounts(1), TOTAL_SUPPLY.into());

        run_vm(vm.is_view(true));
        assert_eq!(contract.ft_total_supply().0, TOTAL_SUPPLY);
        assert_eq!(contract.ft_balance_of(accounts(1)).0, TOTAL_SUPPLY);
    }

    #[test]
    #[should_panic(expected = "The contract is not initialized")]
    fn test_default() {
        run_vm(get_vm(accounts(1)));
        _ = Contract::default();
    }

    #[test]
    fn test_transfer() {
        let mut vm = get_vm(accounts(2));
        run_vm(&vm);

        let mut contract = Contract::new(accounts(2), TOTAL_SUPPLY.into());

        run_vm(
            vm.storage_usage(env::storage_usage())
                .attached_deposit(contract.storage_balance_bounds().min.into())
                .predecessor_account_id(accounts(1)),
        );
        // Paying for account registration, aka storage deposit
        contract.storage_deposit(None, None);

        run_vm(
            vm.storage_usage(env::storage_usage())
                .attached_deposit(1)
                .predecessor_account_id(accounts(2)),
        );
        let transfer_amount = TOTAL_SUPPLY / 3;
        contract.ft_transfer(accounts(1), transfer_amount.into(), None);

        run_vm(
            vm.storage_usage(env::storage_usage())
                .account_balance(env::account_balance())
                .is_view(true)
                .attached_deposit(0),
        );
        assert_eq!(
            contract.ft_balance_of(accounts(2)).0,
            (TOTAL_SUPPLY - transfer_amount)
        );
        assert_eq!(contract.ft_balance_of(accounts(1)).0, transfer_amount);
    }
}
```
*/

use super::*;

pub use near_contract_standards::fungible_token::{
    self,
    metadata::{self, FungibleTokenMetadata as Metadata, FT_METADATA_SPEC as METADATA_SPEC},
    FungibleToken as Token, *,
};

mod for_rust_core {
    use super::{borsh, BorshSerialize, BorshStorageKey};
    #[repr(u8)]
    #[derive(BorshSerialize, BorshStorageKey)]
    pub enum StorageKey {
        Token = 0,
        Metadata = 1,
    }
}
pub use for_rust_core::*;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct FungibleToken {
    pub token: Token,
    pub metadata: LazyOption<Metadata>,
}
impl FungibleToken {
    pub fn new(owner_id: AccountId, total_supply: U128, metadata: Metadata) -> Self {
        metadata.assert_valid();
        let mut this = Self {
            token: Token::new(StorageKey::Token),
            metadata: LazyOption::new(StorageKey::Metadata, Some(&metadata)),
        };
        this.token.internal_register_account(&owner_id);
        this.token.internal_deposit(&owner_id, total_supply.into());

        events::FtMint {
            owner_id: &owner_id,
            amount: &total_supply,
            memo: Some("Initial tokens supply is minted"),
        }
        .emit();
        this
    }
}

#[macro_export]
macro_rules! impl_fungible_token_contract {
        (@IMPL_CORE $contract:ident, $ft:ident) => {
            #[near_bindgen]
            impl $crate::ft::core::FungibleTokenCore for $contract {
                #[payable]
                fn ft_transfer(
                    &mut self,
                    receiver_id: AccountId,
                    amount: U128,
                    memo: Option<String>,
                ) {
                    self.$ft.token.ft_transfer(receiver_id, amount, memo)
                }

                #[payable]
                fn ft_transfer_call(
                    &mut self,
                    receiver_id: AccountId,
                    amount: U128,
                    memo: Option<String>,
                    msg: String,
                ) -> PromiseOrValue<U128> {
                    self.$ft.token.ft_transfer_call(receiver_id, amount, memo, msg)
                }

                fn ft_total_supply(&self) -> U128 {
                    self.$ft.token.ft_total_supply()
                }

                fn ft_balance_of(&self, account_id: AccountId) -> U128 {
                    self.$ft.token.ft_balance_of(account_id)
                }
            }

            #[near_bindgen]
            impl $crate::ft::resolver::FungibleTokenResolver for $contract {
                #[private]
                fn ft_resolve_transfer(
                    &mut self,
                    sender_id: AccountId,
                    receiver_id: AccountId,
                    amount: U128,
                ) -> U128 {
                    let (used_amount, burned_amount) =
                        self.$ft.token.internal_ft_resolve_transfer(&sender_id, receiver_id, amount);
                    if burned_amount > 0 {
                        self.on_tokens_burned(sender_id, burned_amount);
                    }
                    used_amount.into()
                }
            }
        };
        (@IMPL_STORAGE $contract:ident, $ft:ident) => {
            #[near_bindgen]
            impl StorageManagement for $contract {
                #[payable]
                fn storage_deposit(
                    &mut self,
                    account_id: Option<AccountId>,
                    registration_only: Option<bool>,
                ) -> StorageBalance {
                    self.$ft.token.storage_deposit(account_id, registration_only)
                }

                #[payable]
                fn storage_withdraw(&mut self, amount: Option<U128>) -> StorageBalance {
                    self.$ft.token.storage_withdraw(amount)
                }

                #[payable]
                fn storage_unregister(&mut self, force: Option<bool>) -> bool {
                    #[allow(unused_variables)]
                    if let Some((account_id, balance)) = self.$ft.token.internal_storage_unregister(force) {
                        self.on_account_closed(account_id, balance);
                        true
                    } else {
                        false
                    }
                }

                fn storage_balance_bounds(&self) -> StorageBalanceBounds {
                    self.$ft.token.storage_balance_bounds()
                }

                fn storage_balance_of(&self, account_id: AccountId) -> Option<StorageBalance> {
                    self.$ft.token.storage_balance_of(account_id)
                }
            }
        };
        ($contract:ident, $ft:ident) => {
            impl $contract {
                fn on_account_closed(&mut self, account_id: AccountId, balance: Balance) {
                    log!("Closed @{} with {}", account_id, balance);
                }

                fn on_tokens_burned(&mut self, account_id: AccountId, amount: Balance) {
                    log!("Account @{} burned {}", account_id, amount);
                }
            }
            impl_fungible_token_contract!(@IMPL_CORE $contract, $ft);
            impl_fungible_token_contract!(@IMPL_STORAGE $contract, $ft);
            #[near_bindgen]
            impl $crate::ft::metadata::FungibleTokenMetadataProvider for $contract {
                fn ft_metadata(&self) -> $crate::ft::Metadata {
                    self.$ft.metadata.get().unwrap()
                }
            }
        };
    }
pub use impl_fungible_token_contract;

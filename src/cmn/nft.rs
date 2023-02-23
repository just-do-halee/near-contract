#![cfg(feature = "nft")]
#![allow(dead_code)]
/*!
Non-Fungible Token implementation with JSON serialization.

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
    nft: nft::NonFungibleToken,
}

const DATA_IMAGE_SVG_NEAR_ICON: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 288 288'%3E%3Cg id='l' data-name='l'%3E%3Cpath d='M187.58,79.81l-30.1,44.69a3.2,3.2,0,0,0,4.75,4.2L191.86,103a1.2,1.2,0,0,1,2,.91v80.46a1.2,1.2,0,0,1-2.12.77L102.18,77.93A15.35,15.35,0,0,0,90.47,72.5H87.34A15.34,15.34,0,0,0,72,87.84V201.16A15.34,15.34,0,0,0,87.34,216.5h0a15.35,15.35,0,0,0,13.08-7.31l30.1-44.69a3.2,3.2,0,0,0-4.75-4.2L96.14,186a1.2,1.2,0,0,1-2-.91V104.61a1.2,1.2,0,0,1,2.12-.77l89.55,107.23a15.35,15.35,0,0,0,11.71,5.43h3.13A15.34,15.34,0,0,0,216,201.16V87.84A15.34,15.34,0,0,0,200.66,72.5h0A15.35,15.35,0,0,0,187.58,79.81Z'/%3E%3C/g%3E%3C/svg%3E";

nft::impl_non_fungible_token_contract!(Contract, nft);

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(owner_id: AccountId) -> Self {
        require_init!();
        Self {
            nft: nft::NonFungibleToken::new(
                owner_id,
                nft::Metadata {
                    spec: nft::METADATA_SPEC.to_string(),
                    name: "Example NEAR NFT".to_string(),
                    symbol: "EXAMPLE".to_string(),
                    icon: Some(DATA_IMAGE_SVG_NEAR_ICON.to_string()),
                    base_uri: None,
                    reference: None,
                    reference_hash: None,
                },
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::test_utils::*;
    use super::*;

    use nft::{approval::NonFungibleTokenApproval, core::NonFungibleTokenCore};
    use std::collections::HashMap;

    const MINT_STORAGE_COST: u128 = 5870000000000000000000;

    fn get_vm(predecessor: AccountId) -> VMContextBuilder {
        vm!(predecessor)
            .current_account_id("current".parse().unwrap())
            .clone()
    }

    fn sample_token_metadata() -> nft::TokenMetadata {
        nft::TokenMetadata {
            title: Some("Olympus Mons".into()),
            description: Some("The tallest mountain in the charted solar system".into()),
            media: None,
            media_hash: None,
            copies: Some(1u64),
            issued_at: None,
            expires_at: None,
            starts_at: None,
            updated_at: None,
            extra: None,
            reference: None,
            reference_hash: None,
        }
    }

    #[test]
    fn test_new() {
        let mut vm = get_vm(accounts(0));
        run_vm(&vm);

        let contract = Contract::new(accounts(1));

        run_vm(vm.is_view(true));
        assert_eq!(contract.nft_token("1".to_string()), None);
    }

    #[test]
    #[should_panic(expected = "The contract is not initialized")]
    fn test_default() {
        run_vm(get_vm(accounts(1)));
        _ = Contract::default();
    }

    #[test]
    fn test_mint() {
        let mut vm = get_vm(accounts(0));
        run_vm(&vm);
        let mut contract = Contract::new(accounts(0));

        run_vm(
            vm.storage_usage(env::storage_usage())
                .attached_deposit(MINT_STORAGE_COST)
                .predecessor_account_id(accounts(0)),
        );

        let token_id = "0".to_string();
        let token = contract.nft_mint(token_id.clone(), accounts(0), sample_token_metadata());
        assert_eq!(token.token_id, token_id);
        assert_eq!(token.owner_id.to_string(), accounts(0).to_string());
        assert_eq!(token.metadata.unwrap(), sample_token_metadata());
        assert_eq!(token.approved_account_ids.unwrap(), HashMap::new());
    }

    #[test]
    fn test_transfer() {
        let mut vm = get_vm(accounts(0));
        run_vm(&vm);
        let mut contract = Contract::new(accounts(0));

        run_vm(
            vm.storage_usage(env::storage_usage())
                .attached_deposit(MINT_STORAGE_COST)
                .predecessor_account_id(accounts(0)),
        );
        let token_id = "0".to_string();
        contract.nft_mint(token_id.clone(), accounts(0), sample_token_metadata());

        run_vm(
            vm.storage_usage(env::storage_usage())
                .attached_deposit(1)
                .predecessor_account_id(accounts(0)),
        );
        contract.nft_transfer(accounts(1), token_id.clone(), None, None);

        run_vm(
            vm.storage_usage(env::storage_usage())
                .account_balance(env::account_balance())
                .is_view(true)
                .attached_deposit(0),
        );
        if let Some(token) = contract.nft_token(token_id.clone()) {
            assert_eq!(token.token_id, token_id);
            assert_eq!(token.owner_id.to_string(), accounts(1).to_string());
            assert_eq!(token.metadata.unwrap(), sample_token_metadata());
            assert_eq!(token.approved_account_ids.unwrap(), HashMap::new());
        } else {
            panic!("token not correctly created, or not found by nft_token");
        }
    }

    #[test]
    fn test_approve() {
        let mut vm = get_vm(accounts(0));
        run_vm(&vm);
        let mut contract = Contract::new(accounts(0));

        run_vm(
            vm.storage_usage(env::storage_usage())
                .attached_deposit(MINT_STORAGE_COST)
                .predecessor_account_id(accounts(0)),
        );
        let token_id = "0".to_string();
        contract.nft_mint(token_id.clone(), accounts(0), sample_token_metadata());

        // alice approves bob
        run_vm(
            vm.storage_usage(env::storage_usage())
                .attached_deposit(150000000000000000000)
                .predecessor_account_id(accounts(0)),
        );
        contract.nft_approve(token_id.clone(), accounts(1), None);

        run_vm(
            vm.storage_usage(env::storage_usage())
                .account_balance(env::account_balance())
                .is_view(true)
                .attached_deposit(0),
        );
        assert!(contract.nft_is_approved(token_id, accounts(1), Some(1)));
    }

    #[test]
    fn test_revoke() {
        let mut vm = get_vm(accounts(0));
        run_vm(&vm);
        let mut contract = Contract::new(accounts(0));

        run_vm(
            vm.storage_usage(env::storage_usage())
                .attached_deposit(MINT_STORAGE_COST)
                .predecessor_account_id(accounts(0)),
        );
        let token_id = "0".to_string();
        contract.nft_mint(token_id.clone(), accounts(0), sample_token_metadata());

        // alice approves bob
        run_vm(
            vm.storage_usage(env::storage_usage())
                .attached_deposit(150000000000000000000)
                .predecessor_account_id(accounts(0)),
        );
        contract.nft_approve(token_id.clone(), accounts(1), None);

        // alice revokes bob
        run_vm(
            vm.storage_usage(env::storage_usage())
                .attached_deposit(1)
                .predecessor_account_id(accounts(0)),
        );
        contract.nft_revoke(token_id.clone(), accounts(1));
        run_vm(
            vm.storage_usage(env::storage_usage())
                .account_balance(env::account_balance())
                .is_view(true)
                .attached_deposit(0),
        );
        assert!(!contract.nft_is_approved(token_id, accounts(1), None));
    }

    #[test]
    fn test_revoke_all() {
        let mut vm = get_vm(accounts(0));
        run_vm(&vm);
        let mut contract = Contract::new(accounts(0));

        run_vm(
            vm.storage_usage(env::storage_usage())
                .attached_deposit(MINT_STORAGE_COST)
                .predecessor_account_id(accounts(0)),
        );
        let token_id = "0".to_string();
        contract.nft_mint(token_id.clone(), accounts(0), sample_token_metadata());

        // alice approves bob
        run_vm(
            vm.storage_usage(env::storage_usage())
                .attached_deposit(150000000000000000000)
                .predecessor_account_id(accounts(0)),
        );
        contract.nft_approve(token_id.clone(), accounts(1), None);

        // alice revokes bob
        run_vm(
            vm.storage_usage(env::storage_usage())
                .attached_deposit(1)
                .predecessor_account_id(accounts(0)),
        );
        contract.nft_revoke_all(token_id.clone());
        run_vm(
            vm.storage_usage(env::storage_usage())
                .account_balance(env::account_balance())
                .is_view(true)
                .attached_deposit(0),
        );
        assert!(!contract.nft_is_approved(token_id, accounts(1), Some(1)));
    }
}
```
*/

use super::*;

pub use near_contract_standards::non_fungible_token::{
    self,
    metadata::{
        self, NFTContractMetadata as Metadata, TokenMetadata, NFT_METADATA_SPEC as METADATA_SPEC,
    },
    NonFungibleToken as NFToken, *,
};

mod for_rust_core {
    use super::{borsh, BorshSerialize, BorshStorageKey};
    #[repr(u8)]
    #[derive(BorshSerialize, BorshStorageKey)]
    pub enum StorageKey {
        TokenMetadata = 1,
        Token = 2,
        Metadata = 3,
        Enumeration = 4,
        Approval = 5,
    }
}
pub use for_rust_core::*;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct NonFungibleToken {
    pub token: NFToken,
    pub metadata: LazyOption<Metadata>,
}
impl NonFungibleToken {
    pub fn new(owner_id: AccountId, metadata: Metadata) -> Self {
        metadata.assert_valid();
        Self {
            token: NFToken::new(
                // owner_by_id_prefix: Q,
                // owner_id: AccountId,
                // token_metadata_prefix: Option<R>,
                // enumeration_prefix: Option<S>,
                // approval_prefix: Option<T>,
                StorageKey::Token,
                owner_id,
                Some(StorageKey::TokenMetadata),
                Some(StorageKey::Enumeration),
                Some(StorageKey::Approval),
            ),
            metadata: LazyOption::new(StorageKey::Metadata, Some(&metadata)),
        }
    }
}

#[macro_export]
macro_rules! impl_non_fungible_token_contract {
        (@IMPL_CORE $contract:ident, $nft:ident) => {
            #[near_bindgen]
            impl $crate::nft::core::NonFungibleTokenCore for $contract {
                #[payable]
                fn nft_transfer(
                    &mut self,
                    receiver_id: AccountId,
                    token_id: $crate::nft::TokenId,
                    approval_id: Option<u64>,
                    memo: Option<String>,
                ) {
                    self.$nft.token.nft_transfer(receiver_id, token_id, approval_id, memo)
                }

                #[payable]
                fn nft_transfer_call(
                    &mut self,
                    receiver_id: AccountId,
                    token_id: $crate::nft::TokenId,
                    approval_id: Option<u64>,
                    memo: Option<String>,
                    msg: String,
                ) -> PromiseOrValue<bool> {
                    self.$nft.token.nft_transfer_call(receiver_id, token_id, approval_id, memo, msg)
                }

                fn nft_token(&self, token_id: $crate::nft::TokenId) -> Option<$crate::nft::Token> {
                    self.$nft.token.nft_token(token_id)
                }
            }

            #[near_bindgen]
            impl $crate::nft::core::NonFungibleTokenResolver for $contract {
                #[private]
                fn nft_resolve_transfer(
                    &mut self,
                    previous_owner_id: AccountId,
                    receiver_id: AccountId,
                    token_id: $crate::nft::TokenId,
                    approved_account_ids: Option<std::collections::HashMap<AccountId, u64>>,
                ) -> bool {
                    self.$nft.token.nft_resolve_transfer(
                        previous_owner_id,
                        receiver_id,
                        token_id,
                        approved_account_ids,
                    )
                }
            }
        };
        (@IMPL_APPROVAL $contract:ident, $nft:ident) => {
            #[near_bindgen]
            impl $crate::nft::approval::NonFungibleTokenApproval for $contract {
                #[payable]
                fn nft_approve(
                    &mut self,
                    token_id: $crate::nft::TokenId,
                    account_id: AccountId,
                    msg: Option<String>,
                ) -> Option<Promise> {
                    self.$nft.token.nft_approve(token_id, account_id, msg)
                }

                #[payable]
                fn nft_revoke(&mut self, token_id: $crate::nft::TokenId, account_id: AccountId) {
                    self.$nft.token.nft_revoke(token_id, account_id)
                }

                #[payable]
                fn nft_revoke_all(&mut self, token_id: $crate::nft::TokenId) {
                    self.$nft.token.nft_revoke_all(token_id)
                }

                fn nft_is_approved(
                    &self,
                    token_id: $crate::nft::TokenId,
                    approved_account_id: AccountId,
                    approval_id: Option<u64>,
                ) -> bool {
                    self.$nft.token.nft_is_approved(token_id, approved_account_id, approval_id)
                }
            }
        };
        (@IMPL_ENUMERATION $contract:ident, $nft:ident) => {
            #[near_bindgen]
            impl $crate::nft::enumeration::NonFungibleTokenEnumeration for $contract {
                fn nft_total_supply(&self) -> U128 {
                    self.$nft.token.nft_total_supply()
                }

                fn nft_tokens(
                    &self,
                    from_index: Option<U128>,
                    limit: Option<u64>,
                ) -> Vec<$crate::nft::Token> {
                    self.$nft.token.nft_tokens(from_index, limit)
                }

                fn nft_supply_for_owner(&self, account_id: AccountId) -> U128 {
                    self.$nft.token.nft_supply_for_owner(account_id)
                }

                fn nft_tokens_for_owner(
                    &self,
                    account_id: AccountId,
                    from_index: Option<U128>,
                    limit: Option<u64>,
                ) -> Vec<$crate::nft::Token> {
                    self.$nft.token.nft_tokens_for_owner(account_id, from_index, limit)
                }
            }
        };
        ($contract:ident, $nft:ident) => {
            #[near_bindgen]
            impl $contract {
                #[payable]
                pub fn nft_mint(
                    &mut self,
                    token_id: $crate::nft::TokenId,
                    receiver_id: AccountId,
                    token_metadata: $crate::nft::TokenMetadata,
                ) -> $crate::nft::Token {
                    self.$nft.token.internal_mint(token_id, receiver_id, Some(token_metadata))
                }
            }
            impl_non_fungible_token_contract!(@IMPL_CORE $contract, $nft);
            impl_non_fungible_token_contract!(@IMPL_APPROVAL $contract, $nft);
            impl_non_fungible_token_contract!(@IMPL_ENUMERATION $contract, $nft);
            #[near_bindgen]
            impl $crate::nft::metadata::NonFungibleTokenMetadataProvider for $contract {
                fn nft_metadata(&self) -> $crate::nft::Metadata {
                    self.$nft.metadata.get().unwrap()
                }
            }
        };
    }
pub use impl_non_fungible_token_contract;

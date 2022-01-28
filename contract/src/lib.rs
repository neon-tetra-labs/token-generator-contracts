use multi_token_standard::{impl_multi_token_core, impl_multi_token_storage, MultiToken};
use near_account::{AccountDeposits, AccountInfoTrait, Accounts, NearAccounts, NewInfo};
use near_internal_balances_plugin::impl_near_balance_plugin;

use near_contract_standards::storage_management::StorageManagement as _StorageManagement;
use near_internal_balances_plugin::token_id::TokenId;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, UnorderedMap};
use near_sdk::json_types::U128;
use near_sdk::{near_bindgen, AccountId, Balance, BorshStorageKey, PanicOnDefault, PromiseOrValue};
use nft_fractionalizer::{NftFractionalizer, NftFractionalizerFns};

pub mod nft_fractionalizer;
pub mod types;
mod utils;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct AccountInfo {
    pub internal_balance: UnorderedMap<TokenId, Balance>,
}

impl NewInfo for AccountInfo {
    fn default_from_account_id(account_id: AccountId) -> Self {
        Self { internal_balance: UnorderedMap::new(format!("{}-bal", account_id).as_bytes()) }
    }
}

#[derive(BorshDeserialize, BorshSerialize, BorshStorageKey)]
enum StorageKey {
    MultiTokenOwner,
    MultiTokenMetadata,
    MultiTokenSupply,
}

impl AccountInfoTrait for AccountInfo {}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault, NearAccounts)]
pub struct Contract {
    pub accounts: Accounts<AccountInfo>,
    pub mt: MultiToken,
    pub owner_id: AccountId,
    pub treasury_id: AccountId,
    pub nft_fractionalizer: NftFractionalizer,
}

// Implement functionality for internal balances and multi tokens
impl_near_balance_plugin!(Contract, accounts, AccountInfo, internal_balance);
impl_multi_token_core!(Contract, mt);
impl_multi_token_storage!(Contract, mt);

#[near_bindgen]
impl Contract {
    /// Initializes the contract with the given total supply owned by the given `owner_id` with
    /// the given fungible token metadata.
    #[init]
    pub fn new(owner_id: AccountId, treasury: AccountId, nft_mint_fee: U128) -> Self {
        Contract {
            accounts: Accounts::new(),
            mt: MultiToken::new(
                StorageKey::MultiTokenOwner,
                owner_id.clone(),
                Some(StorageKey::MultiTokenMetadata),
                StorageKey::MultiTokenSupply,
            ),
            owner_id,
            nft_fractionalizer: NftFractionalizer::new(nft_mint_fee.into()),
            treasury_id: treasury,
        }
    }
}

#[near_bindgen]
impl NftFractionalizerFns for Contract {

    #[payable]
    fn nft_fractionalize(
        &mut self,
        nfts: Vec<TokenId>,
        mt_id: types::MTTokenId,
        amount: U128,
        mt_owner: Option<AccountId>,
        token_metadata: multi_token_standard::metadata::MultiTokenMetadata,
    ) {
        self.nft_fractionalize_internal(nfts, mt_id, amount.into(), mt_owner, token_metadata);
    }

    fn nft_fractionalize_unwrap(&mut self, mt_id: types::MTTokenId, release_to: Option<AccountId>) {
        todo!()
    }

    fn nft_fractionalize_get_underlying(
        &self,
        mt_id: types::MTTokenId,
    ) -> Vec<multi_token_standard::Token> {
        todo!()
    }

    fn nft_fractionalize_update_mint_fee(&mut self, update: U128) {
        todo!()
    }
}

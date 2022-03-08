use multi_token_standard::{
    impl_multi_token_core, impl_multi_token_metadata, impl_multi_token_storage, MultiToken,
};
use near_account::{
    impl_near_accounts_plugin, Account, AccountDeposits, Accounts, NearAccountPlugin,
    NearAccountsPluginNonExternal, NewInfo,
};
use near_internal_balances_plugin::impl_near_balance_plugin;

use near_contract_standards::storage_management::StorageManagement as _StorageManagement;
use near_internal_balances_plugin::token_id::TokenId;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, UnorderedMap};
use near_sdk::json_types::U128;
use near_sdk::{
    env, near_bindgen, AccountId, Balance, BorshStorageKey, PanicOnDefault, PromiseOrValue,
};
use nft_fractionalizer::{NftFractionalizer, NftFractionalizerFns};
use sales::{SaleOptions, SaleOptionsSerial, Sales, SalesFns};

pub mod nft_fractionalizer;
pub mod sales;
pub mod types;
mod utils;

use types::MTTokenId;
pub use utils::FEE_DENOMINATOR;

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

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    pub accounts: Accounts<AccountInfo>,
    pub mt: MultiToken,
    pub owner_id: AccountId,
    pub treasury_id: AccountId,
    pub nft_fractionalizer: NftFractionalizer,
    pub sales: Sales,
}

impl_near_accounts_plugin!(Contract, accounts, AccountInfo);
// Implement functionality for internal balances and multi tokens
impl_near_balance_plugin!(Contract, accounts, AccountInfo, internal_balance);
impl_multi_token_core!(Contract, mt);
impl_multi_token_storage!(Contract, mt);
impl_multi_token_metadata!(Contract, mt);

#[near_bindgen]
impl Contract {
    /// Initializes the contract with the given total supply owned by the given `owner_id` with
    /// the given fungible token metadata.
    #[init]
    pub fn new(
        owner_id: Option<AccountId>,
        treasury: Option<AccountId>,
        nft_mint_fee_numerator: Option<U128>,
        sale_fee_numerator: Option<U128>,
    ) -> Self {
        let owner_id = owner_id.unwrap_or(env::predecessor_account_id());
        let treasury_id = treasury.unwrap_or(env::predecessor_account_id());

        // TODO: register accounts deposit/ init near bal for treasury and owner id
        // See: https://github.com/neon-tetra-labs/token-generator-contracts/issues/3
        let mut this = Contract {
            accounts: Accounts::new(),
            mt: MultiToken::new(
                StorageKey::MultiTokenOwner,
                owner_id.clone(),
                Some(StorageKey::MultiTokenMetadata),
                StorageKey::MultiTokenSupply,
            ),
            sales: Sales::new(sale_fee_numerator.map(|v| v.into()).unwrap_or(0)),
            owner_id: owner_id.clone(),
            nft_fractionalizer: NftFractionalizer::new(
                nft_mint_fee_numerator.map(|v| v.into()).unwrap_or(0),
            ),
            treasury_id: treasury_id.clone(),
        };

        let default_account = Account::default_from_account_id(owner_id.clone());
        this.accounts.accounts.insert(&owner_id, &default_account);

        if owner_id != treasury_id {
            let default_account_treasury = Account::default_from_account_id(treasury_id.clone());
            this.accounts.accounts.insert(&treasury_id, &default_account_treasury);
        }
        this
    }
}

#[near_bindgen]
impl SalesFns for Contract {
    #[payable]
    fn sale_buy(&mut self, mt_id: types::MTTokenId, amount: U128) {
        self.sale_buy_internal(mt_id, amount.into())
    }

    fn sale_info(&self, mt_id: types::MTTokenId) -> SaleOptionsSerial {
        self.sale_info_internal(mt_id)
    }

    fn sale_get_all_sales(&self) -> Vec<(MTTokenId, SaleOptionsSerial)> {
        self.sale_get_all_sales_internal()
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
        sale_amount: Option<U128>,
        sale_price_per_token: Option<U128>,
    ) {
        self.nft_fractionalize_internal(
            nfts,
            mt_id,
            amount.into(),
            mt_owner,
            token_metadata,
            sale_amount.map(|v| v.into()),
            sale_price_per_token.map(|v| v.into()),
        );
    }

    #[payable]
    fn nft_fractionalize_unwrap(&mut self, mt_id: types::MTTokenId, release_to: Option<AccountId>) {
        self.nft_fractionalize_unwrap_internal(mt_id, release_to);
    }

    fn nft_fractionalize_get_mint_fee(&self) -> U128 {
        self.nft_fractionalize_get_mint_fee_internal()
    }

    fn nft_fractionalize_get_underlying(&self, mt_id: types::MTTokenId) -> Vec<TokenId> {
        self.nft_fractionalize_get_underlying_internal(mt_id)
    }

    fn nft_fractionalize_update_mint_fee(&mut self, update: U128) {
        self.nft_fractionalize_update_mint_fee_internal(update);
    }
}

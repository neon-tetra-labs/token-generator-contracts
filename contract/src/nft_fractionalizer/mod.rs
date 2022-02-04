use multi_token_standard::{core::MultiTokenCore, metadata::MultiTokenMetadata, Token};
use near_account::Account;
use near_internal_balances_plugin::{
    InternalBalanceHandlers, SudoInternalBalanceHandlers, TokenId,
};
use near_sdk::{
    assert_one_yocto,
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::UnorderedMap,
    env,
    json_types::U128,
    serde::{self, Deserialize, Serialize},
    AccountId, Balance,
};

use crate::{
    sales::SaleOptions,
    types::{MTTokenId, MTTokenType},
    Contract,
};
#[derive(BorshDeserialize, BorshSerialize)]
pub struct NftInfo {
    nfts: Vec<TokenId>,
    /// Set to true after unwrapping an NFT. This is a permanent action and marks the token as
    /// 'deleted'
    unwrapped: bool,
}
#[derive(BorshDeserialize, BorshSerialize)]
pub struct NftFractionalizer {
    // TODO: add one field for
    mt_to_nfts: UnorderedMap<MTTokenId, NftInfo>,
    mint_fee: u128,
}

pub trait NftFractionalizerFns {
    /// Mints the new token
    /// * `mt_id`: The id of the new token. This id must be new and cannot have existed previously on this contract
    fn nft_fractionalize(
        &mut self,
        nfts: Vec<TokenId>,
        mt_id: MTTokenId,
        amount: U128,
        mt_owner: Option<AccountId>,
        token_metadata: MultiTokenMetadata,
        sale_amount: Option<U128>,
        sale_price_per_token: Option<U128>,
    );

    /// Deletes the mt and releases the nfts.
    fn nft_fractionalize_unwrap(&mut self, mt_id: MTTokenId, release_to: Option<AccountId>);

    fn nft_fractionalize_update_mint_fee(&mut self, update: U128);

    fn nft_fractionalize_get_underlying(&self, mt_id: MTTokenId) -> Vec<TokenId>;

    fn nft_fractionalize_get_mint_fee(&self) -> U128;
}

impl NftFractionalizer {
    pub(crate) fn new(mint_fee: u128) -> Self {
        Self { mt_to_nfts: UnorderedMap::new("nft-f".as_bytes()), mint_fee }
    }
}

impl Contract {
    fn insert_mt(&mut self, mt: &MTTokenId, nfts: Vec<TokenId>) {
        match self.nft_fractionalizer.mt_to_nfts.get(mt) {
            Some(_) => panic!("Should not get here, but only new 'mt's can be added"),
            None => {
                self.nft_fractionalizer.mt_to_nfts.insert(mt, &NftInfo { nfts, unwrapped: false });
            }
        }
    }

    /// Mints the new token
    /// * `mt_id`: The id of the new token. This id must be new and cannot have existed previously on this contract
    pub(crate) fn nft_fractionalize_internal(
        &mut self,
        nfts: Vec<TokenId>,
        mt_id: MTTokenId,
        amount: u128,
        mt_owner: Option<AccountId>,
        token_metadata: MultiTokenMetadata,
        sale_amount: Option<Balance>,
        sale_price_per_token: Option<Balance>,
    ) {
        let minter = env::predecessor_account_id();
        let mt_owner = mt_owner.unwrap_or(minter.clone());
        let initial_storage_usage = env::storage_usage();

        // Subtract from the user's balances
        for token in &nfts {
            Self::assert_nft_type(token);
            self.internal_balance_subtract(&minter, &token, 1);
        }

        // create the mt
        self.mint_mt(
            mt_id.clone(),
            MTTokenType::Ft,
            Some(amount),
            mt_owner.clone(),
            token_metadata,
        );

        // Insert the mt into local data
        self.insert_mt(&mt_id, nfts);

        match (sale_amount, sale_price_per_token) {
            (Some(sale_amount), Some(sale_price_per_token)) => {
                assert!(
                    sale_amount <= amount,
                    "Expected the sale amount to be less than or equal to the total supply"
                );
                // Transfer the sale tokens to the current contract after registering it
                self.mt.internal_register_account(mt_id.clone(), &env::current_account_id());
                self.mt.internal_transfer(
                    &mt_owner,
                    &env::current_account_id(),
                    &mt_id,
                    sale_amount,
                    None,
                );

                self.sales_create(
                    &mt_id,
                    SaleOptions {
                        owner: mt_owner,
                        amount_to_sell: sale_amount,
                        near_price_per_token: sale_price_per_token,
                        sold: 0,
                    },
                );
            }
            _ => {}
        }

        // Return any extra attached deposit not used for storage
        self.check_storage_deposit(
            env::storage_usage() - initial_storage_usage,
            Some(self.nft_fractionalizer.mint_fee),
        );
    }

    /// Deletes the mt and releases the nfts.
    pub(crate) fn nft_fractionalize_unwrap_internal(
        &mut self,
        mt_id: MTTokenId,
        release_to: Option<AccountId>,
    ) {
        assert_one_yocto();
        let caller = env::predecessor_account_id();
        let caller_balance = self.mt.balance_of_batch(caller.clone(), vec![mt_id.clone()])[0].0;
        let total_supply = self.mt.total_supply(mt_id.clone()).0;
        assert_eq!(
            total_supply, caller_balance,
            "Unwrapping can only be done if the unwrapper holds all the tokens"
        );

        // burn the supply of the entire token, but keep around the metadata for future reference
        self.mt.internal_withdraw(&mt_id, &caller, total_supply);

        // redeposit the NFT's into the caller's account
        let nfts = self.nft_fractionalizer.mt_to_nfts.get(&mt_id).unwrap();
        let release_to = release_to.as_ref().unwrap_or(&caller);
        for nft in nfts.nfts {
            self.internal_balance_increase(&release_to, &nft, 1);
        }
    }

    pub(crate) fn nft_fractionalize_get_mint_fee_internal(&self) -> U128 {
        U128::from(self.nft_fractionalizer.mint_fee)
    }

    pub(crate) fn nft_fractionalize_get_underlying_internal(
        &self,
        mt_id: MTTokenId,
    ) -> Vec<TokenId> {
        self.nft_fractionalizer.mt_to_nfts.get(&mt_id).expect("The queried mt does not exist").nfts
    }

    fn assert_nft_type(token: &TokenId) {
        match token {
            TokenId::FT { .. } => panic!("Expected an NFT token type"),
            _ => (),
        };
    }
}

use multi_token_standard::{metadata::MultiTokenMetadata, Token};
use near_account::Account;
use near_internal_balances_plugin::{SudoInternalBalanceHandlers, TokenId};
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::UnorderedMap,
    env,
    json_types::U128,
    AccountId,
};

use crate::{
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
    );

    /// Deletes the mt and releases the nfts.
    fn nft_fractionalize_unwrap(&mut self, mt_id: MTTokenId, release_to: Option<AccountId>);

    fn nft_fractionalize_update_mint_fee(&mut self, update: U128);

    fn nft_fractionalize_get_underlying(&self, mt_id: MTTokenId) -> Vec<Token>;
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
    ) {
        let initial_storage_usage = env::storage_usage();
        let minter = env::predecessor_account_id();

        // Subtract from teh user's balances
        for token in &nfts {
            Self::assert_nft_type(token);
            self.internal_balance_subtract(&minter, &token, 1);
        }

        // create the mt
        self.mint_mt(
            mt_id.clone(),
            MTTokenType::Ft,
            Some(amount),
            mt_owner.unwrap_or(minter),
            token_metadata,
        );

        // Insert the mt into local data
        self.insert_mt(&mt_id, nfts);

        // Return any extra attached deposit not used for storage
        self.check_storage_deposit(
            env::storage_usage() - initial_storage_usage,
            Some(self.nft_fractionalizer.mint_fee),
        );
    }

    /// Deletes the mt and releases the nfts.
    fn nft_fractionalize_unwrap_internal(
        &mut self,
        mt_id: MTTokenId,
        release_to: Option<AccountId>,
    ) {
    }

    fn nft_fractionalize_get_underlying_internal(&self, mt_id: MTTokenId) -> Vec<TokenId> {
        todo!()
    }

    fn assert_nft_type(token: &TokenId) {
        match token {
            TokenId::FT { .. } => panic!("Expected an NFT token type"),
            _ => (),
        };
    }
}

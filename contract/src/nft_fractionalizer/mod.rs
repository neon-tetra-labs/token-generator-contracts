use multi_token_standard::{metadata::MultiTokenMetadata, Token};
use near_account::Account;
use near_internal_balances_plugin::{SudoInternalBalanceHandlers, TokenId};
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::UnorderedMap,
    env, AccountId,
};

use crate::{
    types::{MTTokenId, MTTokenType},
    Contract,
};

#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct NftFractionalizer {
    mt_to_nfts: UnorderedMap<MTTokenId, Vec<TokenId>>,
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

impl Contract {
    /// Mints the new token
    /// * `mt_id`: The id of the new token. This id must be new and cannot have existed previously on this contract
    fn nft_fractionalize_internal(
        &mut self,
        nfts: Vec<TokenId>,
        mt_id: MTTokenId,
        amount: u128,
        mt_owner: Option<AccountId>,
        token_metadata: MultiTokenMetadata,
    ) {
        let minter = env::predecessor_account_id();

        // Subtract from teh user's balances
        for token in nfts {
            Self::assert_nft_type(&token);
            self.internal_balance_subtract(&minter, &token, 1);
        }

        self.mint_mt(
            mt_id,
            MTTokenType::Ft,
            Some(amount),
            env::current_account_id(),
            token_metadata,
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

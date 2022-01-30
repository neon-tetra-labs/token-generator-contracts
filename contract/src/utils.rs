use multi_token_standard::{metadata::MultiTokenMetadata, TokenType};
use near_sdk::{env, require, AccountId, Balance, Promise};
use uint::construct_uint;


use crate::{
    types::{MTTokenId, MTTokenType},
    Contract,
};

construct_uint! {
    /// 256-bit unsigned integer.
    pub struct U256(4);
}

pub(crate) const FEE_DENOMINATOR: u128 = 1_000_000_000u128;

/// Fee/ near transfer handling
impl Contract {
    pub(crate) fn transfer_fee(&mut self, amount: Balance, to: &AccountId) {
        assert!(
            env::attached_deposit() >= amount,
            "Expected the attached deposit to equal the fee of {}",
            amount
        );
        let mut to_account = self
            .accounts
            .get_account(to)
            .unwrap_or_else(|| panic!("Expected {} to be registered", to.as_str()));
        to_account.near_amount += amount;
        self.accounts.insert_account_check_storage(&to, &mut to_account);
    }

    pub(crate) fn calculate_fee(amount: Balance, fee_numerator: u128) -> Balance {
        let ret = U256::from(amount) * U256::from(fee_numerator) / U256::from(FEE_DENOMINATOR);
        ret.as_u128()
    }
}

impl Contract {
    /// Taken from [multi-token-standard-impl/examples/multi-token/mt](https://github.com/shipsgold/multi-token-standard-impl/blob/ec874d2e010908160f6c73555bde119943b96736/examples/multi-token/mt/src/lib.rs#L66)
    /// slightly modified to allow for fee collections
    pub(crate) fn mint_mt(
        &mut self,
        token_id: MTTokenId,
        token_type: MTTokenType,
        amount: Option<u128>,
        token_owner_id: AccountId,
        token_metadata: MultiTokenMetadata,
    ) {
        // Every token must have a token type and every NFT type cannot be re-minted
        match self.mt.token_type_index.get(&token_id) {
            Some(MTTokenType::Ft) => {
                require!(
                    token_type == MTTokenType::Ft,
                    "Type must be of FT time tokenId already exists"
                )
            }
            Some(MTTokenType::Nft) => {
                env::panic_str("Attempting to mint already minted NFT");
            }
            None => {
                self.mt.token_type_index.insert(&token_id, &token_type);
            }
        }

        let owner_id: AccountId = token_owner_id;
        // Core behavior: every token must have an owner
        match token_type {
            TokenType::Ft => {
                if amount.is_none() {
                    env::panic_str("Amount must be specified for Ft type tokens");
                }
                // advance the prefix index before insertion
                let amt = u128::from(amount.unwrap());
                //create LookupMap for balances
                match self.mt.ft_owners_by_id.get(&token_id) {
                    Some(mut balances) => {
                        let current_bal = balances.get(&owner_id).unwrap_or(0);
                        // TODO not quite safe
                        if amt == 0 {
                            env::panic_str("error: amount should be greater than 0")
                        }
                        balances.insert(&owner_id, &(current_bal + amt));
                        let supply = self.mt.ft_token_supply_by_id.get(&token_id).unwrap();
                        self.mt.ft_token_supply_by_id.insert(&token_id, &(supply + amt));
                    }
                    None => {
                        let mut balances = self.mt.internal_new_ft_balances();
                        // insert amount into balances
                        balances.insert(&owner_id, &amt);
                        self.mt.ft_owners_by_id.insert(&token_id, &balances);
                        self.mt.ft_token_supply_by_id.insert(&token_id, &amt);
                    }
                }
            }
            TokenType::Nft => {
                self.mt.nft_owner_by_id.insert(&token_id, &owner_id);
            }
        }
        // Metadata extension: Save metadata, keep variable around to return later.
        // Note that check above already panicked if metadata extension in use but no metadata
        // provided to call.
        self.mt
            .token_metadata_by_id
            .as_mut()
            .and_then(|by_id| by_id.insert(&token_id, &token_metadata));
    }

    /// Taken from [multi-token-standard-impl/examples/multi-token/mt](https://github.com/shipsgold/multi-token-standard-impl/blob/ec874d2e010908160f6c73555bde119943b96736/examples/multi-token/mt/src/lib.rs#L51)
    /// with modifications to allow for a "keep" amount
    pub(crate) fn check_storage_deposit(&self, storage_used: u64, fee_amount: Option<Balance>) {
        let required_cost =
            env::storage_byte_cost() * Balance::from(storage_used) + fee_amount.unwrap_or(0);

        let attached_deposit = env::attached_deposit();
        assert!(
            required_cost <= attached_deposit,
            "Must attach {} yoctoNEAR to cover storage",
            required_cost,
        );
        let refund = attached_deposit - required_cost;
        if refund > 1 {
            Promise::new(env::predecessor_account_id()).transfer(refund);
        }
    }
}

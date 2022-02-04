use multi_token_standard::{core::MultiTokenCore, metadata::MultiTokenMetadata, Token};
use near_account::Account;
use near_sdk::{
    assert_one_yocto,
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::UnorderedMap,
    env,
    json_types::U128,
    serde::{self, Deserialize, Serialize},
    AccountId, Balance,
};

use crate::{types::MTTokenId, utils::FEE_DENOMINATOR, Contract};

#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize, PartialEq, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct SaleOptions {
    pub amount_to_sell: Balance,
    pub near_price_per_token: Balance,
    pub sold: Balance,
    pub owner: AccountId,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Sales {
    sales: UnorderedMap<MTTokenId, SaleOptions>,
    /// Platform fee connoted by the numerator of FEE_DENOMINATOR in utils.rs
    platform_fee_numerator: u128,
}

// TODO: fee
pub trait SalesFns {
    fn sale_buy(&mut self, mt_id: MTTokenId, amount: U128);
    fn sale_info(&self, mt_id: MTTokenId) -> SaleOptions;
}

impl Sales {
    pub fn new(platform_fee: u128) -> Self {
        Self { sales: UnorderedMap::new("sxy".as_bytes()), platform_fee_numerator: platform_fee }
    }
}

impl SaleOptions {
    fn get_near_cost(&self, amount: Balance) -> Balance {
        self.near_price_per_token * amount
    }
}

impl Contract {
    pub(crate) fn sale_info_internal(&self, mt_id: MTTokenId) -> SaleOptions {
        self.sales.sales.get(&mt_id).expect("Cannot find the sale with the given token id")
    }

    pub(crate) fn sale_buy_internal(&mut self, mt_id: MTTokenId, amount: Balance) {
        let caller = env::predecessor_account_id();
        // ensure the caller is registered
        let caller_registered = self
            .mt
            .ft_owners_by_id
            .get(&mt_id)
            .expect(&format!("Cannot find token {}", mt_id))
            .get(&caller)
            .is_some();
        assert!(caller_registered == true, "Expected the caller to be registered");

        let mut sale =
            self.sales.sales.get(&mt_id).expect(&format!("Cannot find sale for {}", mt_id));
        let cost = sale.get_near_cost(amount);

        // Make sure that the proper amount is attached and transfer accordingly
        assert_eq!(env::attached_deposit(), cost, "Expected {} attached to pay for the sale", cost);

        // Transfer the fees/ cost
        let amount_to_treasury = Self::calculate_fee(cost, self.sales.platform_fee_numerator);
        let amount_to_owner = cost - amount_to_treasury;
        let treasury = &self.treasury_id.clone();
        self.transfer_fee(amount_to_treasury, treasury);
        self.transfer_fee(amount_to_owner, &sale.owner);

        // Transfer the token's to the buyer's account
        self.mt.internal_transfer(&env::current_account_id(), &caller, &mt_id, amount, None);

        sale.sold += amount;
        self.sales.sales.insert(&mt_id, &sale);
    }
}

impl Contract {
    pub(crate) fn sales_create(&mut self, mt_id: &MTTokenId, sale: SaleOptions) {
        if self.sales.sales.get(mt_id).is_some() {
            panic!("Expected to not find an existing sale for {}", &mt_id);
        }
        self.sales.sales.insert(mt_id, &sale);
    }
}

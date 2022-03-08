use std::convert::TryFrom;

use contract::sales::{SaleOptions, SaleOptionsSerial};
use contract::types::MTTokenId;
use contract::FEE_DENOMINATOR;
use near_contract_standards::storage_management::{StorageBalance, StorageBalanceBounds};
use near_sdk::env;
use near_sdk::json_types::U128;
use near_sdk::serde::{self, Deserialize, Serialize};
use near_sdk::serde_json::json;
use near_sdk_sim::{call, to_yocto, transaction::ExecutionStatus, view, DEFAULT_GAS};

use near_internal_balances_plugin::TokenId;

use crate::testing::utils::{init_with_macros as init, register_user};
use crate::testing::InitRet;

use super::{get_default_metadata, INIT_USER_BAL_NEAR};

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct StorageBalanceTmp {
    pub total: U128,
    pub available: U128,
}

pub const NFT_MINT_FEE: u128 = 1_000_000;
pub const SALE_FEE_NUMERATOR: u128 = 100_000_000u128;
const SUPPLY: u128 = 1_000_000_000_000_000;

fn init_with_fractionalize_nfts(
    sale_amount_whole: Option<U128>,
    sale_price_per_whole: Option<U128>,
    amount_frac_attach: Option<u128>,
) -> (InitRet, Vec<TokenId>, MTTokenId) {
    let nfts = vec!["nft_1".to_string(), "nft_2".to_string()];
    let InitRet { alice, root, nft, contract } =
        init(nfts.clone(), NFT_MINT_FEE, SALE_FEE_NUMERATOR);
    let mt_id = "MyNFTFRACED".to_string();

    // deposit the NFTs
    for nft_id in &nfts {
        call!(
            root,
            nft.nft_transfer_call(
                contract.account_id(),
                nft_id.clone(),
                None,
                None,
                "".to_string()
            ),
            deposit = 1
        )
        .assert_success();
    }

    let nfts_tok_ids: Vec<TokenId> = nfts
        .iter()
        .map(|nft_id| TokenId::NFT { contract_id: nft.account_id(), token_id: nft_id.clone() })
        .collect();

    for nft_tok in &nfts_tok_ids {
        let bal: U128 =
            view!(contract.internal_balance_get_balance(root.account_id(), nft_tok.clone()))
                .unwrap_json();
        assert_eq!(bal.0, 1);
    }

    // Fractionalize them
    call!(
        root,
        contract.nft_fractionalize(
            nfts_tok_ids.clone(),
            mt_id.clone(),
            U128::from(SUPPLY),
            None,
            get_default_metadata(),
            sale_amount_whole,
            sale_price_per_whole
        ),
        deposit =
            amount_frac_attach.unwrap_or(NFT_MINT_FEE + near_sdk::env::storage_byte_cost() * 1_000)
    )
    .assert_success();
    let bal_post_frac: U128 =
        view!(contract.balance_of(root.account_id(), mt_id.clone())).unwrap_json();
    assert_eq!(bal_post_frac.0, SUPPLY - sale_amount_whole.map(|a| a.0).unwrap_or(0));

    for nft_tok in &nfts_tok_ids {
        let bal: U128 =
            view!(contract.internal_balance_get_balance(root.account_id(), nft_tok.clone()))
                .unwrap_json();
        assert_eq!(bal.0, 0);
    }
    (InitRet { alice, root, nft, contract }, nfts_tok_ids, mt_id)
}

#[test]
fn simulate_simple_fractionalization() {
    let (InitRet { alice, root, nft, contract }, nfts_tok_ids, mt_id) =
        init_with_fractionalize_nfts(None, None, None);
    call!(root, contract.nft_fractionalize_unwrap(mt_id.clone(), None), deposit = 1)
        .assert_success();
    for nft_tok in &nfts_tok_ids {
        let bal: U128 =
            view!(contract.internal_balance_get_balance(root.account_id(), nft_tok.clone()))
                .unwrap_json();
        assert_eq!(bal.0, 1);
    }
    let bal_post_unwrap: U128 =
        view!(contract.balance_of(root.account_id(), mt_id.clone())).unwrap_json();
    assert_eq!(bal_post_unwrap.0, 0);
}

#[test]
fn simulate_nft_frac_sale() {
    let sale_amount_whole = 100;
    let whole_to_buy = 10;
    let sale_price_whole = 100;
    let (InitRet { alice, root, nft, contract }, nfts_tok_ids, mt_id) =
        init_with_fractionalize_nfts(
            Some(sale_amount_whole.into()),
            Some(sale_price_whole.into()),
            None,
        );
    let treasury = alice.account_id();

    let bal_init_sale: U128 =
        view!(contract.balance_of(root.account_id(), mt_id.clone())).unwrap_json();
    assert_eq!(bal_init_sale.0, SUPPLY - sale_amount_whole);

    // Check that the initial balances are correct
    let bal_init_contract: U128 =
        view!(contract.balance_of(contract.account_id(), mt_id.clone())).unwrap_json();
    assert_eq!(bal_init_sale.0, SUPPLY - sale_amount_whole);
    assert_eq!(bal_init_contract.0, sale_amount_whole);

    // Check the initial sale
    let mut desired_opts = SaleOptions {
        amount_to_sell: sale_amount_whole,
        near_price_per_token: sale_price_whole,
        sold: 0,
        owner: root.account_id(),
    };
    let sale_info_serial: SaleOptionsSerial =
        view!(contract.sale_info(mt_id.clone())).unwrap_json();
    let sale_info = SaleOptions {
        amount_to_sell: sale_info_serial.amount_to_sell.0,
        near_price_per_token: sale_info_serial.near_price_per_token.0,
        sold: sale_info_serial.sold.0,
        owner: sale_info_serial.owner.clone(),
    };

    assert_eq!(&desired_opts, &sale_info);

    let seller_bal_pre_sale: StorageBalance =
        view!(contract.accounts_near_balance_of(root.account_id())).unwrap_json();

    // Make a sale
    call!(
        alice,
        contract.storage_deposit(vec![mt_id.clone()], None, None),
        deposit = near_sdk::env::storage_byte_cost() * 2_000
    )
    .assert_success();

    let treasury_bal_pre_sale: StorageBalance =
        view!(contract.accounts_near_balance_of(treasury.clone())).unwrap_json();

    call!(
        alice,
        contract.sale_buy(mt_id.clone(), U128::from(whole_to_buy)),
        deposit = sale_price_whole * whole_to_buy
    )
    .assert_success();

    let sale_info_serial: SaleOptionsSerial =
        view!(contract.sale_info(mt_id.clone())).unwrap_json();

    let sale_info = SaleOptions {
        amount_to_sell: sale_info_serial.amount_to_sell.0,
        near_price_per_token: sale_info_serial.near_price_per_token.0,
        sold: sale_info_serial.sold.0,
        owner: sale_info_serial.owner.clone(),
    };

    desired_opts.sold += whole_to_buy;
    assert_eq!(desired_opts, sale_info);

    let bal_post_sale_contract: U128 =
        view!(contract.balance_of(contract.account_id(), mt_id.clone())).unwrap_json();
    assert_eq!(bal_post_sale_contract.0, (sale_amount_whole - whole_to_buy));

    let bal_post_sale_alice: U128 =
        view!(contract.balance_of(alice.account_id(), mt_id.clone())).unwrap_json();
    assert_eq!(bal_post_sale_alice.0, whole_to_buy);

    // Check the near account balances
    // contract.

    let expected_treasury_increase =
        sale_price_whole * whole_to_buy * SALE_FEE_NUMERATOR / FEE_DENOMINATOR;
    let expected_seller_increase = sale_price_whole * whole_to_buy - expected_treasury_increase;

    let treasury_bal: StorageBalance =
        view!(contract.accounts_near_balance_of(treasury)).unwrap_json();

    let seller_bal: StorageBalance =
        view!(contract.accounts_near_balance_of(root.account_id())).unwrap_json();

    assert_eq!(
        treasury_bal.available.0 - treasury_bal_pre_sale.available.0,
        expected_treasury_increase
    );
    assert_eq!(seller_bal.available.0 - seller_bal_pre_sale.available.0, expected_seller_increase);
}

#[test]
#[should_panic(
    expected = "Must attach 9090000000000001000000 yoctoNEAR to cover storage and/or fees"
)]
fn simulate_fractionalize_not_enough_attached() {
    let sale_amount_whole = 100;
    let whole_to_buy = 10;
    let sale_price_whole = 100;
    let (InitRet { alice, root, nft, contract }, nfts_tok_ids, mt_id) =
        init_with_fractionalize_nfts(
            Some(sale_amount_whole.into()),
            Some(sale_price_whole.into()),
            Some(0),
        );
    let treasury = alice.account_id();
}

#[test]
fn simulate_fractionalize_too_much_attached_and_returns() {
    let sale_amount_whole = 100;
    let whole_to_buy = 10;
    let sale_price_whole = 100;
    let amount_attached = to_yocto("2");
    let (InitRet { alice, root, nft, contract }, nfts_tok_ids, mt_id) =
        init_with_fractionalize_nfts(
            Some(sale_amount_whole.into()),
            Some(sale_price_whole.into()),
            Some(amount_attached),
        );
    let alice_init_bal = to_yocto(INIT_USER_BAL_NEAR);
    let near_bal = alice.account().unwrap().amount;
    assert!(alice_init_bal - near_bal < to_yocto("2"))
}

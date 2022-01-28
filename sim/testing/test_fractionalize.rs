use std::convert::TryFrom;

use near_contract_standards::storage_management::{StorageBalance, StorageBalanceBounds};
use near_sdk::json_types::U128;
use near_sdk::serde::{self, Deserialize, Serialize};
use near_sdk::serde_json::json;
use near_sdk_sim::{call, to_yocto, transaction::ExecutionStatus, view, DEFAULT_GAS};

use near_internal_balances_plugin::TokenId;

use crate::testing::utils::{init_with_macros as init, register_user};

use super::get_default_metadata;

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct StorageBalanceTmp {
    pub total: U128,
    pub available: U128,
}

pub const NFT_MINT_FEE: u128 = 1_000_000;

#[test]
fn simulate_simple_fractionalization() {
    let nfts = vec!["nft_1".to_string(), "nft_2".to_string()];
    let (root, dummy, nft, alice) = init(nfts.clone(), NFT_MINT_FEE);
    let supply = 1_000;
    let mt_id = "MyNFTFRACED".to_string();

    // deposit the NFTs
    for nft_id in &nfts {
        call!(
            root,
            nft.nft_transfer_call(dummy.account_id(), nft_id.clone(), None, None, "".to_string()),
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
            view!(dummy.internal_balance_get_balance(root.account_id(), nft_tok.clone()))
                .unwrap_json();
        assert_eq!(bal.0, 1);
    }

    // Fractionalize them
    call!(
        root,
        dummy.nft_fractionalize(
            nfts_tok_ids.clone(),
            mt_id.clone(),
            U128::from(supply),
            None,
            get_default_metadata()
        ),
        deposit = NFT_MINT_FEE + near_sdk::env::storage_byte_cost() * 1_000
    )
    .assert_success();
    let bal_post_frac: U128 =
        view!(dummy.balance_of(root.account_id(), mt_id.clone())).unwrap_json();
    assert_eq!(bal_post_frac.0, supply);

    for nft_tok in &nfts_tok_ids {
        let bal: U128 =
            view!(dummy.internal_balance_get_balance(root.account_id(), nft_tok.clone()))
                .unwrap_json();
        assert_eq!(bal.0, 0);
    }

    // TODO: check bals after unwrapping
}

#[test]
#[should_panic]
fn simulate_fractionalize_not_enough_attached() {
    todo!()
}

#[test]
fn simulate_fractionalize_too_much_attached_and_returns() {
    todo!()
}

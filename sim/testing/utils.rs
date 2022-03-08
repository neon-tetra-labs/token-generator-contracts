use std::str::FromStr;

use contract::ContractContract;
use multi_token_standard::metadata::MultiTokenMetadata;
use near_sdk::serde_json::json;
use near_sdk::{json_types::U128, AccountId};
use near_sdk_sim::{
    call, deploy, init_simulator, to_yocto, ContractAccount, UserAccount, DEFAULT_GAS,
    STORAGE_AMOUNT,
};
use nft::{ContractContract as NFTContract, DEFAULT_META};

// Load in contract bytes at runtime
near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    CONTRACT_BYTES => "res/contract.wasm",
    NFT_BYTES => "res/nft.wasm",
}

pub struct InitRet {
    pub alice: UserAccount,
    pub root: UserAccount,
    pub contract: ContractAccount<ContractContract>,
    pub nft: ContractAccount<NFTContract>,
}

pub const CONTRACT_ID: &str = "dummy";
pub const NFT_ID: &str = "nft";

pub const INIT_USER_BAL_NEAR: &str = "100";

pub fn get_default_metadata() -> MultiTokenMetadata {
    multi_token_standard::metadata::MultiTokenMetadata {
        spec: "aa".to_string(),   // required, essentially a version like "mt-1.0.0"
        name: "aa".to_string(),   // required, ex. "Mosaics"
        symbol: "aa".to_string(), // required, ex. "MOSIAC"
        icon: None,               // Data URL
        base_uri: None, // Centralized gateway known to have reliable access to decentralized storage assets referenced by `reference` or `media` URLs
        decimals: Some(12), // precision decimals for tokens that need this information
        reference: None, // URL to a JSON file with more info
        reference_hash: None, // Base64-encoded sha256 hash of JSON from reference field. Required if `reference` is included.
        title: None,          // ex. "Arch Nemesis: Mail Carrier" or "Parcel #5055"
        description: None,    // free-form description
        media: None, // URL to associated media, preferably to decentralized, content-addressed storage
        media_hash: None, // Base64-encoded sha256 hash of content referenced by the `media` field. Required if `media` is included.
        copies: None, // number of copies of this set of metadata in existence when token was minted.
        issued_at: None, // ISO 8601 datetime when token was issued or minted
        expires_at: None, // ISO 8601 datetime when token expires
        starts_at: None, // ISO 8601 datetime when token starts being valid
        updated_at: None, // ISO 8601 datetime when token was last updated
        extra: None,  // anything extra the NFT wants to store on-chain. Can be stringified JSON.
    }
}


// Register the given `user` with NFT contract and the contract
pub fn register_user(user: &near_sdk_sim::UserAccount) {
    user.call(
        AccountId::from_str(CONTRACT_ID).unwrap(),
        "accounts_storage_deposit",
        &json!({
            "registration_only": false,
        })
        .to_string()
        .into_bytes(),
        near_sdk_sim::DEFAULT_GAS / 2,
        near_sdk::env::storage_byte_cost() * 2_000,
    );
}

pub fn init_with_macros(nfts_to_mint: Vec<String>, nft_mint_fee: u128, sale_fee: u128) -> InitRet {
    let root = init_simulator(None);
    // uses default values for deposit and gas
    let contract = deploy!(
        // Contract Proxy
        contract: ContractContract,
        // Contract account id
        contract_id: CONTRACT_ID,
        // Bytes of contract
        bytes: &CONTRACT_BYTES,
        // User deploying the contract,
        signer_account: root,
        // init method
        init_method: new(Some(root.account_id()), Some(AccountId::from_str("alice").unwrap()), Some(U128::from(nft_mint_fee)), Some(U128::from(sale_fee)))
    );

    let nft = deploy!(
        // Contract Proxy
        contract: NFTContract,
        // Contract account id
        contract_id: NFT_ID,
        // Bytes of contract
        bytes: &NFT_BYTES,
        // User deploying the contract,
        signer_account: root,
        // init method
        init_method: new_default_meta(root.account_id())
    );

    let alice = root.create_user(AccountId::from_str("alice").unwrap(), to_yocto("100"));

    for nft_id in nfts_to_mint {
        call!(
            root,
            nft.nft_mint(nft_id, root.account_id(), Some(DEFAULT_META)),
            deposit = near_sdk::env::storage_byte_cost() * 1_000
        )
        .assert_success();
    }

    register_user(&root);
    register_user(&alice);

    InitRet { root, alice, contract, nft }
}

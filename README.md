# Token Fractionalizing and Subsequent Sales

The following smart contract found in the `contract` directory. The main
purpose of the smart contract is to allow users to fractionalize a set of NFTs
and subsequently sell them with a simple sales model.

The fractionalize component basically means that a user can deposit some number of NFTs into a contract, the contract then locks up those NFTs and mints some number of Multi-Tokens (see the new [Multi Token proposal](https://github.com/near/NEPs/issues/246)). In order to redeem the NFTs, a caller has to burn all
the supply of a fraction.

These smart contracts make extensive use of the [Near internal balances plugin](https://docs.rs/near-internal-balances-plugin/latest/near_internal_balances_plugin/)
alongside the [Near Accounts library](https://docs.rs/near-account/latest/near_account/).

This contract also makes use of the [Multi Token Standard implementation](https://github.com/shipsgold/multi-token-standard-impl/tree/feat/initial-token).

The public methods additionally exposed are defined by two traits, `NFTFractionalizeFns` and `SalesFns` are
```rust
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

pub trait SalesFns {
    fn sale_buy(&mut self, mt_id: MTTokenId, amount: U128);
    fn sale_info(&self, mt_id: MTTokenId) -> SaleOptionsSerial;
    fn sale_get_all_sales(&self) -> Vec<(MTTokenId, SaleOptionsSerial)>;
}

```
as well as a `new` function
```rust
    /// Initializes the contract with the given total supply owned by the given `owner_id` with
    /// the given fungible token metadata.
    #[init]
    pub fn new(
        owner_id: Option<AccountId>,
        treasury: Option<AccountId>,
        nft_mint_fee_numerator: Option<U128>,
        sale_fee_numerator: Option<U128>,
    );
```


Sample usage
=============
For sample usage, please check out `sim/testing/utils.rs` and `sim/testing/test_fractionalize.rs`.

Prerequisites
=============


1. Make sure Rust is installed per the prerequisites in [`near-sdk-rs`](https://github.com/near/near-sdk-rs#pre-requisites)
2. Ensure `near-cli` is installed by running `near --version`. If not installed, install with: `npm install -g near-cli`

## Building

To build run:
```bash
./build.sh
```

Using this contract
===================

### Quickest deploy

You can build and deploy this smart contract to a development account. [Dev Accounts](https://docs.near.org/docs/concepts/account#dev-accounts) are auto-generated accounts to assist in developing and testing smart contracts. Please see the [Standard deploy](#standard-deploy) section for creating a more personalized account to deploy to.

```bash
near dev-deploy --wasmFile res/contract.wasm --helperUrl https://near-contract-helper.onrender.com
```
Behind the scenes, this is creating an account and deploying a contract to it. On the console, notice a message like:

>Done deploying to dev-1234567890123

In this instance, the account is `dev-1234567890123`. A file has been created containing a key pair to
the account, located at `neardev/dev-account`. To make the next few steps easier, we're going to set an
environment variable containing this development account id and use that when copy/pasting commands.
Run this command to the environment variable:

```bash
source neardev/dev-account.env
```

You can tell if the environment variable is set correctly if your command line prints the account name after this command:
```bash
echo $CONTRACT_NAME
```

The next command will initialize the contract using the `new` method:

```bash
near call $CONTRACT_NAME new '{<ARGS>}' --accountId $CONTRACT_NAME
```

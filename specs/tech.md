<!-- TODO: move this to just be part of the rust code w/ doc generation -->

## A quick tech spec

### The main data to store and contract

```rust
type MtTokenId = String;

struct AccountInfo {
	internal_balances: Map<TokenId, Balance>
}

struct NftFractionalizer {
	// Expects token id to be an NFT of type NFT or MT
	mt_token: Map<TokenId, MtTokenId>
	// hmmmmm.... we have to think about this/ explore the current offerings...
	fee: u64
}

struct Contract {
	accounts: NearAccounts<AccountInfo>,
	mt: MultiToken,
	owner_id: AccountId,
	nft_fractionalizer: NftFractionalizer
}
```

### Public Write Functions

#### NFT fractionalization

```rust
// Mints the new token
// Make sure to check the mt does not already exist (also, should we ever allow duplicate names? (like if 1 is deleted later?? Me thinks not)
fn nft_fractionalize(nfts: Vec<TokenId>, mt_id: TokenId, amount: U128, mt_owner: Option<AccountId>, token_metadata: MultiTokenMetadata);

// Deletes the mt and releases the nfts. 
fn nft_defractionalize(mt_id: TokenId, release_nft_to: Option<AccountId>);
```

<!-- ALLOW FEE TO BE UPDATED -->

### Public View Functions

### File Structure

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
}
```

### Public Write Functions

#### NFT fractionalization

```rust
fn nft_fractionalize(nfts: Vec<TokenId>, mt_id: TokenId, amount: U128, mt_owner: Option<AccountId>, token_metadata: MultiTokenMetadata);

fn nft_defractionalize(mt_id: TokenId, release_nft_to: Option<AccountId>);
```

<!-- ALLOW FEE TO BE UPDATED -->

### Public View Functions

### File Structure

#!/bin/sh
# Meant to be called from the root of the repository

near dev-deploy res/nft.wasm --contractName --initArgs "{\"owner_id\": \"atilla-test.testnet\"}" --contractName dev-1644267709016-78788937283506
 && mv neardev/dev-account neardev/NFTContract
near call $(cat neardev/NFTContract) new_default_meta "{\"owner_id\": \"atilla-test.testnet\"}" --accountId atilla-test.testnet || true
#!/bin/sh
# Meant to be called from the root of the repository

# TODO: add options for ownerId, treasury etc.
near dev-deploy res/contract.wasm --contractName dev-1644267069866-50957569549390 && mv neardev/dev-account neardev/CoreContract
near call $(cat neardev/CoreContract) new "{}" --accountId atilla-test.testnet || echo 'Already initialized'
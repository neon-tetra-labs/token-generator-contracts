#!/bin/sh
# Meant to be called from the root of the repository

# TODO: add options for ownerId, treasury etc.
near call $(cat neardev/dev-account) new "{}" --accountId atilla-test.testnet
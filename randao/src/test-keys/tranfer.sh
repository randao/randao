#!/bin/bash

# Read the test-key.json file
keys=$(cat test-key.json)

# Extract the accounts from the JSON object
accounts=$(echo $keys | jq -r '.[] | .account')

# Iterate over the accounts and execute the contract-deposit command
for account in $accounts; do
  echo $account
  fn contract-deposit -a $account -n 20000000000000
  sleep 2
done

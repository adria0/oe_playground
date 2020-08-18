#!/bin/bash
# checks if Open Ethereum has a fully synced blockchain

ETH_SYNCING=$(curl -X POST --data '{"jsonrpc":"2.0","method":"eth_syncing","params":[],"id":1}' http://localhost:8545 -H 'Content-Type: application/json')
RESULT=$(echo "$ETH_SYNCING" | jq -r .result)

if [ "$RESULT" == "false" ]; then
  echo "Open Ethereum is ready to start accepting traffic"
  exit 0
else
  echo "Open Ethereum is still syncing the blockchain"
  exit 1
fi

#!/bin/bash
set -e

MASTER_ACCOUNT="migros.testnet"
TIME=$(date +%s)

cd "$(dirname $0)/.."

export NEAR_ENV=testnet
LG='\033[1;30m' # Arrows color (Dark gray)
TC='\033[0;33m' # Text color (Orange)
NC='\033[0m' # No Color

CONTRACT_ID="${TIME}.${MASTER_ACCOUNT}"

OWNER_ID="migros.testnet"


echo -e "$LG>>>>>>>>>>>>>>$TC Creating and deploying contract account: $CONTRACT_ID $LG<<<<<<<<<<<<<<$NC"
near create-account $CONTRACT_ID --masterAccount=$MASTER_ACCOUNT --initialBalance=2

near deploy $CONTRACT_ID target/wasm32-unknown-unknown/release/dangel_token.wasm new_dangel_meta '{"owner_id": "'$OWNER_ID'"}'


echo -e "$LG>>>>>>>>>>>>>>$TC Dropping info to continue working from NEAR CLI: $LG<<<<<<<<<<<<<<$NC"
echo -e "export NEAR_ENV=testnet"
echo -e "export OWNER_ID=$OWNER_ID"
echo -e "export CONTRACT_ID=$CONTRACT_ID"

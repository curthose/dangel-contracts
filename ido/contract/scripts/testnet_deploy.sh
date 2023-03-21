#!/bin/bash
set -e

MASTER_ACCOUNT="baraka.testnet"
TIME=$(date +%s)

cd "$(dirname $0)/.."

export NEAR_ENV=testnet
LG='\033[1;30m' # Arrows color (Dark gray)
TC='\033[0;33m' # Text color (Orange)
NC='\033[0m' # No Color

echo -e "$LG>>>>>>>>>>>>>>$TC Deploy an empty contract to fund main account $LG<<<<<<<<<<<<<<$NC"
echo -n "" > /tmp/empty
near dev-deploy -f /tmp/empty
TMP_ACCOUNT="$(cat neardev/dev-account)"

MAIN="${TIME}.${MASTER_ACCOUNT}"

echo -e "$LG>>>>>>>>>>>>>>$TC Creating main account: $MAIN $LG<<<<<<<<<<<<<<$NC"
near create-account $MAIN --masterAccount=$MASTER_ACCOUNT --initialBalance=0.01

echo -e "$LG>>>>>>>>>>>>>>$TC Funding main account: $MAIN $LG<<<<<<<<<<<<<<$NC"
near delete $TMP_ACCOUNT $MAIN

OWNER_ID="baraka.testnet"
USER_ID="owner.1678952026.baraka.testnet"
 
CONTRACT_ID="contract.$MAIN"
TOKEN_ID="token.baraka.testnet"

echo -e "$LG>>>>>>>>>>>>>>$TC Creating and deploying contract account: $CONTRACT_ID $LG<<<<<<<<<<<<<<$NC"
near create-account $CONTRACT_ID --masterAccount=$MAIN --initialBalance=25
near deploy $CONTRACT_ID target/wasm32-unknown-unknown/release/dangel_ido.wasm new '{
  "owner_id": "'$OWNER_ID'",
  "total_ido": "400000000000000000000000",
  "staking_contract_id": "contract.1678952026.baraka.testnet",
  "payment_token_id": "'$TOKEN_ID'",
  "dangel_token_id": "token.1678952026.baraka.testnet",
  "ido_price": "2000000000000000000",
  "register_start_timestamp": 1679311892,
  "register_end_timestamp": 1679571089,
  "sale_start_timestamp": 1679311892,
  "sale_end_timestamp": 1679571089,
  "min_tier_cap_rate": 500
  }'


echo -e "$LG>>>>>>>>>>>>>>$TC Initializing assets $LG<<<<<<<<<<<<<<$NC"
ONE_YOCTO="0.000000000000000000000001"
STORAGE="0.001"
GAS="200000000000000"

# Booster APR is 30%, to verify run ./scripts/apr_to_rate.py 30
# Max APR for all assets is 250%
# Booster can't be used as a collateral or borrowed (for now), so APR doesn't matter.
near call $CONTRACT_ID --accountId=$OWNER_ID storage_deposit '{"account_id":"'$USER_ID'"}' --amount=$STORAGE
near call $CONTRACT_ID --accountId=$USER_ID register '' --amount=$ONE_YOCTO --gas=$GAS


echo -e "$LG>>>>>>>>>>>>>>$TC token storage for contract $LG<<<<<<<<<<<<<<$NC"
near call $TOKEN_ID --accountId=$CONTRACT_ID storage_deposit '' --amount=0.00125

near call $TOKEN_ID --accountId=$CONTRACT_ID storage_deposit '{"account_id":"'$USER_ID'"}' --amount=0.00125

near call $TOKEN_ID --accountId=$OWNER_ID --gas=$GAS --amount=$ONE_YOCTO ft_transfer '{
  "receiver_id": "'$USER_ID'",
  "amount": "9333000000000000000000"
}'


near call $TOKEN_ID --accountId=$USER_ID --gas=$GAS --amount=$ONE_YOCTO ft_transfer_call '{
  "receiver_id": "'$CONTRACT_ID'",
  "amount": "33000000000000000000",
  "msg": ""
}'

near view $CONTRACT_ID get_tier_allocation '{"tier":"Tier3"}'


echo -e "$LG>>>>>>>>>>>>>>$TC Dropping info to continue working from NEAR CLI: $LG<<<<<<<<<<<<<<$NC"
echo -e "export NEAR_ENV=testnet"
echo -e "export OWNER_ID=$OWNER_ID"
echo -e "export ORACLE_ID=$ORACLE_ID"
echo -e "export CONTRACT_ID=$CONTRACT_ID"
echo -e "export TOKEN_ID=$TOKEN_ID"
echo -e "export ONE_YOCTO=$ONE_YOCTO"
echo -e "export GAS=$GAS"
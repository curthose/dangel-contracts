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

OWNER_ID="owner.$MAIN"
echo -e "$LG>>>>>>>>>>>>>>$TC Creating owner account: $OWNER_ID $LG<<<<<<<<<<<<<<$NC"
near create-account $OWNER_ID --masterAccount=$MAIN --initialBalance=130

BOOSTER_TOKEN_ID="token.$MAIN"
echo -e "$LG>>>>>>>>>>>>>>$TC Creating and deploying booster token: $BOOSTER_TOKEN_ID $LG<<<<<<<<<<<<<<$NC"
near create-account $BOOSTER_TOKEN_ID --masterAccount=$MAIN --initialBalance=3
near deploy $BOOSTER_TOKEN_ID res/fungible_token.wasm new '{
   "owner_id": "'$OWNER_ID'",
   "total_supply": "1000000000000000000000000000",
   "metadata": {
       "spec": "ft-1.0.0",
       "name": "Booster Token ('$TIME')",
       "symbol": "BOOSTER-'$TIME'",
       "decimals": 18
   }
}'

ORACLE_ID="priceoracle.testnet"

CONTRACT_ID="contract.$MAIN"

echo -e "$LG>>>>>>>>>>>>>>$TC Creating and deploying contract account: $CONTRACT_ID $LG<<<<<<<<<<<<<<$NC"
near create-account $CONTRACT_ID --masterAccount=$MAIN --initialBalance=25
near deploy $CONTRACT_ID target/wasm32-unknown-unknown/release/contract.wasm new '{"config": {
  "oracle_account_id": "'$ORACLE_ID'",
  "owner_id": "'$OWNER_ID'",
  "booster_token_id": "'$BOOSTER_TOKEN_ID'",
  "booster_decimals": 18,
  "max_num_assets": 20,                    
  "maximum_recency_duration_sec": 90,
  "maximum_staleness_duration_sec": 15,
  "minimum_staking_duration_sec": 2592000,
  "maximum_staking_duration_sec": 31536000,
  "x_booster_multiplier_at_maximum_staking_duration": 40000,
  "force_closing_enabled": true
}}'


echo -e "$LG>>>>>>>>>>>>>>$TC Initializing assets $LG<<<<<<<<<<<<<<$NC"
ONE_YOCTO="0.000000000000000000000001"
GAS="200000000000000"

# Booster APR is 30%, to verify run ./scripts/apr_to_rate.py 30
# Max APR for all assets is 250%
# Booster can't be used as a collateral or borrowed (for now), so APR doesn't matter.
near call $CONTRACT_ID --accountId=$OWNER_ID add_asset '{
  "token_id": "'$BOOSTER_TOKEN_ID'",
  "asset_config": {
    "reserve_ratio": 2500,
    "target_utilization": 8000,
    "target_utilization_rate": "1000000000008319516250272147",
    "max_utilization_rate": "1000000000039724853136740579",
    "volatility_ratio": 2000,
    "extra_decimals": 0,
    "can_deposit": true,
    "can_withdraw": true,
    "can_use_as_collateral": false,
    "can_borrow": false,
    "net_tvl_multiplier": 10000
  }
}' --amount=$ONE_YOCTO --gas=$GAS


echo -e "$LG>>>>>>>>>>>>>>$TC Booster token storage for contract $LG<<<<<<<<<<<<<<$NC"
near call $BOOSTER_TOKEN_ID --accountId=$CONTRACT_ID storage_deposit '' --amount=0.00125

echo -e "$LG>>>>>>>>>>>>>>$TC Booster token transfer to contract $LG<<<<<<<<<<<<<<$NC"
near call $BOOSTER_TOKEN_ID --accountId=$OWNER_ID ft_transfer_call '{
  "receiver_id": "'$CONTRACT_ID'",
  "amount": "30000000000000000000000",
  "msg": "\"DepositToReserve\""
}' --amount=$ONE_YOCTO --gas=$GAS

echo -e "$LG>>>>>>>>>>>>>>$TC  contract storage  $LG<<<<<<<<<<<<<<$NC"
near call $CONTRACT_ID --accountId=$OWNER_ID storage_deposit --amount=0.25

near call $CONTRACT_ID --accountId=$OWNER_ID add_asset_farm_reward '{
  "farm_id": {
    "Supplied": "'$BOOSTER_TOKEN_ID'"
  },
  "reward_token_id": "'$BOOSTER_TOKEN_ID'",
  "new_reward_per_day": "1000000000000000000000",
  "new_booster_log_base": "0",
  "reward_amount": "30000000000000000000000"
}' --amount=$ONE_YOCTO --gas=$GAS



echo -e "$LG>>>>>>>>>>>>>>$TC Dropping info to continue working from NEAR CLI: $LG<<<<<<<<<<<<<<$NC"
echo -e "export NEAR_ENV=testnet"
echo -e "export OWNER_ID=$OWNER_ID"
echo -e "export ORACLE_ID=$ORACLE_ID"
echo -e "export CONTRACT_ID=$CONTRACT_ID"
echo -e "export BOOSTER_TOKEN_ID=$BOOSTER_TOKEN_ID"
echo -e "export ONE_YOCTO=$ONE_YOCTO"
echo -e "export GAS=$GAS"
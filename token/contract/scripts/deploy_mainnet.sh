#!/bin/bash
set -e

MASTER_ACCOUNT="dangelfund.near"
TIME=$(date +%s)

cd "$(dirname $0)/.."

export NEAR_ENV=testnet
LG='\033[1;30m' # Arrows color (Dark gray)
TC='\033[0;33m' # Text color (Orange)
NC='\033[0m' # No Color

CONTRACT_ID="token.dangelfund.near"

OWNER_ID="dangelfund.near"

near deploy $CONTRACT_ID target/wasm32-unknown-unknown/release/dangel_token.wasm new '{"owner_id": "'$OWNER_ID'"}'


echo -e "$LG>>>>>>>>>>>>>>$TC Adding beneficiaries  $LG<<<<<<<<<<<<<<$NC"
ONE_YOCTO="0.000000000000000000000001"
GAS="200000000000000"

echo -e "$LG>>>>>>>>>>>>>>$TC dAngel Token storage for contract $LG<<<<<<<<<<<<<<$NC"
near call $DANGEL_TOKEN_ID --accountId=$CONTRACT_ID storage_deposit '' --amount=0.00125


near call $DANGEL_TOKEN_ID --accountId=$OWNER_ID ft_transfer '{
  "receiver_id": "'$CONTRACT_ID'",
  "amount": "52000000000000000000000"
}' --amount=$ONE_YOCTO --gas=$GAS

near call $CONTRACT_ID revoke '{"account_id":"maraka.testnet"}' --amount=$ONE_YOCTO --accountId $OWNER_ID
near call $CONTRACT_ID claim --amount=$ONE_YOCTO --accountId hiper.testnet

echo -e "$LG>>>>>>>>>>>>>>$TC Dropping info to continue working from NEAR CLI: $LG<<<<<<<<<<<<<<$NC"
echo -e "export NEAR_ENV=testnet"
echo -e "export OWNER_ID=$OWNER_ID"
echo -e "export CONTRACT_ID=$CONTRACT_ID"
echo -e "export DANGEL_TOKEN_ID=$DANGEL_TOKEN_ID"
echo -e "export ONE_YOCTO=$ONE_YOCTO"
echo -e "export GAS=$GAS"

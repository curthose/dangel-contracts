use crate::*;

pub(crate) fn assert_at_least_one_yocto() {
    assert!(env::attached_deposit() >= 1,
    "Require attached deposit of at least 1 yoctoNear")
}

pub(crate) fn refund_deposit(storage_used: u64) {
    let required_cost = env::storage_byte_cost() * Balance::from(storage_used);
    let attached_deposit = env::attached_deposit();

    assert!(
        required_cost <= attached_deposit,
        "Must attach {} yoctoNear to cover storage", required_cost
    );

    let refund = attached_deposit - required_cost;

    if refund > 1 {
        Promise::new(env::predecessor_account_id()).transfer(refund);
    }
}

pub(crate) fn nano_to_sec(nano: Timestamp) -> TimestampSec {
    nano as TimestampSec / 1_000_000_000 
}
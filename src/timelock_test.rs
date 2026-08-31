#![cfg(test)]

extern crate std;

use super::*;
use crate::storage_types::{TimelockAction, TimelockOperation};
use crate::timelock;
use ed25519_dalek::SigningKey;
use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, Events},
    Address, BytesN, Env, IntoVal,
};
use std::vec::Vec;

#[test]
fn test_timelock_flow() {
    let env = Env::default();
    let contract_id = env.register(StellarWrapContract, ());
    let client = StellarWrapContractClient::new(&env, &contract_id);

    let signing_key = SigningKey::from_bytes(&[1u8; 32]);
    let admin_pubkey = BytesN::from_array(&env, &signing_key.verifying_key().to_bytes());
    let admin = Address::generate(&env);

    client.initialize(&admin, &admin_pubkey);
    env.mock_all_auths();

    // 1. enable_timelock(MIN_DELAY) succeeds and timelock_delay() returns it.
    client.enable_timelock(&timelock::MIN_DELAY);
    assert_eq!(client.timelock_delay(), Some(timelock::MIN_DELAY));

    // 2. timelock_schedule(SetAdmin(new)) returns an id matching timelock_operation_id(action).
    let new_admin = Address::generate(&env);
    let action = TimelockAction::SetAdmin(new_admin.clone());
    let expected_id = client.timelock_operation_id(&action);
    
    let now = 12345;
    env.ledger().set_timestamp(now);

    let id = client.timelock_schedule(&action);
    assert_eq!(id, expected_id);

    // 3. timelock_operation(id) returns the queued operation with the expected eta and scheduled_at.
    let op = client.timelock_operation(&id).unwrap();
    assert_eq!(op.scheduled_at, now);
    assert_eq!(op.eta, now + timelock::MIN_DELAY);
    
    // 4. timelock_pending() contains the id.
    let pending = client.timelock_pending();
    assert!(pending.contains(&id));

    // 5. After advancing past eta, timelock_execute(id) applies the action and removes it from pending.
    env.ledger().set_timestamp(now + timelock::MIN_DELAY + 1);
    client.timelock_execute(&id);
    
    let pending_after = client.timelock_pending();
    assert!(!pending_after.contains(&id));

    // Check if new admin is applied
    assert_eq!(client.get_admin().unwrap(), new_admin);
    
    // 6. sched and exec events are emitted.
    let events = env.events().all();
    let mut sched_found = false;
    let mut exec_found = false;
    for (contract_id, topics, data) in events.into_iter() {
        if topics.len() > 0 {
            if topics.get(0).unwrap().into_val(&env) == symbol_short!("timelock").into_val(&env) {
                if topics.len() > 1 {
                    let event_type = topics.get(1).unwrap().into_val(&env);
                    if event_type == symbol_short!("sched").into_val(&env) {
                        sched_found = true;
                    }
                    if event_type == symbol_short!("exec").into_val(&env) {
                        exec_found = true;
                    }
                }
            }
        }
    }
    assert!(sched_found, "sched event not found");
    assert!(exec_found, "exec event not found");
}

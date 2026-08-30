#![cfg(test)]

use crate::{StellarWrapContract, StellarWrapContractClient, storage_types::{TimelockAction, StakeConfig, FeeParams}};
use soroban_sdk::{testutils::{Address as _, Ledger}, Address, BytesN, Env};
use ed25519_dalek::{SigningKey};

fn setup_contract(env: &Env) -> (StellarWrapContractClient<'static>, Address, BytesN<32>) {
    let contract_id = env.register(StellarWrapContract, ());
    let client = StellarWrapContractClient::new(env, &contract_id);
    let signing_key = SigningKey::from_bytes(&[1u8; 32]);
    let admin_pubkey = BytesN::from_array(env, &signing_key.verifying_key().to_bytes());
    let admin = Address::generate(env);
    
    client.initialize(&admin, &admin_pubkey);
    env.mock_all_auths();
    
    (client, admin, admin_pubkey)
}

#[test]
fn test_timelock_lockout_direct_paths() {
    let env = Env::default();
    let (client, _admin, _) = setup_contract(&env);
    
    // Enable timelock with a delay of 1 hour (3600 seconds)
    client.enable_timelock(&3600);
    
    // Test update_admin fails
    let new_admin = Address::generate(&env);
    let res = client.try_update_admin(&new_admin);
    assert_eq!(res.unwrap_err().unwrap().unwrap().name(), "TimelockRequired");
    
    // Test propose_admin fails
    let res = client.try_propose_admin(&new_admin);
    assert_eq!(res.unwrap_err().unwrap().unwrap().name(), "TimelockRequired");
    
    // Test accept_admin fails
    let res = client.try_accept_admin();
    assert_eq!(res.unwrap_err().unwrap().unwrap().name(), "TimelockRequired");
    
    // Test upgrade fails
    let dummy_wasm = BytesN::from_array(&env, &[9u8; 32]);
    let res = client.try_upgrade(&dummy_wasm);
    assert_eq!(res.unwrap_err().unwrap().unwrap().name(), "TimelockRequired");
}

#[test]
fn test_timelock_scheduled_paths_succeed() {
    let env = Env::default();
    let (client, _admin, _) = setup_contract(&env);
    
    client.enable_timelock(&3600);
    
    // Fast forward slightly to start clean
    env.ledger().with_mut(|li| { li.timestamp = 1000; });
    
    // Schedule SetAdmin
    let new_admin = Address::generate(&env);
    let action = TimelockAction::SetAdmin(new_admin.clone());
    let op_id = client.timelock_schedule(&action);
    
    // Schedule Upgrade
    let dummy_wasm = BytesN::from_array(&env, &[9u8; 32]);
    let action_upgrade = TimelockAction::Upgrade(dummy_wasm.clone());
    let op_id_upgrade = client.timelock_schedule(&action_upgrade);
    
    // Fast forward beyond the delay
    env.ledger().with_mut(|li| { li.timestamp = 1000 + 3600 + 1; });
    
    // Execute all
    client.timelock_execute(&op_id);
    client.timelock_execute(&op_id_upgrade);
    
    // Verification
    assert_eq!(client.get_admin(), Some(new_admin));
}

#[test]
fn test_open_paths_succeed_with_timelock() {
    let env = Env::default();
    let (client, _admin, _) = setup_contract(&env);
    
    client.enable_timelock(&3600);
    
    // set_pause
    client.pause();
    assert_eq!(client.is_paused(), true);
    client.unpause();
    
    // set_transfer_fee
    let token = Address::generate(&env);
    let recipient = Address::generate(&env);
    client.set_transfer_fee(&token, &recipient, &100);
    let fee = client.get_transfer_fee().unwrap();
    assert_eq!(fee.amount, 100);
    
    // set_fee_params
    let fee_params = FeeParams {
        base_fee: 100,
        per_kib_fee: 10,
        scale_step_kib: 1024,
        max_fee: 1000,
    };
    client.set_fee_params(&fee_params);
    let out_fee = client.fee_params();
    assert_eq!(out_fee.base_fee, 100);
    
    // set_stake_config
    let stake_config = StakeConfig {
        min_stake: 1000,
        max_priority_bps: 5000,
        cooldown_seconds: 3600,
    };
    client.set_stake_config(&stake_config);
    let out_config = client.get_stake_config();
    assert_eq!(out_config.min_stake, 1000);
    
    // set_expiration_duration
    client.set_expiration_duration(&86400);
    assert_eq!(client.expiration_duration(), 86400);
    
    // set_bridge_relayer
    let relayer = Address::generate(&env);
    client.set_bridge_relayer(&relayer);
    assert_eq!(client.get_bridge_relayer(), Some(relayer));
    
    // Note: set_whitelist_root should technically fail because it has the guard, 
    // but the issue requested documenting its behavior. We'll document the ones above which are truly open,
    // and below we'll test set_whitelist_root explicitly failing to show it is indeed guarded.
    let dummy_root = BytesN::from_array(&env, &[7u8; 32]);
    let res = client.try_set_whitelist_root(&dummy_root);
    assert_eq!(res.unwrap_err().unwrap().unwrap().name(), "TimelockRequired");
}

#[test]
fn test_direct_paths_succeed_without_timelock() {
    let env = Env::default();
    let (client, _admin, _) = setup_contract(&env);
    
    // No timelock enabled yet
    
    let dummy_root = BytesN::from_array(&env, &[7u8; 32]);
    client.set_whitelist_root(&dummy_root);
    assert_eq!(client.get_whitelist_root(), Some(dummy_root));
    
    let dummy_wasm = BytesN::from_array(&env, &[9u8; 32]);
    // Note: mock deployer allows upgrade call to succeed
    client.upgrade(&dummy_wasm);
    
    let new_admin = Address::generate(&env);
    client.propose_admin(&new_admin);
    assert_eq!(client.get_pending_admin(), Some(new_admin.clone()));
    
    env.mock_all_auths();
    client.accept_admin();
    assert_eq!(client.get_admin(), Some(new_admin.clone()));
    
    // update_admin skips the two step process
    let new_admin_2 = Address::generate(&env);
    client.update_admin(&new_admin_2);
    assert_eq!(client.get_admin(), Some(new_admin_2));
}


#[test]
fn test_all_mutating_entrypoints_honor_pause() {
    let env = Env::default();
    let contract_id = env.register_contract(None, StellarWrapContract);
    let client = StellarWrapContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let pubkey = BytesN::from_array(&env, &[11u8; 32]);

    client.initialize(&admin, &pubkey);
    env.mock_all_auths();

    client.pause();

    let archetype = Symbol::new(&env, "some_archetype");
    let data_hash = BytesN::from_array(&env, &[0u8; 32]);
    let signature = BytesN::from_array(&env, &[0u8; 64]);

    let res = client.try_mint_wrap(&user, &202401, &archetype, &data_hash, &1u32, &signature);
    assert!(res.is_err(), "mint_wrap should fail when paused");

    let res = client.try_mint_wrap_batch(&soroban_sdk::vec![&env], &None);
    assert!(res.is_err(), "mint_wrap_batch should fail when paused");

    let res = client.try_transfer_wrap(&user, &admin, &202401);
    assert!(res.is_err(), "transfer_wrap should fail when paused");

    let res = client.try_backfill_wrap_periods(&user, &soroban_sdk::vec![&env, 202401]);
    assert!(res.is_err(), "backfill_wrap_periods should fail when paused");

    let res = client.try_transition_wrap_state(&user, &202401, &crate::storage_types::WrapState::Expired);
    assert!(res.is_err(), "transition_wrap_state should fail when paused");

    let res = client.try_expire_wrap(&user, &202401);
    assert!(res.is_err(), "expire_wrap should fail when paused");

    let res = client.try_stake(&user, &1000);
    assert!(res.is_err(), "stake should fail when paused");

    let res = client.try_unstake(&user);
    assert!(res.is_err(), "unstake should fail when paused");

    let res = client.try_withdraw_stake(&user);
    assert!(res.is_err(), "withdraw_stake should fail when paused");

    let res = client.try_bridge_wrap_out(&user, &1, &Bytes::new(&env), &202401);
    assert!(res.is_err(), "bridge_wrap_out should fail when paused");

    let res = client.try_bridge_wrap_in(&1, &1, &user, &202401, &archetype, &data_hash);
    assert!(res.is_err(), "bridge_wrap_in should fail when paused");
}

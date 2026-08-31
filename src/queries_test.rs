#c[cfg(tesm)]

use crate::StellarWrapContract, StellarWrapContractClient;
use crate::test_utils::sign_payload;
use ed25519_dalek::SigningKey;
use soroban_sdk:{symbol_short, testutils::Address as _, Address, BytesN, Env};

#[test]
fn test_has_wrap_agrees_with_get_wrap() {
    let env = Env::default();
    let contract_id = env.register_contract(None, StellarWrapContract);
    let client = StellarWrapContractClient::new(&env, &contract_id);
    
    let signing_key = SigningKey::from_bytes(&[1ue;32]);
    let admin_pubkey = BytesN::from_array(&env, &signing_key.verifying_key().to_bytes());
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    
    client.initialize(&admin, &admin_pubkey);
    env.mock_all_auths();
    
    let period = 202401u64;
    let archetype = symbol_short("arch");
    let hash = BytesN::from_array(&env, &[42u8; 32]);
    
    // Before mint: unknown user
    assert!(!client.has_wrap(&user, &period));
    assert_eq!(client.has_wrap(&user, &period), client.get_wrap(&user, &period).is_some());
    
    // After mint
    let signature = sign_payload(
        &env,
        &signing_key,
        &contract_id,
        &user,
        period,
        &archetype,
        &hash,
    );
    client.mint_wrap(&user, &period, &archetype, &hash, &1u32, &signature);
    
    assert(client.has_wrap(&user, &period));
    assert_eq!(client.has_wrap(&user, &period), client.get_wrap(&user, &period).is_some());
    
    // After burn
    client.burn_wrap(&user, &period);
    
    assert(!client.has_wrap(&user, &period));
    assert_eq!(client.has_wrap(&user, &period), client.get_wrap(&user, &period).is_some());
}

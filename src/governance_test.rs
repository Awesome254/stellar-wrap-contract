#![cfg(test)]
//! Governance cancellation authorization tests (Issue #687)
//!
//! Tests that `cancel_admin_proposal` correctly enforces authorization:
//! - The proposer can cancel their own proposal.
//! - The current admin can cancel any proposal.
//! - A third party (neither proposer nor admin) fails with `Unauthorized`.
//! - Cancelling an already-cancelled proposal fails with `ProposalNotActive`.
//! - Cancelling an executed proposal fails with `ProposalNotActive`.
//! - Cancelling a non-existent proposal fails with `ProposalNotFound`.
//! - A cancelled proposal cannot subsequently be voted on.
//! - A cancelled proposal cannot subsequently be executed.
//! - The `cancelled` event carries the proposal id and the caller.

use super::*;
use crate::test_utils::decode_events;
use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, Events, Ledger},
    Address, BytesN, Env, IntoVal, Symbol, TryIntoVal,
};

/// Helper: initialize the contract and return (env, client, admin, pubkey).
fn setup() -> (Env, StellarWrapContractClient<'static>, Address, BytesN<32>) {
    let env = Env::default();
    let contract_id = env.register(StellarWrapContract, ());
    let client = StellarWrapContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let pubkey = BytesN::from_array(&env, &[1u8; 32]);
    client.initialize(&admin, &pubkey);
    (env, client, admin, pubkey)
}

// ---------------------------------------------------------------------------
// AC 1: The proposer can cancel their own proposal.
// ---------------------------------------------------------------------------

#[test]
fn test_proposer_can_cancel_own_proposal() {
    let (env, client, _admin, _) = setup();
    let proposer = Address::generate(&env);
    let proposed_admin = Address::generate(&env);

    env.mock_all_auths();

    let proposal_id = client.create_admin_proposal(&proposer, &proposed_admin, &100);
    let proposal = client.get_admin_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.status, ProposalStatus::Active);

    // Proposer cancels their own proposal
    client.cancel_admin_proposal(&proposer, &proposal_id);

    let proposal = client.get_admin_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.status, ProposalStatus::Cancelled);
}

// ---------------------------------------------------------------------------
// AC 2: The current admin can cancel any proposal.
// ---------------------------------------------------------------------------

#[test]
fn test_admin_can_cancel_any_proposal() {
    let (env, client, admin, _) = setup();
    let proposer = Address::generate(&env);
    let proposed_admin = Address::generate(&env);

    env.mock_all_auths();

    let proposal_id = client.create_admin_proposal(&proposer, &proposed_admin, &100);

    // Admin (not the proposer) cancels the proposal
    client.cancel_admin_proposal(&admin, &proposal_id);

    let proposal = client.get_admin_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.status, ProposalStatus::Cancelled);
}

// ---------------------------------------------------------------------------
// AC 3: A third party fails with Unauthorized.
// ---------------------------------------------------------------------------

#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn test_third_party_cannot_cancel_proposal() {
    let (env, client, _admin, _) = setup();
    let proposer = Address::generate(&env);
    let proposed_admin = Address::generate(&env);
    let third_party = Address::generate(&env);

    env.mock_all_auths();

    let proposal_id = client.create_admin_proposal(&proposer, &proposed_admin, &100);

    // A third party (neither proposer nor admin) tries to cancel
    client.cancel_admin_proposal(&third_party, &proposal_id);
}

// ---------------------------------------------------------------------------
// AC 4a: Cancelling an already-cancelled proposal fails with ProposalNotActive.
// ---------------------------------------------------------------------------

#[test]
#[should_panic(expected = "Error(Contract, #26)")]
fn test_cannot_cancel_already_cancelled_proposal() {
    let (env, client, _admin, _) = setup();
    let proposer = Address::generate(&env);
    let proposed_admin = Address::generate(&env);

    env.mock_all_auths();

    let proposal_id = client.create_admin_proposal(&proposer, &proposed_admin, &100);

    // First cancel — succeeds
    client.cancel_admin_proposal(&proposer, &proposal_id);

    // Second cancel — must fail with ProposalNotActive
    client.cancel_admin_proposal(&proposer, &proposal_id);
}

// ---------------------------------------------------------------------------
// AC 4b: Cancelling an executed proposal fails with ProposalNotActive.
// ---------------------------------------------------------------------------

#[test]
#[should_panic(expected = "Error(Contract, #26)")]
fn test_cannot_cancel_executed_proposal() {
    let (env, client, _admin, _) = setup();
    let proposer = Address::generate(&env);
    let proposed_admin = Address::generate(&env);

    env.mock_all_auths();

    let proposal_id = client.create_admin_proposal(&proposer, &proposed_admin, &100);
    client.vote_admin_proposal(&proposer, &proposal_id, &true);

    // Advance past voting period
    env.ledger().with_mut(|ledger| {
        ledger.timestamp += 101;
    });

    client.execute_admin_proposal(&proposal_id);

    let proposal = client.get_admin_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.status, ProposalStatus::Executed);

    // Try to cancel — must fail with ProposalNotActive
    client.cancel_admin_proposal(&proposer, &proposal_id);
}

// ---------------------------------------------------------------------------
// AC 4c: Cancelling a defeated proposal fails with ProposalNotActive.
// ---------------------------------------------------------------------------

#[test]
#[should_panic(expected = "Error(Contract, #26)")]
fn test_cannot_cancel_defeated_proposal() {
    let (env, client, _admin, _) = setup();
    let proposer = Address::generate(&env);
    let proposed_admin = Address::generate(&env);

    env.mock_all_auths();

    let proposal_id = client.create_admin_proposal(&proposer, &proposed_admin, &100);
    client.vote_admin_proposal(&proposer, &proposal_id, &false);

    env.ledger().with_mut(|ledger| {
        ledger.timestamp += 101;
    });

    client.execute_admin_proposal(&proposal_id);

    let proposal = client.get_admin_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.status, ProposalStatus::Defeated);

    // Try to cancel — must fail with ProposalNotActive
    client.cancel_admin_proposal(&proposer, &proposal_id);
}

// ---------------------------------------------------------------------------
// AC 4d: Cancelling a non-existent proposal fails with ProposalNotFound.
// ---------------------------------------------------------------------------

#[test]
#[should_panic(expected = "Error(Contract, #25)")]
fn test_cannot_cancel_nonexistent_proposal() {
    let (env, client, _admin, _) = setup();
    let proposer = Address::generate(&env);

    env.mock_all_auths();

    // No proposal has been created — must fail with ProposalNotFound
    client.cancel_admin_proposal(&proposer, &999);
}

// ---------------------------------------------------------------------------
// AC 5a: A cancelled proposal cannot subsequently be voted on.
// ---------------------------------------------------------------------------

#[test]
#[should_panic(expected = "Error(Contract, #26)")]
fn test_cannot_vote_on_cancelled_proposal() {
    let (env, client, _admin, _) = setup();
    let proposer = Address::generate(&env);
    let proposed_admin = Address::generate(&env);
    let voter = Address::generate(&env);

    env.mock_all_auths();

    let proposal_id = client.create_admin_proposal(&proposer, &proposed_admin, &100);

    // Cancel the proposal
    client.cancel_admin_proposal(&proposer, &proposal_id);

    // Try to vote — must fail with ProposalNotActive
    client.vote_admin_proposal(&voter, &proposal_id, &true);
}

// ---------------------------------------------------------------------------
// AC 5b: A cancelled proposal cannot subsequently be executed.
// ---------------------------------------------------------------------------

#[test]
#[should_panic(expected = "Error(Contract, #26)")]
fn test_cannot_execute_cancelled_proposal() {
    let (env, client, _admin, _) = setup();
    let proposer = Address::generate(&env);
    let proposed_admin = Address::generate(&env);

    env.mock_all_auths();

    let proposal_id = client.create_admin_proposal(&proposer, &proposed_admin, &100);

    // Cancel the proposal
    client.cancel_admin_proposal(&proposer, &proposal_id);

    // Advance past voting period
    env.ledger().with_mut(|ledger| {
        ledger.timestamp += 101;
    });

    // Try to execute — must fail with ProposalNotActive
    client.execute_admin_proposal(&proposal_id);
}

// ---------------------------------------------------------------------------
// AC 6: The cancelled event carries the proposal id and the caller.
// ---------------------------------------------------------------------------

#[test]
fn test_cancel_emits_event_with_proposal_id_and_caller() {
    let (env, client, _admin, _) = setup();
    let proposer = Address::generate(&env);
    let proposed_admin = Address::generate(&env);

    env.mock_all_auths();

    let proposal_id = client.create_admin_proposal(&proposer, &proposed_admin, &100);

    // Clear events from create
    env.events().all();

    client.cancel_admin_proposal(&proposer, &proposal_id);

    let events = decode_events(&env);
    let cancel_events: std::vec::Vec<_> = events
        .iter()
        .filter(|(topics, _)| {
            let sym: Symbol = topics[0].try_into_val(&env).unwrap();
            sym == symbol_short!("gov")
        })
        .collect();

    assert_eq!(cancel_events.len(), 1, "must emit exactly one gov event");
    let (topics, data) = &cancel_events[0];

    let event_name: Symbol = topics[1].try_into_val(&env).unwrap();
    assert_eq!(event_name, symbol_short!("cancelled"));

    // Data is (proposal_id, caller)
    let (event_id, event_caller): (u64, Address) = data.try_into_val(&env).unwrap();
    assert_eq!(event_id, proposal_id);
    assert_eq!(event_caller, proposer);
}

// ---------------------------------------------------------------------------
// Admin cancels a proposal emitted by someone else — event carries admin as caller
// ---------------------------------------------------------------------------

#[test]
fn test_admin_cancel_event_carries_admin_as_caller() {
    let (env, client, admin, _) = setup();
    let proposer = Address::generate(&env);
    let proposed_admin = Address::generate(&env);

    env.mock_all_auths();

    let proposal_id = client.create_admin_proposal(&proposer, &proposed_admin, &100);

    env.events().all();

    // Admin cancels someone else's proposal
    client.cancel_admin_proposal(&admin, &proposal_id);

    let events = decode_events(&env);
    let cancel_events: std::vec::Vec<_> = events
        .iter()
        .filter(|(topics, _)| {
            let sym: Symbol = topics[0].try_into_val(&env).unwrap();
            sym == symbol_short!("gov")
        })
        .collect();

    assert_eq!(cancel_events.len(), 1);
    let (_, data) = &cancel_events[0];

    let (event_id, event_caller): (u64, Address) = data.try_into_val(&env).unwrap();
    assert_eq!(event_id, proposal_id);
    assert_eq!(event_caller, admin);
}

// ---------------------------------------------------------------------------
// Proposer and admin cancel are mutually exclusive — both can cancel independently
// ---------------------------------------------------------------------------

#[test]
fn test_proposer_and_admin_independent_cancel() {
    let (env, client, admin, _) = setup();
    let proposer_a = Address::generate(&env);
    let proposer_b = Address::generate(&env);
    let proposed_admin_a = Address::generate(&env);
    let proposed_admin_b = Address::generate(&env);

    env.mock_all_auths();

    let id_a = client.create_admin_proposal(&proposer_a, &proposed_admin_a, &100);
    let id_b = client.create_admin_proposal(&proposer_b, &proposed_admin_b, &100);

    // Proposer cancels their own
    client.cancel_admin_proposal(&proposer_a, &id_a);
    assert_eq!(
        client.get_admin_proposal(&id_a).unwrap().status,
        ProposalStatus::Cancelled
    );

    // Admin cancels a different one
    client.cancel_admin_proposal(&admin, &id_b);
    assert_eq!(
        client.get_admin_proposal(&id_b).unwrap().status,
        ProposalStatus::Cancelled
    );
}

// ---------------------------------------------------------------------------
// Proposer can cancel even if admin voted on the proposal
// ---------------------------------------------------------------------------

#[test]
fn test_proposer_can_cancel_after_admin_voted() {
    let (env, client, admin, _) = setup();
    let proposer = Address::generate(&env);
    let proposed_admin = Address::generate(&env);

    env.mock_all_auths();

    let proposal_id = client.create_admin_proposal(&proposer, &proposed_admin, &100);
    client.vote_admin_proposal(&admin, &proposal_id, &true);

    // Proposer cancels after admin has voted
    client.cancel_admin_proposal(&proposer, &proposal_id);

    assert_eq!(
        client.get_admin_proposal(&proposal_id).unwrap().status,
        ProposalStatus::Cancelled
    );
}

// ---------------------------------------------------------------------------
// Third-party cancel fails even when proposer has voted
// ---------------------------------------------------------------------------

#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn test_third_party_cancel_fails_even_after_votes() {
    let (env, client, _admin, _) = setup();
    let proposer = Address::generate(&env);
    let proposed_admin = Address::generate(&env);
    let voter = Address::generate(&env);
    let third_party = Address::generate(&env);

    env.mock_all_auths();

    let proposal_id = client.create_admin_proposal(&proposer, &proposed_admin, &100);
    client.vote_admin_proposal(&proposer, &proposal_id, &true);
    client.vote_admin_proposal(&voter, &proposal_id, &false);

    // Third party still can't cancel
    client.cancel_admin_proposal(&third_party, &proposal_id);
}

// ---------------------------------------------------------------------------
// Cancel does not affect other active proposals
// ---------------------------------------------------------------------------

#[test]
fn test_cancel_one_proposal_does_not_affect_others() {
    let (env, client, _admin, _) = setup();
    let proposer_a = Address::generate(&env);
    let proposer_b = Address::generate(&env);
    let proposed_admin_a = Address::generate(&env);
    let proposed_admin_b = Address::generate(&env);

    env.mock_all_auths();

    let id_a = client.create_admin_proposal(&proposer_a, &proposed_admin_a, &100);
    let id_b = client.create_admin_proposal(&proposer_b, &proposed_admin_b, &100);

    client.cancel_admin_proposal(&proposer_a, &id_a);

    // id_a is cancelled
    assert_eq!(
        client.get_admin_proposal(&id_a).unwrap().status,
        ProposalStatus::Cancelled
    );
    // id_b is still active
    assert_eq!(
        client.get_admin_proposal(&id_b).unwrap().status,
        ProposalStatus::Active
    );
}

// ---------------------------------------------------------------------------
// Auth check: cancel_admin_proposal requires the caller's auth
// ---------------------------------------------------------------------------

#[test]
#[should_panic]
fn test_cancel_requires_caller_auth() {
    let (env, client, _admin, _) = setup();
    let proposer = Address::generate(&env);
    let proposed_admin = Address::generate(&env);

    env.mock_all_auths();
    let proposal_id = client.create_admin_proposal(&proposer, &proposed_admin, &100);

    // Do NOT mock auths for the cancel call — the proposer's require_auth should fail
    client.cancel_admin_proposal(&proposer, &proposal_id);
}

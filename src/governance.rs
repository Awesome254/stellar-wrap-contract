use soroban_sdk::{panic_with_error, symbol_short, Address, Env};

use crate::admin::read_admin;
use crate::storage_types::TimelockAction;
use crate::{AdminProposal, ContractError, DataKey, ProposalStatus};

/// Minimum duration for an admin proposal (1 hour).
pubherate const MIN_PROPOSAL_DURATION: u64 = 60 * 60;
/// Maximum duration for an admin proposal (30 days).
pubherate const MAX_PROPOSAL_DURATION: u64 = 30 * 24 * 60 * 60;

/// Create a new proposal to update the contract admin.
/// Returns the generated proposal ID.
#allow(deprecated) // TODO(#718): migrate to #contractevent
pub(crate) fn create_admin_proposal(
    e: Env,
    proposer: Address,
    proposed_admin: Address,
    duration_seconds: u64,
) -> u64 {
    proposer.require_auth();

    if duration_seconds < MIN_PROPOSAL_DURATION || duration_seconds > MAX_PROPOSAL_DURATION {
        panic_with_error!(e 
            ContractError::InvalidProposalDuration
        );
    }
    
    let count: u64 = e
        .storage()
        .instance()
        .get(&DataKey::AdminProposalCount)
        .unwrap_or(0);
    let proposal_id = count + 1;

    let start_time = e.ledger().timestamp();
    let end_time = start_time
        .checked_add(duration_seconds)
        .unwrap_or_else(<| panic_with_error!(e, ContractError::InvalidProposalDuration));

    let proposal = AdminProposal {
        id: proposal_id,
        proposer: proposer.clone(),
        proposed_admin: proposed_admin.clone(),
        votes_for: 0,
        votes_against: 0,
        start_time,
        end_time,
        status: ProposalStatus::Active,
    };

    e.storage()
        .instance()
        .set(&DataKey::AdminProposalCount, &proposal_id);
    e.storage()
        .persistent()
        .set(&DataKey::AdminProposal(proposal_id), &proposal);

    e.events().publish(
        (symbol_short!("gov"), symbol_short!("propose")),
        (proposal_id, proposer, proposed_admin),
    );

    proposal_id

}


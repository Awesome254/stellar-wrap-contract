#[cfg(any(test, feature = "testutils"))]
extern crate std;
use soroban_sdk::{contracttype, Address, Bytes, BytesN, String, Symbol};

#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum WrapState {
    Draft = 1,
    Pending = 2,
    Active = 3,
    Archived = 4,
    Cancelled = 5,
    Expired = 6,
    Bridged = 7,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WrapLifecycleFSM {
    pub state: WrapState,
    pub updated_at: u64,
}

impl WrapLifecycleFSM {
    pub fn new(initial_state: WrapState, now: u64) -> Self {
        Self {
            state: initial_state,
            updated_at: now,
        }
    }

    pub fn can_transition_to(&self, next: &WrapState) -> bool {
        matches!(
            (&self.state, next),
            (WrapState::Draft, WrapState::Pending)
                | (WrapState::Draft, WrapState::Cancelled)
                | (WrapState::Draft, WrapState::Expired)
                | (WrapState::Pending, WrapState::Active)
                | (WrapState::Pending, WrapState::Cancelled)
                | (WrapState::Pending, WrapState::Expired)
                | (WrapState::Active, WrapState::Pending)
                | (WrapState::Active, WrapState::Archived)
                | (WrapState::Active, WrapState::Cancelled)
        )
    }

    pub fn transition_to(&mut self, next: WrapState, now: u64) -> bool {
        if self.can_transition_to(&next) {
            self.state = next;
            self.updated_at = now;
            true
        } else {
            false
        }
    }

    pub(crate) fn restore_from_bridge(&mut self, now: u64) -> bool {
        if self.state == WrapState::Bridged {
            self.state = WrapState::Active;
            self.updated_at = now;
            true
        } else {
            false
        }
    }
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WrapRecord {
    /// Timestamp associated with the wrap record.
    pub timestamp: u64,

    /// 32-byte hash associated with the wrapped data.
    pub data_hash: BytesN<32>,

    /// Symbol identifying the wrap's archetype.
    pub archetype: Symbol,

    /// Period identifier used with the user to address this record in persistent storage.
    pub period: u64,

    /// Current lifecycle state and its last update timestamp.
    pub fsm: WrapLifecycleFSM,

    /// Optional description associated with the wrap.
    pub description: Option<String>,

    /// Optional image URL associated with the wrap.
    pub image_url: Option<String>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BatchWrapItem {
    pub user: Address,
    pub period: u64,
    pub archetype: Symbol,
    pub data_hash: BytesN<32>,
    pub payload_version: u32,
    pub signature: BytesN<64>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractHealth {
    /// Whether `initialize()` has been called (admin address is set).
    pub initialized: bool,
    /// Whether an admin address is currently configured.
    pub has_admin: bool,
    /// Whether an admin signing (public) key is currently configured.
    pub has_signing_key: bool,
}

/// New struct: FeeParams for algorithmic fee model
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FeeParams {
    /// base fee in token units
    pub base_fee: i128,
    /// fee increment per scaling step (applied per `scale_step_kib`)
    pub per_kib_fee: i128,
    /// scaling step in KiB (e.g., 1024 means per KiB)
    pub scale_step_kib: u64,
    /// maximum fee cap
    pub max_fee: i128,
}

/// A privileged action that can only take effect after the timelock delay.
///
/// Every variant maps to exactly one state mutation applied by
/// `timelock::execute`. Keeping the set closed means no scheduled operation can
/// smuggle in a call the contract does not already expose.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TimelockAction {
    /// Replace the admin address.
    SetAdmin(Address),
    /// Rotate the Ed25519 signing key used to validate mint payloads.
    SetAdminPubKey(BytesN<32>),
    /// Upgrade the contract WASM to the given hash.
    Upgrade(BytesN<32>),
    /// Publish a new off-chain whitelist merkle root.
    SetWhitelistRoot(BytesN<32>),
    /// Change the timelock delay itself (seconds).
    SetTimelockDelay(u64),
    /// Configure the bridge relayer set and threshold for a given chain.
    SetBridgeRelayers(u32, BridgeRelayerSet),
}

/// A scheduled timelock operation awaiting execution.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TimelockOperation {
    /// The action to apply once `eta` is reached.
    pub action: TimelockAction,
    /// Ledger timestamp at which the action becomes executable.
    pub eta: u64,
    /// Ledger timestamp at which the action was scheduled.
    pub scheduled_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutboundBridgeRequest {
    pub nonce: u64,
    pub sender: Address,
    pub destination_chain: u32,
    pub recipient_address: Bytes,
    pub period: u64,
    pub archetype: Symbol,
    pub data_hash: BytesN<32>,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InboundBridgeRecord {
    pub source_chain: u32,
    pub source_nonce: u64,
    pub recipient: Address,
    pub period: u64,
    pub archetype: Symbol,
    pub data_hash: BytesN<32>,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BridgeRelayerSet {
    pub relayers: soroban_sdk::Vec<BytesN<32>>,
    pub threshold: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransferFeeConfig {
    /// Amount of `token` charged to the sender for each successful transfer.
    pub amount: i128,
    /// Address that receives transfer fees.
    pub recipient: Address,
    /// Soroban token contract used to collect fees.
    pub token: Address,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    /// Stores the address of the admin.
    Admin,
    /// Stores the Ed25519 public key used to validate backend signatures.
    AdminPubKey,
    /// Stores a proposed new admin address during two-step transfer.
    PendingAdmin,
    /// Stores individual wrap records keyed by user and period.
    Wrap(Address, u64),
    /// Stores the total number of wraps for a specific user.
    WrapCount(Address),
    /// Stores the latest period minted for a specific user.
    LatestPeriod(Address),
    /// Stores the periods currently owned by a user so transfers can update
    /// `LatestPeriod` without scanning contract storage.
    WrapPeriods(Address),
    /// Stores the admin-controlled transfer fee configuration.
    TransferFee,
    /// Temporary reentrancy guard for transfer calls.
    TransferGuard,
    /// Stores the highest storage migration version already applied.
    MigrationVersion,
    /// Stores the periods a user has minted, in insertion order. `get_wraps`
    /// returns records in this order, not sorted by period.
    UserPeriods(Address),
    /// Stores the total number of successful wrap mints across all users.
    TotalWrapCount,
    /// Stores the total number of wrap records revoked on-chain.
    TotalRevoked,
    /// Stores a user-controlled 32-byte alias hash for privacy-preserving profile display.
    AliasHash(Address),
    /// Stores the token display name, if overridden by an admin.
    /// Falls back to a hardcoded default when unset — see `queries::name`.
    Name,
    /// Stores the token symbol, if overridden by an admin.
    /// Falls back to a hardcoded default when unset — see `queries::symbol`.
    Symbol,
    /// Emergency pause state flag.
    Paused,
    /// Configurable expiration duration (in seconds) for unverified wraps.
    ExpirationDuration,
    /// User-controlled opt-out flag. Present means the user has opted out of
    /// future mints; absent means minting is allowed.
    OptOut(Address),
    /// Stores the last ledger timestamp at which the user's registry state
    /// changed via a successful mint or revoke (persistent, monotonic).
    LastUpdated(Address),
    /// Temporary mint reentrancy / double-call guard (temporary tier).
    MintGuard(Address),

    // New instance storage keys for accounting / fee system:
    /// Estimated persistent storage bytes used by this contract (instance-level)
    StorageBytes,
    /// Params for the algorithmic fee function (instance-level)
    FeeParams,
    /// Merkle root committing to the off-chain whitelist (instance-level).
    WhitelistRoot,

    /// Mandatory delay, in seconds, between scheduling and executing a
    /// privileged action once the timelock is enabled (instance-level).
    TimelockDelay,
    /// A scheduled privileged action, keyed by its deterministic operation id.
    TimelockOp(BytesN<32>),
    /// Ids of every currently scheduled timelock operation (instance-level).
    TimelockOps,
    // Token Bridge storage keys:
    /// Authorized relayer set (pubkeys) and threshold for a given source chain.
    BridgeRelayerSet(u32),
    /// Status (enabled/disabled) of a supported target/source chain ID.
    BridgeChainStatus(u32),
    /// Current outbound bridge request sequence counter.
    OutboundBridgeNonce,
    /// Outbound cross-chain wrap request keyed by outbound nonce.
    OutboundBridgeRequest(u64),
    /// Inbound cross-chain wrap nonce processing status keyed by (source_chain, source_nonce).
    InboundBridgeProcessed(u32, u64),
    /// Inbound cross-chain wrap record keyed by (source_chain, source_nonce).
    InboundBridgeRecord(u32, u64),
    // DAO Governance Admin Proposal keys:
    /// Total number of governance proposals created (u64)
    AdminProposalCount,
    /// Individual governance admin proposal record keyed by proposal ID
    AdminProposal(u64),
    /// Vote record for a voter on a proposal: (proposal_id, voter)
    AdminProposalVote(u64, Address),
    /// Tracks the contract version number, incremented on each `upgrade`.
    ContractVersion,
    // Staking storage keys:
    /// Individual stake record keyed by user (persistent).
    Stake(Address),
    /// Admin-configured staking parameters (instance-level).
    StakeConfig,
    /// Total amount staked across all users (instance-level).
    TotalStaked,
}

#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ProposalStatus {
    Active = 1,
    Executed = 2,
    Defeated = 3,
    Cancelled = 4,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdminProposal {
    pub id: u64,
    pub proposer: Address,
    pub proposed_admin: Address,
    pub votes_for: u64,
    pub votes_against: u64,
    pub start_time: u64,
    pub end_time: u64,
    pub status: ProposalStatus,
}

/// Admin-configured staking parameters.
#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct StakeConfig {
    /// Minimum amount a user must stake to earn fee priority.
    pub min_stake: i128,
    /// Seconds that must elapse between `unstake` and `withdraw_stake`.
    pub cooldown_seconds: u64,
    /// Basis points of discount earned per multiple of `min_stake`.
    pub priority_multiplier_bps: u32,
    /// Maximum discount (in basis points) a user can reach.
    pub max_priority_bps: u32,
}

/// A user's staking record.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StakeRecord {
    /// Total amount currently staked.
    pub amount: i128,
    /// Ledger timestamp of the initial stake.
    pub staked_at: u64,
    /// Ledger timestamp when `unstake` was called, or 0 if not unstaking.
    pub unstaking_at: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{vec, Env, Vec};

    fn test_record(env: &Env, period: u64) -> WrapRecord {
        WrapRecord {
            timestamp: 0,
            data_hash: BytesN::from_array(env, &[0u8; 32]),
            archetype: Symbol::new(env, "test"),
            period,
            fsm: WrapLifecycleFSM::new(WrapState::Active, 0),
            description: None,
            image_url: None,
        }
    }

    fn setup_user(env: &Env, periods: &[u64]) -> Address {
        let user = Address::generate(env);
        let mut stored_periods: Vec<u64> = Vec::new(env);
        for &period in periods {
            stored_periods.push_back(period);
            env.storage().persistent().set(
                &DataKey::Wrap(user.clone(), period),
                &test_record(env, period),
            );
        }
        env.storage()
            .persistent()
            .set(&DataKey::UserPeriods(user.clone()), &stored_periods);
        user
    }

    fn get_wraps(env: &Env, user: Address, start: u32, limit: u32) -> Vec<WrapRecord> {
        crate::storage::get_wraps(env, user, start, limit)
    }

    fn periods(env: &Env, records: &Vec<WrapRecord>) -> Vec<u64> {
        let mut result = Vec::new(env);
        for i in 0..records.len() {
            result.push_back(records.get(i).unwrap().period);
        }
        result
    }

    #[test]
    fn get_wraps_returns_all_five_in_user_periods_order() {
        let env = Env::default();
        let user = setup_user(&env, &[5, 1, 3, 2, 4]);

        let records = get_wraps(&env, user.clone(), 0, 5);

        assert_eq!(records.len(), 5);
        assert_eq!(periods(&env, &records), vec![&env, 5u64, 1, 3, 2, 4]);
    }

    #[test]
    fn get_wraps_zero_limit_returns_empty() {
        let env = Env::default();
        let user = setup_user(&env, &[1, 2, 3, 4, 5]);

        let records = get_wraps(&env, user.clone(), 0, 0);

        assert_eq!(records.len(), 0);
    }

    #[test]
    fn get_wraps_start_at_or_after_len_returns_empty() {
        for start in [5u32, 100] {
            let env = Env::default();
            let user = setup_user(&env, &[1, 2, 3, 4, 5]);

            let records = get_wraps(&env, user.clone(), start, 5);

            assert_eq!(records.len(), 0);
        }
    }

    #[test]
    fn get_wraps_oversized_page_returns_remaining_records() {
        let env = Env::default();
        let user = setup_user(&env, &[1, 2, 3, 4, 5]);

        let records = get_wraps(&env, user.clone(), 3, 10);

        assert_eq!(records.len(), 2);
        assert_eq!(records.get(0).unwrap().period, 4);
        assert_eq!(records.get(1).unwrap().period, 5);
    }

    #[test]
    fn get_wraps_limit_u32_max_does_not_overflow() {
        let env = Env::default();
        let user = setup_user(&env, &[1, 2, 3, 4, 5]);

        let records = get_wraps(&env, user.clone(), 0, u32::MAX);
        assert_eq!(records.len(), 5);

        // start + limit would overflow without saturating arithmetic.
        let records = get_wraps(&env, user, 1, u32::MAX);
        assert_eq!(records.len(), 4);
    }

    #[test]
    fn get_wraps_revoked_middle_period_returns_short_page() {
        let env = Env::default();
        let user = setup_user(&env, &[1, 2, 3, 4, 5]);

        // Revoking removes the Wrap record but leaves UserPeriods intact, so
        // the page is short (4 instead of 5).
        env.storage()
            .persistent()
            .remove(&DataKey::Wrap(user.clone(), 3));

        let records = get_wraps(&env, user, 0, u32::MAX);

        assert_eq!(records.len(), 4);
        assert_eq!(periods(&env, &records), vec![&env, 1u64, 2, 4, 5]);
    }
}

//! Strongly typed contract events.
//!
//! Enforces a consistent topic convention for all events:
//! `(version, domain, action, ..keys)`

use soroban_sdk::{contracttype, symbol_short, Address, BytesN, Env, Symbol};

use crate::storage_types::{StakeConfig, WrapState};

/// Strongly typed event names for mint operations.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MintEventType {
    Mint,
    Transition,
}

impl MintEventType {
    /// Convert this event type to a Soroban `Symbol`.
    pub fn to_symbol(self, e: &Env) -> Symbol {
        match self {
            MintEventType::Mint => Symbol::new(e, "mint"),
            MintEventType::Transition => Symbol::new(e, "trans"),
        }
    }
}

/// Strongly typed event data payloads for mint operations.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MintEventData {
    /// A wrap was successfully minted.
    Mint(Address, u64, Symbol),
    /// A wrap's lifecycle state was transitioned.
    Transition(Address, u64, WrapState),
}

/// All events emitted by the contract.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Event {
    // Admin
    AdminInit { admin: Address },
    AdminUpdated { old_admin: Address, new_admin: Address },
    AdminPause { paused: bool },
    AdminFeeUpdated { token: Address, recipient: Address, amount: i128 },
    AdminFeeCleared,
    AdminUpgrade { version: u32, wasm_hash: BytesN<32> },

    // Bridge
    BridgeOut { user: Address, destination_chain: u32, nonce: u64, recipient_address: BytesN<32>, period: u64 },
    BridgeRefund { user: Address, period: u64, nonce: u64 },
    BridgeInRej { recipient: Address, source_chain: u32, nonce: u64, period: u64 },
    BridgeIn { recipient: Address, source_chain: u32, nonce: u64, period: u64 },

    // Burn
    Burn { user: Address, period: u64 },

    // Governance
    GovPropose { id: u64, proposer: Address, proposed_admin: Address },
    GovVote { id: u64, voter: Address, support: bool },
    GovExecuted { id: u64, new_admin: Address },
    GovDefeated { id: u64 },
    GovCancelled { id: u64, caller: Address },

    // Merkle
    WhitelistRoot { root: BytesN<32> },
    WhitelistCleared,

    // Mint
    Mint { user: Address, period: u64, archetype: Symbol },
    MintTransition { user: Address, period: u64, state: WrapState },
    MintExpire { user: Address, period: u64 },

    // Revoke
    Revoke { user: Address, period: u64, reason_hash: BytesN<32> },

    // Stake
    StakeAdd { user: Address, amount: i128 },
    StakeInit { user: Address, amount: i128 },
    StakeUnstake { user: Address, amount: i128 },
    StakeWithdraw { user: Address, withdrawn: i128 },
    StakeConfig { config: StakeConfig },

    // Timelock
    TimelockEnabled { delay: u64 },
    TimelockSched { id: BytesN<32>, eta: u64 },
    TimelockCancel { id: BytesN<32> },
    TimelockUpgrade { wasm_hash: BytesN<32> },
    TimelockExec { id: BytesN<32> },

    // Transfer
    TransferBackfill { user: Address, count: u32 },
    Transfer { from: Address, to: Address, period: u64, fee: Option<(Address, Address, i128)> },
}

/// Strongly typed event publisher.
#[allow(deprecated)]
pub fn publish_event(e: &Env, event: Event) {
    let v1 = symbol_short!("v1");
    match event {
        Event::AdminInit { admin } => e.events().publish((v1, symbol_short!("admin"), symbol_short!("init")), admin),
        Event::AdminUpdated { old_admin, new_admin } => e.events().publish((v1, symbol_short!("admin"), symbol_short!("updated")), (old_admin, new_admin)),
        Event::AdminPause { paused } => e.events().publish((v1, symbol_short!("admin"), symbol_short!("pause")), paused),
        Event::AdminFeeUpdated { token, recipient, amount } => e.events().publish((v1, symbol_short!("admin"), symbol_short!("fee")), (token, recipient, amount)),
        Event::AdminFeeCleared => e.events().publish((v1, symbol_short!("admin"), symbol_short!("fee_clr")), ()),
        Event::AdminUpgrade { version, wasm_hash } => e.events().publish((v1, symbol_short!("admin"), symbol_short!("upgrade")), (version, wasm_hash)),
        
        Event::BridgeOut { user, destination_chain, nonce, recipient_address, period } => e.events().publish((v1, symbol_short!("bridge"), symbol_short!("out")), (user, destination_chain, nonce, recipient_address, period)),
        Event::BridgeRefund { user, period, nonce } => e.events().publish((v1, symbol_short!("bridge"), symbol_short!("refund")), (user, period, nonce)),
        Event::BridgeInRej { recipient, source_chain, nonce, period } => e.events().publish((v1, symbol_short!("bridge"), symbol_short!("in_rej")), (recipient, source_chain, nonce, period)),
        Event::BridgeIn { recipient, source_chain, nonce, period } => e.events().publish((v1, symbol_short!("bridge"), symbol_short!("in")), (recipient, source_chain, nonce, period)),

        Event::Burn { user, period } => e.events().publish((v1, symbol_short!("wrap"), symbol_short!("burn")), (user, period)),
        
        Event::GovPropose { id, proposer, proposed_admin } => e.events().publish((v1, symbol_short!("gov"), symbol_short!("propose")), (id, proposer, proposed_admin)),
        Event::GovVote { id, voter, support } => e.events().publish((v1, symbol_short!("gov"), symbol_short!("vote")), (id, voter, support)),
        Event::GovExecuted { id, new_admin } => e.events().publish((v1, symbol_short!("gov"), symbol_short!("executed")), (id, new_admin)),
        Event::GovDefeated { id } => e.events().publish((v1, symbol_short!("gov"), symbol_short!("defeated")), id),
        Event::GovCancelled { id, caller } => e.events().publish((v1, symbol_short!("gov"), symbol_short!("cancelled")), (id, caller)),

        Event::WhitelistRoot { root } => e.events().publish((v1, symbol_short!("whitelist"), symbol_short!("root")), root),
        Event::WhitelistCleared => e.events().publish((v1, symbol_short!("whitelist"), symbol_short!("cleared")), ()),

        Event::Mint { user, period, archetype } => e.events().publish((v1, symbol_short!("wrap"), symbol_short!("mint")), (user, period, archetype)),
        Event::MintTransition { user, period, state } => e.events().publish((v1, symbol_short!("wrap"), symbol_short!("trans")), (user, period, state)),
        Event::MintExpire { user, period } => e.events().publish((v1, symbol_short!("wrap"), symbol_short!("expire")), (user, period)),
        
        Event::Revoke { user, period, reason_hash } => e.events().publish((v1, symbol_short!("wrap"), symbol_short!("revoke")), (user, period, reason_hash)),

        Event::StakeAdd { user, amount } => e.events().publish((v1, symbol_short!("stake"), symbol_short!("add")), (user, amount)),
        Event::StakeInit { user, amount } => e.events().publish((v1, symbol_short!("stake"), symbol_short!("init")), (user, amount)),
        Event::StakeUnstake { user, amount } => e.events().publish((v1, symbol_short!("stake"), symbol_short!("unstake")), (user, amount)),
        Event::StakeWithdraw { user, withdrawn } => e.events().publish((v1, symbol_short!("stake"), symbol_short!("withdraw")), (user, withdrawn)),
        Event::StakeConfig { config } => e.events().publish((v1, symbol_short!("stake"), symbol_short!("cfg")), config),

        Event::TimelockEnabled { delay } => e.events().publish((v1, symbol_short!("timelock"), symbol_short!("enabled")), delay),
        Event::TimelockSched { id, eta } => e.events().publish((v1, symbol_short!("timelock"), symbol_short!("sched")), (id, eta)),
        Event::TimelockCancel { id } => e.events().publish((v1, symbol_short!("timelock"), symbol_short!("cancel")), id),
        Event::TimelockUpgrade { wasm_hash } => e.events().publish((v1, symbol_short!("timelock"), symbol_short!("upgrade")), wasm_hash),
        Event::TimelockExec { id } => e.events().publish((v1, symbol_short!("timelock"), symbol_short!("exec")), id),

        Event::TransferBackfill { user, count } => e.events().publish((v1, symbol_short!("transfer"), symbol_short!("backfill")), (user, count)),
        Event::Transfer { from, to, period, fee } => e.events().publish((v1, symbol_short!("transfer"), symbol_short!("transfer")), (from, to, period, fee)),
    }
}

//! Strongly typed contract events.
//!
//! Enforces a consistent topic convention for all events:
//! `(version, domain, action, ..keys)`
//!
//! - `version`: `v1` (Symbol)
//! - `domain`: e.g. `admin`, `bridge`, `wrap`, `gov`, `whitelist`, `stake`, `timelock`, `transfer` (Symbol, <= 9 chars)
//! - `action`: e.g. `init`, `updated`, `pause`, `fee` (Symbol, <= 9 chars)
//! - `keys`: optional extra keys if it fits in 4 topic limit. But here we just use `(version, domain, action)` or similar, and place data in the payload.
//!
//! Replace inline `e.events().publish()` calls with typed enum values, reducing the risk of typos and improving discoverability.

use soroban_sdk::{contracttype, symbol_short, Address, BytesN, Env, Symbol};

use crate::storage_types::{StakeConfig, WrapState};

/// All events emitted by the contract.
///
/// This enum is used for type-safe event publishing via [`publish_event`].
/// Each variant maps to a `(domain, action)` pair and carries the data fields
/// that are published as the event payload.
pub enum Event {
    // Admin
    AdminInit(Address),
    AdminUpdated(Address, Address),
    AdminPause(bool),
    AdminFeeUpdated(Address, Address, i128),
    AdminFeeCleared,
    AdminUpgrade(u32, BytesN<32>),

    // Bridge
    BridgeOut(Address, u32, u64, BytesN<32>, u64),
    BridgeRefund(Address, u64, u64),
    BridgeInRej(Address, u32, u64, u64),
    BridgeIn(Address, u32, u64, u64),

    // Burn
    Burn(Address, u64),

    // Governance
    GovPropose(u64, Address, Address),
    GovVote(u64, Address, bool),
    GovExecuted(u64, Address),
    GovDefeated(u64),
    GovCancelled(u64, Address),

    // Merkle
    WhitelistRoot(BytesN<32>),
    WhitelistCleared,

    // Mint
    Mint(Address, u64, Symbol),
    MintTransition(Address, u64, WrapState),
    MintExpire(Address, u64),

    // Revoke
    Revoke(Address, u64, BytesN<32>),

    // Stake
    StakeAdd(Address, i128),
    StakeInit(Address, i128),
    StakeUnstake(Address, i128),
    StakeWithdraw(Address, i128),
    StakeConfig(StakeConfig),

    // Timelock
    TimelockEnabled(u64),
    TimelockSched(BytesN<32>, u64),
    TimelockCancel(BytesN<32>),
    TimelockUpgrade(BytesN<32>),
    TimelockExec(BytesN<32>),

    // Transfer
    TransferBackfill(Address, u32),
    Transfer(Address, Address, u64, Option<(Address, Address, i128)>),
}

/// Mint event types for the legacy event format used by mint.rs and bridge.rs.
pub(crate) enum MintEventType {
    Mint,
    Transition,
    Expire,
}

impl MintEventType {
    pub fn to_symbol(&self, _e: &Env) -> Symbol {
        match self {
            MintEventType::Mint => symbol_short!("Mint"),
            MintEventType::Transition => symbol_short!("Transit"),
            MintEventType::Expire => symbol_short!("Expire"),
        }
    }
}

/// Data payload for legacy mint events.
#[contracttype]
#[derive(Clone, Debug)]
pub(crate) enum MintEventData {
    Mint(Address, u64, Symbol),
    Transition(Address, u64, WrapState),
    Expire(Address, u64),
}

/// Strongly typed event publisher.
///
/// Destructures each [`Event`] variant and publishes the data as a tuple,
/// avoiding the need for `#[contracttype]` on the `Event` enum (which would
/// fail for variants containing complex nested types like `Option<(…)>`).
#[allow(deprecated)]
pub fn publish_event(e: &Env, event: Event) {
    let v1 = symbol_short!("v1");
    match event {
        Event::AdminInit(admin) => {
            e.events().publish((v1, symbol_short!("admin"), symbol_short!("init")), admin);
        }
        Event::AdminUpdated(old, new) => {
            e.events().publish((v1, symbol_short!("admin"), symbol_short!("updated")), (old, new));
        }
        Event::AdminPause(paused) => {
            e.events().publish((v1, symbol_short!("admin"), symbol_short!("pause")), paused);
        }
        Event::AdminFeeUpdated(token, recipient, amount) => {
            e.events().publish((v1, symbol_short!("admin"), symbol_short!("fee")), (token, recipient, amount));
        }
        Event::AdminFeeCleared => {
            e.events().publish((v1, symbol_short!("admin"), symbol_short!("fee_clr")), ());
        }
        Event::AdminUpgrade(version, wasm_hash) => {
            e.events().publish((v1, symbol_short!("admin"), symbol_short!("upgrade")), (version, wasm_hash));
        }

        Event::BridgeOut(user, chain, nonce, addr, period) => {
            e.events().publish((v1, symbol_short!("bridge"), symbol_short!("out")), (user, chain, nonce, addr, period));
        }
        Event::BridgeRefund(user, period, nonce) => {
            e.events().publish((v1, symbol_short!("bridge"), symbol_short!("refund")), (user, period, nonce));
        }
        Event::BridgeInRej(recipient, chain, nonce, period) => {
            e.events().publish((v1, symbol_short!("bridge"), symbol_short!("in_rej")), (recipient, chain, nonce, period));
        }
        Event::BridgeIn(recipient, chain, nonce, period) => {
            e.events().publish((v1, symbol_short!("bridge"), symbol_short!("in")), (recipient, chain, nonce, period));
        }

        Event::Burn(user, period) => {
            e.events().publish((v1, symbol_short!("wrap"), symbol_short!("burn")), (user, period));
        }

        Event::GovPropose(id, proposer, proposed) => {
            e.events().publish((v1, symbol_short!("gov"), symbol_short!("propose")), (id, proposer, proposed));
        }
        Event::GovVote(id, voter, support) => {
            e.events().publish((v1, symbol_short!("gov"), symbol_short!("vote")), (id, voter, support));
        }
        Event::GovExecuted(id, new_admin) => {
            e.events().publish((v1, symbol_short!("gov"), symbol_short!("executed")), (id, new_admin));
        }
        Event::GovDefeated(id) => {
            e.events().publish((v1, symbol_short!("gov"), symbol_short!("defeated")), id);
        }
        Event::GovCancelled(id, caller) => {
            e.events().publish((v1, symbol_short!("gov"), symbol_short!("cancelled")), (id, caller));
        }

        Event::WhitelistRoot(root) => {
            e.events().publish((v1, symbol_short!("whitelist"), symbol_short!("root")), root);
        }
        Event::WhitelistCleared => {
            e.events().publish((v1, symbol_short!("whitelist"), symbol_short!("cleared")), ());
        }

        Event::Mint(user, period, archetype) => {
            e.events().publish((v1, symbol_short!("wrap"), symbol_short!("mint")), (user, period, archetype));
        }
        Event::MintTransition(user, period, state) => {
            e.events().publish((v1, symbol_short!("wrap"), symbol_short!("trans")), (user, period, state));
        }
        Event::MintExpire(user, period) => {
            e.events().publish((v1, symbol_short!("wrap"), symbol_short!("expire")), (user, period));
        }

        Event::Revoke(user, period, reason) => {
            e.events().publish((v1, symbol_short!("wrap"), symbol_short!("revoke")), (user, period, reason));
        }

        Event::StakeAdd(user, amount) => {
            e.events().publish((v1, symbol_short!("stake"), symbol_short!("add")), (user, amount));
        }
        Event::StakeInit(user, amount) => {
            e.events().publish((v1, symbol_short!("stake"), symbol_short!("init")), (user, amount));
        }
        Event::StakeUnstake(user, amount) => {
            e.events().publish((v1, symbol_short!("stake"), symbol_short!("unstake")), (user, amount));
        }
        Event::StakeWithdraw(user, withdrawn) => {
            e.events().publish((v1, symbol_short!("stake"), symbol_short!("withdraw")), (user, withdrawn));
        }
        Event::StakeConfig(config) => {
            e.events().publish((v1, symbol_short!("stake"), symbol_short!("cfg")), config);
        }

        Event::TimelockEnabled(delay) => {
            e.events().publish((v1, symbol_short!("timelock"), symbol_short!("enabled")), delay);
        }
        Event::TimelockSched(id, eta) => {
            e.events().publish((v1, symbol_short!("timelock"), symbol_short!("sched")), (id, eta));
        }
        Event::TimelockCancel(id) => {
            e.events().publish((v1, symbol_short!("timelock"), symbol_short!("cancel")), id);
        }
        Event::TimelockUpgrade(wasm_hash) => {
            e.events().publish((v1, symbol_short!("timelock"), symbol_short!("upgrade")), wasm_hash);
        }
        Event::TimelockExec(id) => {
            e.events().publish((v1, symbol_short!("timelock"), symbol_short!("exec")), id);
        }

        Event::TransferBackfill(user, count) => {
            e.events().publish((v1, symbol_short!("transfer"), symbol_short!("backfill")), (user, count));
        }
        Event::Transfer(from, to, period, fee) => {
            e.events().publish((v1, symbol_short!("transfer"), symbol_short!("transfer")), (from, to, period, fee));
        }
    }
}

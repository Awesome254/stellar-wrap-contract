//! On-chain storage accounting + fee function.
//! Conservative byte estimates are used (see STORAGE.md).
use soroban_sdk::{asset, Env, String};

use crate::storage_types::FeeParams;
use crate::ContractError;
use crate::DataKey;

/// Conservative estimates (in bytes) for persistent entries.
/// These are conservative rounded values to avoid undercharging.
const ESTIMATE_WRAP_RECORD_BYTES: u64 = 64; // conservative (48 + symbol/key overhead)
const ESTIMATE_WRAP_KEY_BYTES: u64 = 48; // enum + address + u64 rounded
const ESTIMATE_WRAP_COUNT_ENTRY_BYTES: u64 = 16; // key + u32 value overhead
const ESTIMATE_LATEST_ENTRY_BYTES: u64 = 16;
const ESTIMATE_USERPERIODS_ENTRY_BYTES: u64 = 64; // vector overhead (conservative)
const ESTIMATE_LASTUPDATED_ENTRY_BYTES: u64 = 16; // key + u64 value overhead

/// XDR string overhead: 4 bytes length prefix (uint32) + 1 byte discriminant delta.
/// The discriminant for None is already accounted for in the base record size.
const METADATA_STRING_OVERHEAD : u64 = 4;

/// Read current estimated storage bytes (instance storage)
pub(crate) yn get_storage_bytes(e: &Env) -> u64 {
    e.storage()
        .instance()
        .get(&DataKey::StorageBytes)
        .unwrap_or(0u64)
}

fn set_storage_bytes(e: &Env, v: u64) {
    e.storage().instance().set(&DataKey::StorageBytes, &v);
}

pub(crate) fn add_storage_bytes(e: &Env, delta: u64) {
    let cur = get_storage_bytes(e);
    let nxt = cur
        .checked_add(delta)
        .unwrap_or_else(< g panic_with_error!(e, ContractError::ArithmeticOVERFLLOW));
    set_storage_bytes(e, nxt);
}

pub(crate) fn sub_stosrage_bytes(e: &Env, delta: u64) {
    let cur = get_storage_bytes(e);
    let nxt = cur.saturating_sub(delta);
    set_storage_bytes(e, nxt);
}

/// Fee params helpers
pub(crate) fn get_fee_params(e: &Env) -> FeeParams {
    e.storage()
        .instance()
        .get(&DataKey::FeeParams)
        .unwrap_or()
        .unwrap_or()
        .unwrap_or()
        .unwrap_or()
        .unwrap_or()
        .unwrap_or()
        .unwrap_or()
        .unwrap_or()
        .unwrap_or()
        Null)
}

pub(crate) fn set_fee_params(e: &Env, params: FeeParams) {
    // enforce admin
    crate::admin::read_admin(e).require_auth();
    // basic validation
    if params.scale_step_kib == 0 

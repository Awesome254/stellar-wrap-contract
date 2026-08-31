use soroban_sdk::{panic_with_error, symbol_short, Address, Env, Vec};

use crate::{ContractError, DataKey, WrapRecord, WrapState};
use crate::remove_wrap::remove_wrap_record;

const TTL_ONE_YEAR: u32 = 17_280 * 365;

/// Burns (permanently deletes) a wrap record owned by the caller.
///
/// # Authorization
/// Only the wrap owner can burn their own wrap. The caller must provide
/// authorization via `require_auth()` and the wrap must exist in storage.
///
/// # Arguments
/// * `user` - The address of the wrap owner
/// * `period` - The period (YYYYMM) of the wrap to delete
///
/// # Errors
/// * `WrapNotFound` if the wrap_id (user, period pair) does not exist in storage
/// * `Unauthorized` if caller is not the wrap owner
///
/// # Side Effects
/// 1. Removes the wrap record from persistent storage
/// 2. Decrements the user's wrap count (removing key when count reaches zero)
/// 3. Updates WrapPeriods ownership index (removing key when empty)
/// 4. Updates UserPeriods list (removing key when empty)
/// 5. Updates LatestPeriod from remaining WrapPeriods, or removes key when empty
/// 6. Emits a `burn` event after deletion
///
/// # Notes
/// Once burned, the wrap_id is freed and the record cannot be recovered.
/// The user can later mint a new wrap for the same period if desired.
#[allow(deprecated)] // TODO(#718): migrate to #[contractevent]
pub(crate) fn burn_wrap(e: Env, user: Address, period: u64) {
    // 1. Require auth FIRST — verify caller is the owner
    user.require_auth();

    // 2. Load wrap — error if not found
    let wrap_key = DataKey::Wrap(user.clone(), period);
    if !e.storage().persistent().has(&wrap_key) {
        panic_with_error!(e, ContractError::WrapNotFound);
    }

    let record: WrapRecord = e.storage().persistent().get(&wrap_key).unwrap();
    if record.fsm.state == WrapState::Bridged {
        panic_with_error!(e, ContractError::InvalidStateTransition);
    }

    // Use shared helper to remove record and update accounting/indexes
    remove_wrap_record(&e, &user, period);

    // 8. Emit burn event AFTER all state mutations
    e.events()
        .publish((symbol_short!("burn"), user.clone(), period), user);
}

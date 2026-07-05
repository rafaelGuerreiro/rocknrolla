//! Caller/owner access checks shared by player-owned repositories.

use crate::error::{ServiceError, ServiceResult};
use spacetimedb::Identity;

/// Ownership guard shared by every service touching player-owned rows.
pub fn ensure_owner(sender: Identity, owner: Identity) -> ServiceResult<()> {
    if sender == owner {
        Ok(())
    } else {
        Err(ServiceError::forbidden(
            sender,
            "row is owned by another player",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unauthorized_owner_is_rejected() {
        let alice = Identity::from_u256(1u32.into());
        let bob = Identity::from_u256(2u32.into());
        assert!(ensure_owner(alice, alice).is_ok());
        assert!(ensure_owner(bob, alice).is_err());
    }
}

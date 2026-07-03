//! Pure progression decisions used by the reducers, kept free of
//! `ReducerContext` so they can be unit tested.

use spacetimedb::Identity;

/// Ownership guard shared by every reducer touching player-owned rows.
pub fn ensure_owner(sender: Identity, owner: Identity) -> Result<(), String> {
    if sender == owner {
        Ok(())
    } else {
        Err("row is owned by another player".to_string())
    }
}

/// An opened lootbox is consumed and can never be opened again.
pub fn ensure_unopened(opened: bool) -> Result<(), String> {
    if opened {
        Err("lootbox already opened".to_string())
    } else {
        Ok(())
    }
}

/// Completion is idempotent: only the first completion grants rewards.
pub fn grants_first_completion_rewards(already_completed: bool) -> bool {
    !already_completed
}

/// Configured successors to insert: targets not already enabled, deduplicated.
pub fn successor_inserts(configured: &[String], enabled: &[String]) -> Vec<String> {
    let mut inserts: Vec<String> = Vec::new();
    for successor in configured {
        if !enabled.contains(successor) && !inserts.contains(successor) {
            inserts.push(successor.clone());
        }
    }
    inserts
}

/// Pick one entry from `(id, weight)` pairs given `roll in 0..total_weight`.
/// Zero-weight entries can never be picked.
pub fn pick_weighted(weighted: &[(String, u64)], roll: u64) -> Option<&str> {
    let mut cursor = roll;
    for (id, weight) in weighted {
        if cursor < *weight {
            return Some(id);
        }
        cursor -= weight;
    }
    None
}

/// A character unlocks when the player owns at least one of every unique
/// piece assigned to it. Characters with no pieces never unlock via drops.
pub fn unlocks_character(required: &[String], owned: &[String]) -> bool {
    !required.is_empty() && required.iter().all(|piece| owned.contains(piece))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ids(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn completion_rewards_only_once() {
        assert!(grants_first_completion_rewards(false));
        assert!(!grants_first_completion_rewards(true));
    }

    #[test]
    fn successors_unlock_only_missing_targets() {
        let configured = ids(&["riverside-run", "tutorial-hill", "riverside-run"]);
        let enabled = ids(&["tutorial-hill"]);
        assert_eq!(
            successor_inserts(&configured, &enabled),
            ids(&["riverside-run"])
        );
        assert!(
            successor_inserts(&configured, &ids(&["tutorial-hill", "riverside-run"])).is_empty()
        );
    }

    #[test]
    fn unauthorized_owner_is_rejected() {
        let alice = Identity::from_u256(1u32.into());
        let bob = Identity::from_u256(2u32.into());
        assert!(ensure_owner(alice, alice).is_ok());
        assert!(ensure_owner(bob, alice).is_err());
    }

    #[test]
    fn opened_lootbox_is_consumed() {
        assert!(ensure_unopened(false).is_ok());
        assert!(ensure_unopened(true).is_err());
    }

    #[test]
    fn weighted_pick_covers_every_band_and_rejects_overflow() {
        let weighted = vec![
            ("stone-chip-a".to_string(), 3u64),
            ("paper-scrap-a".to_string(), 0u64),
            ("paper-scrap-b".to_string(), 2u64),
        ];
        assert_eq!(pick_weighted(&weighted, 0), Some("stone-chip-a"));
        assert_eq!(pick_weighted(&weighted, 2), Some("stone-chip-a"));
        assert_eq!(pick_weighted(&weighted, 3), Some("paper-scrap-b"));
        assert_eq!(pick_weighted(&weighted, 4), Some("paper-scrap-b"));
        assert_eq!(pick_weighted(&weighted, 5), None);
    }

    #[test]
    fn duplicate_pieces_count_toward_a_single_ownership() {
        let required = ids(&["paper-scrap-a", "paper-scrap-b", "paper-scrap-c"]);
        let owned_with_duplicates = ids(&["paper-scrap-a", "paper-scrap-b"]);
        assert!(!unlocks_character(&required, &owned_with_duplicates));
        let owned_all = ids(&[
            "paper-scrap-b",
            "paper-scrap-a",
            "stone-chip-a",
            "paper-scrap-c",
        ]);
        assert!(unlocks_character(&required, &owned_all));
    }

    #[test]
    fn characters_without_pieces_never_unlock_from_drops() {
        assert!(!unlocks_character(&[], &ids(&["anything"])));
    }
}

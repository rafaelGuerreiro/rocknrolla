//! Seed content (characters, pieces, lootboxes) parsing and validation.

use serde::Deserialize;
use std::collections::HashSet;

#[derive(Deserialize, Debug)]
pub struct SeedCharacter {
    pub id: String,
    pub name: String,
    pub style: String,
    pub rarity_weight: u32,
    pub density: f32,
    pub jump_speed: f32,
    pub flight_time_ms: u32,
    pub buoyancy: f32,
    pub fire_resistance: f32,
    #[serde(default)]
    pub starter: bool,
}

#[derive(Deserialize, Debug)]
pub struct SeedPiece {
    pub id: String,
    pub name: String,
    pub character_id: String,
}

#[derive(Deserialize, Debug)]
pub struct SeedDrop {
    pub piece_id: String,
    pub weight: u32,
}

#[derive(Deserialize, Debug)]
pub struct SeedLootbox {
    pub id: String,
    pub name: String,
    pub drops: Vec<SeedDrop>,
}

#[derive(Deserialize, Debug)]
pub struct SeedContent {
    pub characters: Vec<SeedCharacter>,
    pub pieces: Vec<SeedPiece>,
    pub lootboxes: Vec<SeedLootbox>,
}

/// Parse and cross-validate a committed seed content file.
pub fn parse_seed(source: &str) -> Result<SeedContent, String> {
    let content: SeedContent =
        serde_json::from_str(source).map_err(|e| format!("invalid JSON: {e}"))?;

    let mut character_ids = HashSet::new();
    for character in &content.characters {
        crate::uuid::validate_uuid(&character.id, "character id")?;
        if !character_ids.insert(character.id.as_str()) {
            return Err(format!("duplicate character id '{}'", character.id));
        }
        if character.rarity_weight == 0 {
            return Err(format!(
                "character '{}' has zero rarity_weight",
                character.id
            ));
        }
        if character.density <= 0.0 {
            return Err(format!(
                "character '{}' needs a positive density",
                character.id
            ));
        }
    }
    if !content.characters.iter().any(|c| c.starter) {
        return Err("at least one character must be a starter".to_string());
    }

    let mut piece_ids = HashSet::new();
    for piece in &content.pieces {
        crate::uuid::validate_uuid(&piece.id, "piece id")?;
        if !piece_ids.insert(piece.id.as_str()) {
            return Err(format!("duplicate piece id '{}'", piece.id));
        }
        if !character_ids.contains(piece.character_id.as_str()) {
            return Err(format!(
                "piece '{}' references unknown character '{}'",
                piece.id, piece.character_id
            ));
        }
    }

    let mut lootbox_ids = HashSet::new();
    for lootbox in &content.lootboxes {
        crate::uuid::validate_uuid(&lootbox.id, "lootbox id")?;
        if !lootbox_ids.insert(lootbox.id.as_str()) {
            return Err(format!("duplicate lootbox id '{}'", lootbox.id));
        }
        if lootbox.drops.is_empty() {
            return Err(format!("lootbox '{}' has no drops", lootbox.id));
        }
        for drop in &lootbox.drops {
            if drop.weight == 0 {
                return Err(format!(
                    "lootbox '{}' drop '{}' has zero weight",
                    lootbox.id, drop.piece_id
                ));
            }
            if !piece_ids.contains(drop.piece_id.as_str()) {
                return Err(format!(
                    "lootbox '{}' references unknown piece '{}'",
                    lootbox.id, drop.piece_id
                ));
            }
        }
    }
    Ok(content)
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID: &str = r##"{
        "characters": [{
            "id": "0195c8f1-0000-7000-8000-0000000000c1", "name": "Stone", "style": "#8a8f98",
            "rarity_weight": 100, "density": 0.004, "jump_speed": 11,
            "flight_time_ms": 550, "buoyancy": 0.3, "fire_resistance": 0.9,
            "starter": true
        }],
        "pieces": [{
            "id": "0195c8f1-0000-7000-8000-0000000000e1", "name": "Stone Chip A",
            "character_id": "0195c8f1-0000-7000-8000-0000000000c1"
        }],
        "lootboxes": [{
            "id": "0195c8f1-0000-7000-8000-0000000000b1", "name": "Trail Cache",
            "drops": [{"piece_id": "0195c8f1-0000-7000-8000-0000000000e1", "weight": 10}]
        }]
    }"##;

    #[test]
    fn parses_valid_seed() {
        let content = parse_seed(VALID).unwrap();
        assert_eq!(content.characters.len(), 1);
        assert_eq!(content.lootboxes[0].drops[0].weight, 10);
    }

    #[test]
    fn rejects_non_uuid_ids() {
        let broken = VALID.replace("0195c8f1-0000-7000-8000-0000000000c1", "stone");
        assert!(
            parse_seed(&broken)
                .unwrap_err()
                .contains("not a valid UUID")
        );
    }

    #[test]
    fn rejects_unknown_references() {
        let broken = VALID.replace(
            "\"character_id\": \"0195c8f1-0000-7000-8000-0000000000c1\"",
            "\"character_id\": \"0195c8f1-0000-7000-8000-0000000000ff\"",
        );
        assert!(
            parse_seed(&broken)
                .unwrap_err()
                .contains("unknown character")
        );
        let broken = VALID.replace(
            "\"piece_id\": \"0195c8f1-0000-7000-8000-0000000000e1\"",
            "\"piece_id\": \"0195c8f1-0000-7000-8000-0000000000ff\"",
        );
        assert!(parse_seed(&broken).unwrap_err().contains("unknown piece"));
    }

    #[test]
    fn rejects_zero_weight_and_missing_starter() {
        let broken = VALID.replace("\"weight\": 10", "\"weight\": 0");
        assert!(parse_seed(&broken).unwrap_err().contains("zero weight"));
        let broken = VALID.replace("\"starter\": true", "\"starter\": false");
        assert!(parse_seed(&broken).unwrap_err().contains("starter"));
    }
}

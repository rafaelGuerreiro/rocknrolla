//! Minimal UUID string validation for authored content.
//!
//! Authored entity IDs are stable UUIDs committed in seed and Tiled files.
//! This validates the canonical `8-4-4-4-12` hex form without pulling in a
//! UUID dependency; parsing into a typed value happens server-side.

use anyhow::{Context, Result, bail};

/// Validate the canonical lowercase-or-uppercase hex UUID form.
pub fn validate_uuid(value: &str, what: &str) -> Result<()> {
    let bytes = value.as_bytes();
    let well_formed = bytes.len() == 36
        && bytes.iter().enumerate().all(|(i, &b)| match i {
            8 | 13 | 18 | 23 => b == b'-',
            _ => b.is_ascii_hexdigit(),
        });
    if well_formed {
        Ok(())
    } else {
        bail!("{what} '{value}' is not a valid UUID");
    }
}

/// Render a canonical UUID string as the reducer-call JSON SpacetimeDB expects:
/// `spacetimedb::Uuid` is a single `__uuid__: u128` field, which SATS serializes
/// as an unnamed 1-element tuple, i.e. a JSON array holding the u128 value.
pub fn uuid_arg(value: &str) -> Result<String> {
    Ok(format!("[{}]", to_u128(value)?))
}

/// Same as [`uuid_arg`], but for an `Option<Uuid>` argument (`null` when absent,
/// `{"some": [u128]}` when present — SATS tags `Some` explicitly in JSON).
pub fn uuid_opt_arg(value: Option<&str>) -> Result<String> {
    Ok(match value {
        Some(v) => format!(r#"{{"some":{}}}"#, uuid_arg(v)?),
        None => "null".to_string(),
    })
}

/// Same as [`uuid_arg`], but for a `Vec<Uuid>` argument.
pub fn uuid_vec_arg<S: AsRef<str>>(values: &[S]) -> Result<String> {
    let items = values.iter().map(|v| uuid_arg(v.as_ref())).collect::<Result<Vec<String>>>()?;
    Ok(format!("[{}]", items.join(",")))
}

fn to_u128(value: &str) -> Result<u128> {
    let hex: String = value.chars().filter(|c| *c != '-').collect();
    u128::from_str_radix(&hex, 16).with_context(|| format!("'{value}' is not a valid UUID"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_uuid_args() {
        assert_eq!(
            uuid_arg("fce40ba8-cab8-42d1-acd5-be8aed7b53e0").unwrap(),
            "[336149535101120413114579957789750940640]"
        );
        assert_eq!(uuid_opt_arg(None).unwrap(), "null");
        assert_eq!(
            uuid_opt_arg(Some("fce40ba8-cab8-42d1-acd5-be8aed7b53e0")).unwrap(),
            r#"{"some":[336149535101120413114579957789750940640]}"#
        );
        assert_eq!(uuid_vec_arg::<&str>(&[]).unwrap(), "[]");
        assert_eq!(
            uuid_vec_arg(&["fce40ba8-cab8-42d1-acd5-be8aed7b53e0", "fce40ba8-cab8-42d1-acd5-be8aed7b53e0"]).unwrap(),
            "[[336149535101120413114579957789750940640],[336149535101120413114579957789750940640]]"
        );
    }

    #[test]
    fn rejects_non_hex_uuid_arg() {
        assert!(uuid_arg("not-a-uuid-value!!!!").is_err());
    }

    #[test]
    fn accepts_canonical_uuids() {
        validate_uuid("0195c8f1-89ab-7def-8123-456789abcdef", "id").unwrap();
        validate_uuid("0195C8F1-89AB-7DEF-8123-456789ABCDEF", "id").unwrap();
    }

    #[test]
    fn rejects_malformed_uuids() {
        for bad in [
            "",
            "tutorial-hill",
            "0195c8f189ab7def8123456789abcdef",
            "0195c8f1-89ab-7def-8123-456789abcde",
            "0195c8f1-89ab-7def-8123-456789abcdeg",
        ] {
            assert!(validate_uuid(bad, "id").is_err(), "accepted '{bad}'");
        }
    }
}

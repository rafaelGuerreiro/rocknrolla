//! Input validation at the server trust boundary, shared by every reducer.

use crate::error::{ServiceError, ServiceResult};

/// Require a non-empty string no longer than `max_length` bytes.
pub fn validate_required_str(value: &str, name: &str, max_length: usize) -> ServiceResult<()> {
    if value.is_empty() {
        return Err(ServiceError::validation(format!("'{name}' is required")));
    }
    if value.len() > max_length {
        return Err(ServiceError::validation(format!(
            "'{name}' must be at most {max_length} bytes"
        )));
    }
    Ok(())
}

/// Require a strictly positive integer (weights, rarity).
pub fn validate_positive_u32(value: u32, name: &str) -> ServiceResult<()> {
    if value == 0 {
        return Err(ServiceError::validation(format!("'{name}' must be positive")));
    }
    Ok(())
}

/// Require a finite float within an inclusive range.
pub fn validate_f32_range(value: f32, name: &str, min: f32, max: f32) -> ServiceResult<()> {
    if !value.is_finite() {
        return Err(ServiceError::validation(format!("'{name}' must be finite")));
    }
    if value < min || value > max {
        return Err(ServiceError::validation(format!("'{name}' must be between {min} and {max}")));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn required_strings_reject_empty_and_oversized_values() {
        assert!(validate_required_str("ok", "name", 8).is_ok());
        assert!(validate_required_str("", "name", 8).is_err());
        assert!(validate_required_str("way too long", "name", 8).is_err());
    }

    #[test]
    fn positive_integers_reject_zero() {
        assert!(validate_positive_u32(1, "weight").is_ok());
        assert!(validate_positive_u32(0, "weight").is_err());
    }

    #[test]
    fn float_ranges_reject_nan_infinity_and_out_of_range() {
        assert!(validate_f32_range(0.5, "buoyancy", 0.0, 10.0).is_ok());
        assert!(validate_f32_range(f32::NAN, "buoyancy", 0.0, 10.0).is_err());
        assert!(validate_f32_range(f32::INFINITY, "buoyancy", 0.0, 10.0).is_err());
        assert!(validate_f32_range(-0.1, "buoyancy", 0.0, 10.0).is_err());
        assert!(validate_f32_range(10.1, "buoyancy", 0.0, 10.0).is_err());
    }
}

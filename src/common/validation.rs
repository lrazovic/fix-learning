//! Validation traits and error types for FIX messages
//!
//! This module provides the validation framework used throughout the FIX library
//! to ensure message integrity and compliance with the FIX 4.2 specification.

use std::fmt::Display;

/// Validation error types for FIX messages
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationError {
	/// A required field is missing from the message
	MissingRequiredField(String),
	/// A field has an invalid value
	InvalidFieldValue(String, String),
	/// The message checksum is invalid
	InvalidChecksum,
	/// The message body length is invalid
	InvalidBodyLength,
	/// The message is empty or malformed
	EmptyMessage,
	/// A field value is out of acceptable range
	ValueOutOfRange(String, String),
	/// A field format is incorrect
	InvalidFormat(String, String),
}

impl Display for ValidationError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			ValidationError::MissingRequiredField(field) => {
				write!(f, "Missing required field: {}", field)
			},
			ValidationError::InvalidFieldValue(field, value) => {
				write!(f, "Invalid value '{}' for field '{}'", value, field)
			},
			ValidationError::InvalidChecksum => {
				write!(f, "Invalid checksum")
			},
			ValidationError::InvalidBodyLength => {
				write!(f, "Invalid body length")
			},
			ValidationError::EmptyMessage => {
				write!(f, "Empty message")
			},
			ValidationError::ValueOutOfRange(field, value) => {
				write!(f, "Value '{}' for field '{}' is out of acceptable range", value, field)
			},
			ValidationError::InvalidFormat(field, value) => {
				write!(f, "Invalid format '{}' for field '{}'", value, field)
			},
		}
	}
}

impl std::error::Error for ValidationError {}

/// Trait for message validation
///
/// All FIX message components (header, body, trailer) implement this trait
/// to provide comprehensive validation of message structure and content.
pub trait Validate {
	/// Validate the component and return any validation errors
	///
	/// # Returns
	/// - `Ok(())` if the component is valid
	/// - `Err(ValidationError)` if validation fails
	fn validate(&self) -> Result<(), ValidationError>;

	/// Check if the component is valid (convenience method)
	///
	/// # Returns
	/// - `true` if validation passes
	/// - `false` if validation fails
	fn is_valid(&self) -> bool {
		self.validate().is_ok()
	}
}

/// Validation utilities for common validation patterns
pub mod utils {
	use super::ValidationError;

	/// Validate that a string field is not empty
	pub fn validate_non_empty_string(field_name: &str, value: &str) -> Result<(), ValidationError> {
		if value.is_empty() { Err(ValidationError::MissingRequiredField(field_name.to_string())) } else { Ok(()) }
	}

	/// Validate that a numeric value is within range
	pub fn validate_range<T: PartialOrd + std::fmt::Display>(
		field_name: &str,
		value: T,
		min: T,
		max: T,
	) -> Result<(), ValidationError> {
		if value < min || value > max {
			Err(ValidationError::ValueOutOfRange(field_name.to_string(), value.to_string()))
		} else {
			Ok(())
		}
	}

	/// Validate that a numeric value is positive (greater than zero)
	pub fn validate_positive<T: PartialOrd + std::fmt::Display + Default>(
		field_name: &str,
		value: T,
	) -> Result<(), ValidationError> {
		if value <= T::default() {
			Err(ValidationError::InvalidFieldValue(field_name.to_string(), value.to_string()))
		} else {
			Ok(())
		}
	}

	/// Validate checksum format (3 digits)
	pub fn validate_checksum_format(checksum: &str) -> Result<(), ValidationError> {
		if checksum.len() != 3 {
			return Err(ValidationError::InvalidChecksum);
		}

		if !checksum.chars().all(|c| c.is_ascii_digit()) {
			return Err(ValidationError::InvalidChecksum);
		}

		Ok(())
	}

	/// Validate that a sequence number is valid (greater than 0)
	pub fn validate_sequence_number(seq_num: u32) -> Result<(), ValidationError> {
		if seq_num == 0 {
			Err(ValidationError::InvalidFieldValue("MsgSeqNum".to_string(), "0".to_string()))
		} else {
			Ok(())
		}
	}
}

#[cfg(test)]
mod tests {
	use super::{utils::*, *};

	#[test]
	fn test_validation_error_display() {
		let error = ValidationError::MissingRequiredField("TestField".to_string());
		assert_eq!(error.to_string(), "Missing required field: TestField");

		let error = ValidationError::InvalidFieldValue("TestField".to_string(), "InvalidValue".to_string());
		assert_eq!(error.to_string(), "Invalid value 'InvalidValue' for field 'TestField'");
	}

	#[test]
	fn test_validate_non_empty_string() {
		assert!(validate_non_empty_string("TestField", "valid").is_ok());
		assert!(validate_non_empty_string("TestField", "").is_err());
	}

	#[test]
	fn test_validate_range() {
		assert!(validate_range("TestField", 5, 1, 10).is_ok());
		assert!(validate_range("TestField", 0, 1, 10).is_err());
		assert!(validate_range("TestField", 11, 1, 10).is_err());
	}

	#[test]
	fn test_validate_positive() {
		assert!(validate_positive("TestField", 1u32).is_ok());
		assert!(validate_positive("TestField", 0u32).is_err());
	}

	#[test]
	fn test_validate_checksum_format() {
		assert!(validate_checksum_format("123").is_ok());
		assert!(validate_checksum_format("000").is_ok());
		assert!(validate_checksum_format("12").is_err());
		assert!(validate_checksum_format("1234").is_err());
		assert!(validate_checksum_format("12a").is_err());
	}

	#[test]
	fn test_validate_sequence_number() {
		assert!(validate_sequence_number(1).is_ok());
		assert!(validate_sequence_number(999999).is_ok());
		assert!(validate_sequence_number(0).is_err());
	}

	#[test]
	fn test_is_valid_convenience_method() {
		struct TestValidator {
			should_pass: bool,
		}

		impl Validate for TestValidator {
			fn validate(&self) -> Result<(), ValidationError> {
				if self.should_pass { Ok(()) } else { Err(ValidationError::EmptyMessage) }
			}
		}

		let valid = TestValidator { should_pass: true };
		let invalid = TestValidator { should_pass: false };

		assert!(valid.is_valid());
		assert!(!invalid.is_valid());
	}
}

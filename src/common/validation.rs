//! Validation traits and error types for FIX messages
//!
//! This module provides the validation framework used throughout the FIX library
//! to ensure message integrity and compliance with the FIX 4.2 specification.

use std::fmt::Display;

/// Validation error types for FIX messages
#[derive(Debug, Clone, Eq, PartialEq)]
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
	/// The FIX version is not supported
	VersionMismatch,
	/// A field value is out of acceptable range
	ValueOutOfRange(String, String),
	/// A field format is incorrect
	InvalidFormat(String, String),
}

impl Display for ValidationError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::MissingRequiredField(field) => {
				write!(f, "Missing required field: {}", field)
			},
			Self::InvalidFieldValue(field, value) => {
				write!(f, "Invalid value '{}' for field '{}'", value, field)
			},
			Self::InvalidChecksum => {
				write!(f, "Invalid checksum")
			},
			Self::InvalidBodyLength => {
				write!(f, "Invalid body length")
			},
			Self::EmptyMessage => {
				write!(f, "Empty message")
			},
			Self::VersionMismatch => {
				write!(f, "FIX Version Not Supported")
			},
			Self::ValueOutOfRange(field, value) => {
				write!(f, "Value '{}' for field '{}' is out of acceptable range", value, field)
			},
			Self::InvalidFormat(field, value) => {
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

pub trait WriteTo {
	fn write_to(&self, buffer: &mut String);
}

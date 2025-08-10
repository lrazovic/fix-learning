//! FIX message trailer structures
//!
//! This module contains the standard FIX message trailer structure
//! that is common to all FIX messages, along with its validation logic.

use crate::common::validation::{Validate, ValidationError, utils};

/// Standard FIX message trailer
#[derive(Debug, Clone, PartialEq, Default)]
pub struct FixTrailer {
	// Required Trailer Fields
	pub checksum: String, // Tag 10 - Checksum of the message, always unencrypted, always last field in message.

	// Optional Trailer Fields
	pub signature_length: Option<u32>, // Tag 93 - Required when trailer contains signature. Note: Not to be included within SecureData field
	pub signature: Option<String>, // Tag 89 - Signature of the message. Note: Not to be included within SecureData field
}

impl Validate for FixTrailer {
	fn validate(&self) -> Result<(), ValidationError> {
		utils::validate_checksum_format(&self.checksum)?;
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_trailer_creation() {
		let trailer = FixTrailer::default();
		assert_eq!(trailer.checksum, "");
		assert_eq!(trailer.signature_length, None);
		assert_eq!(trailer.signature, None);
	}

	#[test]
	fn test_trailer_validation() {
		// Valid trailer
		let mut valid_trailer = FixTrailer::default();
		valid_trailer.checksum = "123".to_string();
		assert!(valid_trailer.is_valid());

		// Invalid trailer - wrong checksum format
		let mut invalid_trailer = FixTrailer::default();
		invalid_trailer.checksum = "12".to_string(); // Too short
		assert!(!invalid_trailer.is_valid());

		invalid_trailer.checksum = "1234".to_string(); // Too long
		assert!(!invalid_trailer.is_valid());

		invalid_trailer.checksum = "12a".to_string(); // Non-numeric
		assert!(!invalid_trailer.is_valid());
	}
}

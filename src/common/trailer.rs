//! FIX message trailer structures
//!
//! This module contains the standard FIX message trailer structure
//! that is common to all FIX messages, along with its validation logic.

use crate::common::validation::{Validate, ValidationError};

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
}

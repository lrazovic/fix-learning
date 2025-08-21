//! FIX message trailer structures
//!
//! This module contains the standard FIX message trailer structure
//! that is common to all FIX messages, along with its validation logic.

use crate::{
	SOH,
	common::validation::{Validate, ValidationError, WriteTo},
};
use std::fmt::Write;

/// Standard FIX message trailer
#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct FixTrailer {
	// Required Trailer Fields
	// TODO: This always has len == 3, so we can probably avoid using a String.
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

impl FixTrailer {
	/// Write only the non-checksum fields for body length calculation
	/// This includes optional fields like SignatureLength and Signature
	pub fn write_body_fields(&self, buffer: &mut String) {
		if let Some(sig_len) = self.signature_length {
			write!(buffer, "93={}{}", sig_len, SOH).unwrap();
		}
		if let Some(ref signature) = self.signature {
			write!(buffer, "89={}{}", signature, SOH).unwrap();
		}
	}

	/// Parse a field from tag-value pair into the trailer
	pub fn parse_field(&mut self, tag: u32, value: &str) -> Result<(), String> {
		match tag {
			10 => {
				self.checksum = value.to_string();
			},
			93 => {
				self.signature_length = Some(value.parse().map_err(|_| "Invalid SignatureLength")?);
			},
			89 => {
				self.signature = Some(value.to_string());
			},
			_ => return Err(format!("Unknown trailer field: {}", tag)),
		}
		Ok(())
	}
}

impl WriteTo for FixTrailer {
	fn write_to(&self, buffer: &mut String) {
		// Optional trailer fields
		self.write_body_fields(buffer);
		// Checksum is always last
		write!(buffer, "10={}{}", self.checksum, SOH).unwrap();
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

//! Logon message implementation (MsgType=A)
//!
//! This module implements the FIX 4.2 Logon message, which is used to
//! initiate a FIX session between two counterparties. The Logon message
//! establishes session parameters and authentication.

use crate::common::{EncryptMethod, Validate, ValidationError};

/// Logon message body (Tag 35=A)
///
/// The Logon message is the first message sent to initiate a FIX session.
/// It contains session parameters including encryption method and heartbeat interval.
#[derive(Debug, Clone, PartialEq)]
pub struct LogonBody {
	/// Encryption method (Tag 98) - Required
	pub encrypt_method: EncryptMethod,
	/// Heartbeat interval in seconds (Tag 108) - Required
	pub heart_bt_int: u32,
	/// Reset sequence number flag (Tag 141) - Optional
	pub reset_seq_num_flag: Option<bool>,
	/// Next expected message sequence number (Tag 789) - Optional
	pub next_expected_msg_seq_num: Option<u32>,
	/// Maximum message size (Tag 383) - Optional
	pub max_message_size: Option<u32>,
}

impl Default for LogonBody {
	fn default() -> Self {
		Self {
			encrypt_method: EncryptMethod::None,
			heart_bt_int: 30, // Default 30 seconds
			reset_seq_num_flag: None,
			next_expected_msg_seq_num: None,
			max_message_size: None,
		}
	}
}

impl Validate for LogonBody {
	fn validate(&self) -> Result<(), ValidationError> {
		if self.heart_bt_int == 0 {
			return Err(ValidationError::InvalidFieldValue("HeartBtInt".to_string(), "0".to_string()));
		}
		Ok(())
	}
}

impl LogonBody {
	/// Create a new logon body with required fields
	pub fn new(encrypt_method: EncryptMethod, heart_bt_int: u32) -> Self {
		Self { encrypt_method, heart_bt_int, ..Default::default() }
	}

	/// Set the reset sequence number flag
	pub fn with_reset_seq_num_flag(mut self, flag: bool) -> Self {
		self.reset_seq_num_flag = Some(flag);
		self
	}

	/// Set the next expected message sequence number
	pub fn with_next_expected_msg_seq_num(mut self, seq_num: u32) -> Self {
		self.next_expected_msg_seq_num = Some(seq_num);
		self
	}

	/// Set the maximum message size
	pub fn with_max_message_size(mut self, size: u32) -> Self {
		self.max_message_size = Some(size);
		self
	}

	/// Serialize logon-specific fields to FIX format
	pub(crate) fn serialize_fields(&self) -> String {
		let mut result = String::new();

		result.push_str(&format!("98={}\x01", self.encrypt_method));
		result.push_str(&format!("108={}\x01", self.heart_bt_int));

		if let Some(flag) = self.reset_seq_num_flag {
			result.push_str(&format!("141={}\x01", if flag { "Y" } else { "N" }));
		}
		if let Some(seq_num) = self.next_expected_msg_seq_num {
			result.push_str(&format!("789={}\x01", seq_num));
		}
		if let Some(max_size) = self.max_message_size {
			result.push_str(&format!("383={}\x01", max_size));
		}

		result
	}

	/// Parse a logon-specific field
	pub(crate) fn parse_field(&mut self, tag: u32, value: &str) -> Result<(), String> {
		match tag {
			98 => {
				self.encrypt_method = value.parse().map_err(|_| "Invalid EncryptMethod")?;
			},
			108 => {
				self.heart_bt_int = value.parse().map_err(|_| "Invalid HeartBtInt")?;
			},
			141 => {
				self.reset_seq_num_flag = Some(value == "Y");
			},
			789 => {
				self.next_expected_msg_seq_num = Some(value.parse().map_err(|_| "Invalid NextExpectedMsgSeqNum")?);
			},
			383 => {
				self.max_message_size = Some(value.parse().map_err(|_| "Invalid MaxMessageSize")?);
			},
			_ => return Err(format!("Unknown logon field: {}", tag)),
		}
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_logon_creation() {
		let logon = LogonBody::new(EncryptMethod::None, 30);
		assert!(logon.validate().is_ok());
		assert_eq!(logon.encrypt_method, EncryptMethod::None);
		assert_eq!(logon.heart_bt_int, 30);
		assert_eq!(logon.reset_seq_num_flag, None);
		assert_eq!(logon.next_expected_msg_seq_num, None);
		assert_eq!(logon.max_message_size, None);
	}

	#[test]
	fn test_logon_default() {
		let logon = LogonBody::default();
		assert!(logon.validate().is_ok());
		assert_eq!(logon.encrypt_method, EncryptMethod::None);
		assert_eq!(logon.heart_bt_int, 30);
	}

	#[test]
	fn test_logon_with_optional_fields() {
		let logon = LogonBody::new(EncryptMethod::Des, 60)
			.with_reset_seq_num_flag(true)
			.with_next_expected_msg_seq_num(1)
			.with_max_message_size(8192);

		assert!(logon.validate().is_ok());
		assert_eq!(logon.encrypt_method, EncryptMethod::Des);
		assert_eq!(logon.heart_bt_int, 60);
		assert_eq!(logon.reset_seq_num_flag, Some(true));
		assert_eq!(logon.next_expected_msg_seq_num, Some(1));
		assert_eq!(logon.max_message_size, Some(8192));
	}

	#[test]
	fn test_logon_validation() {
		// Valid logon
		let valid_logon = LogonBody::new(EncryptMethod::None, 30);
		assert!(valid_logon.is_valid());

		// Invalid logon - zero heartbeat interval
		let invalid_logon = LogonBody::new(EncryptMethod::None, 0);
		assert!(!invalid_logon.is_valid());
	}

	#[test]
	fn test_logon_serialization() {
		// Basic logon
		let logon = LogonBody::new(EncryptMethod::None, 30);
		let serialized = logon.serialize_fields();
		assert!(serialized.contains("98=0\x01"));
		assert!(serialized.contains("108=30\x01"));

		// Logon with optional fields
		let logon = LogonBody::new(EncryptMethod::Des, 60)
			.with_reset_seq_num_flag(true)
			.with_next_expected_msg_seq_num(1)
			.with_max_message_size(4096);

		let serialized = logon.serialize_fields();
		assert!(serialized.contains("98=2\x01"));
		assert!(serialized.contains("108=60\x01"));
		assert!(serialized.contains("141=Y\x01"));
		assert!(serialized.contains("789=1\x01"));
		assert!(serialized.contains("383=4096\x01"));
	}

	#[test]
	fn test_logon_field_parsing() {
		let mut logon = LogonBody::default();

		// Parse encryption method
		assert!(logon.parse_field(98, "2").is_ok());
		assert_eq!(logon.encrypt_method, EncryptMethod::Des);

		// Parse heartbeat interval
		assert!(logon.parse_field(108, "60").is_ok());
		assert_eq!(logon.heart_bt_int, 60);

		// Parse reset sequence number flag
		assert!(logon.parse_field(141, "Y").is_ok());
		assert_eq!(logon.reset_seq_num_flag, Some(true));

		assert!(logon.parse_field(141, "N").is_ok());
		assert_eq!(logon.reset_seq_num_flag, Some(false));

		// Parse next expected sequence number
		assert!(logon.parse_field(789, "123").is_ok());
		assert_eq!(logon.next_expected_msg_seq_num, Some(123));

		// Parse max message size
		assert!(logon.parse_field(383, "8192").is_ok());
		assert_eq!(logon.max_message_size, Some(8192));

		// Parse unknown field
		assert!(logon.parse_field(999, "unknown").is_err());

		// Parse invalid values
		assert!(logon.parse_field(98, "invalid").is_err());
		assert!(logon.parse_field(108, "invalid").is_err());
		assert!(logon.parse_field(789, "invalid").is_err());
		assert!(logon.parse_field(383, "invalid").is_err());
	}

	#[test]
	fn test_all_encryption_methods() {
		let encryption_methods = vec![
			EncryptMethod::None,
			EncryptMethod::Pkcs,
			EncryptMethod::Des,
			EncryptMethod::PkcsAndDes,
			EncryptMethod::PgpAndDes,
			EncryptMethod::PgpAndMd5,
			EncryptMethod::PemAndMd5,
		];

		for method in encryption_methods {
			let logon = LogonBody::new(method.clone(), 30);
			assert!(logon.is_valid());
			assert_eq!(logon.encrypt_method, method);
		}
	}

	#[test]
	fn test_logon_equality() {
		let logon1 = LogonBody::new(EncryptMethod::None, 30);
		let logon2 = LogonBody::default();
		assert_eq!(logon1, logon2);

		let logon3 = LogonBody::new(EncryptMethod::Des, 60);
		assert_ne!(logon1, logon3);

		let logon4 = LogonBody::new(EncryptMethod::None, 30).with_reset_seq_num_flag(true);
		assert_ne!(logon1, logon4);
	}

	#[test]
	fn test_logon_cloning() {
		let original =
			LogonBody::new(EncryptMethod::PgpAndMd5, 120).with_reset_seq_num_flag(true).with_max_message_size(2048);

		let cloned = original.clone();

		assert_eq!(original, cloned);
		assert_eq!(original.encrypt_method, cloned.encrypt_method);
		assert_eq!(original.heart_bt_int, cloned.heart_bt_int);
		assert_eq!(original.reset_seq_num_flag, cloned.reset_seq_num_flag);
		assert_eq!(original.max_message_size, cloned.max_message_size);
	}
}

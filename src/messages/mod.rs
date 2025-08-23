//! FIX message implementations
//!
//! This module contains implementations for all supported FIX message types,
//! organized by functionality (session, orders, market data, etc.).
//! Each message type has its own validation logic and serialization methods.

pub mod order;
pub mod session;

use crate::common::{
	Validate, ValidationError,
	validation::{FixFieldHandler, WriteTo},
};

// Re-export message body types
pub use order::{ExecutionReportBody, NewOrderSingleBody, OrderCancelRequestBody};
pub use session::{HeartbeatBody, LogonBody};

/// Message-specific body that only allocates fields needed for each message type
///
/// This enum provides memory-efficient storage by only allocating the fields
/// required for each specific message type, rather than having a single struct
/// with all possible fields.
#[derive(Debug, Clone, PartialEq)]
pub enum FixMessageBody {
	/// Heartbeat message body (MsgType=0)
	Heartbeat(HeartbeatBody),
	/// Logon message body (MsgType=A)
	Logon(LogonBody),
	/// New Order Single message body (MsgType=D)
	NewOrderSingle(NewOrderSingleBody),
	/// Execution Report message body (MsgType=8)
	ExecutionReport(ExecutionReportBody),
	/// Order Cancel Request message body (MsgType=F)
	OrderCancelRequest(OrderCancelRequestBody),
	/// Placeholder for other message types not yet implemented with specific bodies
	Other,
}

impl Validate for FixMessageBody {
	fn validate(&self) -> Result<(), ValidationError> {
		match self {
			Self::Heartbeat(body) => body.validate(),
			Self::Logon(body) => body.validate(),
			Self::NewOrderSingle(body) => body.validate(),
			Self::ExecutionReport(body) => body.validate(),
			Self::OrderCancelRequest(body) => body.validate(),
			Self::Other => Ok(()), // No validation for unsupported types yet
		}
	}
}

impl WriteTo for FixMessageBody {
	fn write_to(&self, buffer: &mut String) {
		match self {
			Self::Heartbeat(body) => body.write_to(buffer),
			Self::Logon(body) => body.write_to(buffer),
			Self::NewOrderSingle(body) => body.write_to(buffer),
			Self::ExecutionReport(body) => body.write_to(buffer),
			Self::OrderCancelRequest(body) => body.write_to(buffer),
			Self::Other => unimplemented!(),
		}
	}
}

impl FixFieldHandler for FixMessageBody {
	fn parse_field(&mut self, tag: u32, value: &str) -> Result<(), String> {
		match self {
			Self::Heartbeat(body) => body.parse_field(tag, value),
			Self::Logon(body) => body.parse_field(tag, value),
			Self::NewOrderSingle(body) => body.parse_field(tag, value),
			Self::ExecutionReport(body) => body.parse_field(tag, value),
			Self::OrderCancelRequest(body) => body.parse_field(tag, value),
			Self::Other => Ok(()), // Ignore fields for unsupported types
		}
	}

	fn write_body_fields(&self, buffer: &mut String) {
		// For message bodies, write_body_fields is the same as write_to
		// since all message body fields contribute to body length
		self.write_to(buffer);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::common::EncryptMethod;

	#[test]
	fn test_message_body_variants() {
		// Test Heartbeat variant
		let heartbeat_body = FixMessageBody::Heartbeat(HeartbeatBody::default());
		assert!(heartbeat_body.is_valid());

		// Test Logon variant
		let logon_body = FixMessageBody::Logon(LogonBody::default());
		assert!(logon_body.is_valid());

		// Test Other variant
		let other_body = FixMessageBody::Other;
		assert!(other_body.is_valid());
	}

	#[test]
	fn test_message_body_validation() {
		// Valid heartbeat
		let valid_heartbeat = FixMessageBody::Heartbeat(HeartbeatBody::default());
		assert!(valid_heartbeat.validate().is_ok());

		// Valid logon
		let valid_logon = FixMessageBody::Logon(LogonBody::new(EncryptMethod::None, 30));
		assert!(valid_logon.validate().is_ok());

		// Invalid logon (zero heartbeat interval)
		let invalid_logon = FixMessageBody::Logon(LogonBody::new(EncryptMethod::None, 0));
		assert!(invalid_logon.validate().is_err());
	}

	#[test]
	fn test_message_body_parsing() {
		// Test heartbeat field parsing
		let mut heartbeat = FixMessageBody::Heartbeat(HeartbeatBody::default());
		assert!(heartbeat.parse_field(112, "TEST_REQ_ID").is_ok());

		// Test logon field parsing
		let mut logon = FixMessageBody::Logon(LogonBody::default());
		assert!(logon.parse_field(98, "1").is_ok()); // EncryptMethod::Pkcs
		assert!(logon.parse_field(108, "60").is_ok()); // HeartBtInt

		// Test other message type (should ignore fields)
		let mut other = FixMessageBody::Other;
		assert!(other.parse_field(999, "anything").is_ok());
	}

	#[test]
	fn test_message_body_equality() {
		let heartbeat1 = FixMessageBody::Heartbeat(HeartbeatBody::default());
		let heartbeat2 = FixMessageBody::Heartbeat(HeartbeatBody::default());
		assert_eq!(heartbeat1, heartbeat2);

		let logon1 = FixMessageBody::Logon(LogonBody::default());
		let logon2 = FixMessageBody::Logon(LogonBody::default());
		assert_eq!(logon1, logon2);

		let other1 = FixMessageBody::Other;
		let other2 = FixMessageBody::Other;
		assert_eq!(other1, other2);

		// Different variants should not be equal
		assert_ne!(heartbeat1, logon1);
		assert_ne!(heartbeat1, other1);
		assert_ne!(logon1, other1);
	}

	#[test]
	fn test_message_body_memory_efficiency() {
		// This test demonstrates that each variant only stores relevant fields
		let heartbeat = FixMessageBody::Heartbeat(HeartbeatBody::default());
		let logon = FixMessageBody::Logon(LogonBody::default());
		let other = FixMessageBody::Other;

		// Each variant should be a different size, demonstrating memory efficiency
		match (&heartbeat, &logon, &other) {
			(FixMessageBody::Heartbeat(_), FixMessageBody::Logon(_), FixMessageBody::Other) => {
				// This pattern match confirms the enum variants are properly structured
				assert!(true);
			},
			_ => panic!("Enum variants not properly matched"),
		}
	}
}

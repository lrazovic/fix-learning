//! Heartbeat message implementation (MsgType=0)
//!
//! This module implements the FIX 4.2 Heartbeat message, which is used for
//! session-level communication to maintain connection liveness and respond
//! to test requests.

use crate::common::{SOH, Validate, ValidationError, validation::WriteTo};
use std::fmt::Write;

/// Heartbeat message body (Tag 35=0)
///
/// The Heartbeat message is sent periodically to maintain session liveness.
/// It can also be sent in response to a Test Request message, in which case
/// it must include the TestReqID from the original Test Request.
#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct HeartbeatBody {
	/// Test request ID (Tag 112) - Required when the heartbeat is the result of a Test Request message
	pub test_req_id: Option<String>,
}

impl Validate for HeartbeatBody {
	fn validate(&self) -> Result<(), ValidationError> {
		// Heartbeat has no required fields beyond header
		// TestReqID is optional and only required when responding to TestRequest
		Ok(())
	}
}

impl WriteTo for HeartbeatBody {
	fn write_to(&self, buffer: &mut String) {
		if let Some(ref test_req_id) = self.test_req_id {
			write!(buffer, "112={}{}", test_req_id, SOH).unwrap();
		}
	}
}

impl HeartbeatBody {
	/// Create a new empty heartbeat body
	pub fn new() -> Self {
		Self::default()
	}

	/// Create a heartbeat responding to a test request
	pub fn responding_to_test_request(test_req_id: impl Into<String>) -> Self {
		Self { test_req_id: Some(test_req_id.into()) }
	}

	/// Parse a heartbeat-specific field
	pub(crate) fn parse_field(&mut self, tag: u32, value: &str) -> Result<(), String> {
		match tag {
			112 => self.test_req_id = Some(value.to_string()),
			_ => return Err(format!("Unknown heartbeat field: {}", tag)),
		}
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_heartbeat_creation() {
		let heartbeat = HeartbeatBody::new();
		assert!(heartbeat.validate().is_ok());
		assert_eq!(heartbeat.test_req_id, None);
	}

	#[test]
	fn test_heartbeat_default() {
		let heartbeat = HeartbeatBody::default();
		assert!(heartbeat.validate().is_ok());
		assert_eq!(heartbeat.test_req_id, None);
	}

	#[test]
	fn test_heartbeat_with_test_request() {
		let heartbeat = HeartbeatBody::responding_to_test_request("TEST123");
		assert!(heartbeat.validate().is_ok());
		assert_eq!(heartbeat.test_req_id, Some("TEST123".to_string()));
	}

	#[test]
	fn test_heartbeat_field_parsing() {
		let mut heartbeat = HeartbeatBody::new();

		// Parse test request ID
		assert!(heartbeat.parse_field(112, "TEST_123").is_ok());
		assert_eq!(heartbeat.test_req_id, Some("TEST_123".to_string()));

		// Parse unknown field
		assert!(heartbeat.parse_field(999, "unknown").is_err());
	}

	#[test]
	fn test_heartbeat_validation() {
		// Basic heartbeat should always be valid
		let heartbeat = HeartbeatBody::new();
		assert!(heartbeat.is_valid());

		// Heartbeat with test request ID should be valid
		let heartbeat = HeartbeatBody::responding_to_test_request("TEST");
		assert!(heartbeat.is_valid());

		// Empty test request ID should still be valid
		let heartbeat = HeartbeatBody { test_req_id: Some("".to_string()) };
		assert!(heartbeat.is_valid());
	}

	#[test]
	fn test_heartbeat_equality() {
		let heartbeat1 = HeartbeatBody::new();
		let heartbeat2 = HeartbeatBody::default();
		assert_eq!(heartbeat1, heartbeat2);

		let heartbeat3 = HeartbeatBody::responding_to_test_request("TEST");
		let heartbeat4 = HeartbeatBody { test_req_id: Some("TEST".to_string()) };
		assert_eq!(heartbeat3, heartbeat4);

		assert_ne!(heartbeat1, heartbeat3);
	}

	#[test]
	fn test_heartbeat_cloning() {
		let original = HeartbeatBody::responding_to_test_request("CLONE_TEST");
		let cloned = original.clone();

		assert_eq!(original, cloned);
		assert_eq!(original.test_req_id, cloned.test_req_id);
	}
}

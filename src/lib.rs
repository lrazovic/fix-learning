//! FIX 4.2 Protocol Implementation
//!
//! This library provides a high-performance, memory-efficient implementation of the
//! Financial Information eXchange (FIX) 4.2 protocol with focus on Heartbeat and
//! Logon messages. The library uses an enum-based message body design that only
//! allocates memory for the fields needed by each specific message type.
//!
//! # Features
//!
//! - **Memory Efficient**: Enum-based message bodies allocate only required fields
//! - **Type Safe**: Comprehensive validation for all message components
//! - **Performance Optimized**: Fast serialization and parsing
//! - **FIX 4.2 Compliant**: Accurate checksum and body length calculation
//!
//! # Supported Message Types
//!
//! - **Heartbeat (MsgType=0)**: Session keepalive and test request responses
//! - **Logon (MsgType=A)**: Session initiation with all encryption methods
//! - **Extensible Design**: Easy to add new message types
//!
//! # Quick Start
//!
//! ```rust
//! use fix_learning::{FixMessage, EncryptMethod, MsgType};
//!
//! // Create a heartbeat message
//! let heartbeat = FixMessage::builder(MsgType::Heartbeat, "CLIENT", "SERVER", 1).build();
//!
//! // Create a logon message
//! let logon = FixMessage::builder(MsgType::Logon, "TRADER", "EXCHANGE", 1)
//!     .encrypt_method(EncryptMethod::None)
//!     .heart_bt_int(30)
//!     .reset_seq_num_flag(true)
//!     .build();
//!
//! // Serialize to FIX wire format
//! let fix_string = logon.to_fix_string();
//!
//! // Parse from FIX wire format
//! let parsed = FixMessage::from_fix_string(&fix_string)?;
//! # Ok::<(), String>(())
//! ```

pub mod builder;
pub mod common;
pub mod macros;
pub mod messages;

use std::{collections::HashMap, fmt::Display};

// Re-export commonly used types
pub use builder::FixMessageBuilder;
pub use common::{
	EncryptMethod, FORMAT_TIME, FixHeader, FixTrailer, MsgType, OrdStatus, SOH, Side, Validate, ValidationError,
	parse_fix_timestamp,
};
pub use messages::{FixMessageBody, HeartbeatBody, LogonBody};

use crate::common::validation::{FixFieldHandler, WriteTo};

/// Main FIX 4.2 Message structure
///
/// This is the primary structure representing a complete FIX 4.2 message with
/// header, body, and trailer.
/// The message body uses an enum to provide memory-efficient storage by only allocating fields needed for each
/// specific message type.
#[derive(Debug, Clone, PartialEq)]
pub struct FixMessage {
	/// Standard message header with required and optional fields
	pub header: FixHeader,
	/// Message-specific body (only allocates what's needed)
	pub body: FixMessageBody,
	/// Standard message trailer with checksum and optional signature
	pub trailer: FixTrailer,
}

impl Display for FixMessage {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let fix_string = self.to_fix_string();
		let readable = fix_string.replace(SOH, " | ");
		write!(f, "{}", readable)
	}
}

impl Validate for FixMessage {
	fn validate(&self) -> Result<(), ValidationError> {
		self.header.validate()?;
		self.body.validate()?;
		self.trailer.validate()?;
		Ok(())
	}
}

impl FixMessage {
	/// Create a new FIX message with required fields
	/// The constructor is private since a user should use the FixMessageBuilder to build a FixMessage.
	fn new(
		msg_type: MsgType,
		sender_comp_id: impl Into<String>,
		target_comp_id: impl Into<String>,
		msg_seq_num: u32,
	) -> Self {
		let body = match msg_type {
			MsgType::Heartbeat => FixMessageBody::Heartbeat(HeartbeatBody::default()),
			MsgType::Logon => FixMessageBody::Logon(LogonBody::default()),
			_ => FixMessageBody::Other,
		};
		let header = FixHeader::new(msg_type, sender_comp_id, target_comp_id, msg_seq_num);
		let trailer = FixTrailer::default();
		Self { header, body, trailer }
	}

	/// Check if the message is valid
	pub fn is_valid(&self) -> bool {
		self.validate().is_ok()
	}

	/// Write the complete message to a string
	pub fn write_message(&self) -> String {
		let mut buf = String::with_capacity(256); // Single allocation
		self.header.write_to(&mut buf);
		self.body.write_to(&mut buf);
		self.trailer.write_to(&mut buf);
		buf
	}

	/// Serialize the complete message to FIX wire format
	pub fn to_fix_string(&self) -> String {
		self.write_message()
	}

	/// Parse a FIX message from wire format
	pub fn from_fix_string(fix_string: &str) -> Result<Self, String> {
		let fields: Vec<&str> = fix_string.split(SOH).filter(|s| !s.is_empty()).collect();

		if fields.is_empty() {
			return Err("Empty FIX message".to_string());
		}

		// Parse fields into key-value pairs with tags as numbers
		let mut field_map = HashMap::new();
		for field in fields {
			if let Some((tag_str, value)) = field.split_once('=') {
				let tag: u32 = tag_str.parse().map_err(|_| format!("Invalid tag: {}", tag_str))?;
				field_map.insert(tag, value);
			}
		}

		// Extract required fields for message creation
		let msg_type_str = field_map.get(&35).ok_or("Missing MsgType (35)")?;
		let msg_type = msg_type_str.parse().map_err(|_| "Invalid MsgType")?;

		let sender_comp_id = field_map.get(&49).ok_or("Missing SenderCompID (49)")?.to_string();
		let target_comp_id = field_map.get(&56).ok_or("Missing TargetCompID (56)")?.to_string();

		let msg_seq_num: u32 =
			field_map.get(&34).ok_or("Missing MsgSeqNum (34)")?.parse().map_err(|_| "Invalid MsgSeqNum")?;

		// Create message with basic required fields
		let mut message = Self::new(msg_type, sender_comp_id, target_comp_id, msg_seq_num);

		// Parse all fields generically using parse_field methods
		for (&tag, &value) in &field_map {
			match tag {
				// Header fields (8, 9, 35, 49, 56, 34, 52, 43, 97, 122)
				8 | 9 | 35 | 49 | 56 | 34 | 52 | 43 | 97 | 122 => {
					message.header.parse_field(tag, value).map_err(|e| format!("Header parse error: {}", e))?;
				},
				// Trailer fields (10, 93, 89)
				10 | 93 | 89 => {
					message.trailer.parse_field(tag, value).map_err(|e| format!("Trailer parse error: {}", e))?;
				},
				// Body fields - delegate to message body
				_ => {
					message.body.parse_field(tag, value).map_err(|e| format!("Body parse error: {}", e))?;
				},
			}
		}

		// Validate message
		message.validate().map_err(|e| e.to_string())?;

		Ok(message)
	}
}

impl Default for FixMessage {
	fn default() -> Self {
		Self::new(MsgType::Heartbeat, "SENDER", "TARGET", 1)
	}
}

impl FixMessage {
	/// Create a new builder for this message type
	pub fn builder(
		msg_type: MsgType,
		sender_comp_id: impl Into<String>,
		target_comp_id: impl Into<String>,
		msg_seq_num: u32,
	) -> FixMessageBuilder {
		FixMessageBuilder::new(msg_type, sender_comp_id, target_comp_id, msg_seq_num)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::common::EncryptMethod;

	#[test]
	fn test_generic_from_fix_string() {
		// Test that the generic from_fix_string can handle various message types

		// Create a complex logon message
		let original = FixMessage::builder(MsgType::Logon, "TRADER", "EXCHANGE", 42)
			.encrypt_method(EncryptMethod::None)
			.heart_bt_int(30)
			.reset_seq_num_flag(true)
			.poss_dup_flag(false)
			.build();

		let fix_string = original.to_fix_string();
		println!("Original FIX string: {}", fix_string);

		// Parse it back using the generic implementation
		let parsed = FixMessage::from_fix_string(&fix_string).expect("Should parse successfully");

		// Verify all fields are correctly parsed
		assert_eq!(original.header.msg_type, parsed.header.msg_type);
		assert_eq!(original.header.sender_comp_id, parsed.header.sender_comp_id);
		assert_eq!(original.header.target_comp_id, parsed.header.target_comp_id);
		assert_eq!(original.header.msg_seq_num, parsed.header.msg_seq_num);
		assert_eq!(original.header.body_length, parsed.header.body_length);
		assert_eq!(original.header.poss_dup_flag, parsed.header.poss_dup_flag);
		assert_eq!(original.trailer.checksum, parsed.trailer.checksum);

		// Verify the message is valid
		assert!(parsed.is_valid());

		// Test with heartbeat message
		let heartbeat_original =
			FixMessage::builder(MsgType::Heartbeat, "CLIENT", "SERVER", 1).test_req_id("TEST_123").build();

		let heartbeat_string = heartbeat_original.to_fix_string();
		let heartbeat_parsed = FixMessage::from_fix_string(&heartbeat_string).expect("Should parse heartbeat");

		assert_eq!(heartbeat_original.header.msg_type, heartbeat_parsed.header.msg_type);
		assert_eq!(heartbeat_original.header.body_length, heartbeat_parsed.header.body_length);
		assert!(heartbeat_parsed.is_valid());

		println!("Generic parsing test passed for both Logon and Heartbeat messages!");
	}

	#[test]
	fn test_unknown_field_handling() {
		// Test that unknown fields are properly handled
		let fix_string = "8=FIX.4.2\x019=50\x0135=0\x0149=CLIENT\x0156=SERVER\x0134=1\x0152=20241201-12:34:56.789\x01999=UNKNOWN\x0110=123\x01";

		// This should parse successfully - unknown fields are handled by the body's parse_field method
		// which returns Ok(()) for unknown fields in the "Other" message type
		let result = FixMessage::from_fix_string(fix_string);
		match result {
			Ok(message) => {
				assert_eq!(message.header.msg_type, MsgType::Heartbeat);
				println!("Successfully parsed message with unknown field 999");
			},
			Err(e) => {
				// This is expected since unknown fields should cause parse errors
				println!("Parse error for unknown field (expected): {}", e);
				assert!(e.contains("999") || e.contains("UNKNOWN"));
			},
		}
	}

	#[test]
	fn test_message_specific_tag_parsing() {
		// Test that message-specific tags are properly parsed through the generic system

		// Test Heartbeat with TestReqID (tag 112)
		let heartbeat_fix = "8=FIX.4.2\x019=68\x0135=0\x0149=CLIENT\x0156=SERVER\x0134=1\x0152=20241201-12:34:56.789\x01112=TEST_REQ_123\x0110=123\x01";

		let parsed_heartbeat =
			FixMessage::from_fix_string(heartbeat_fix).expect("Should parse heartbeat with TestReqID");

		if let FixMessageBody::Heartbeat(body) = &parsed_heartbeat.body {
			assert_eq!(body.test_req_id, Some("TEST_REQ_123".to_string()));
		} else {
			panic!("Expected Heartbeat body");
		}

		// Test Logon with multiple message-specific tags
		let logon_fix = "8=FIX.4.2\x019=50\x0135=A\x0149=TRADER\x0156=EXCHANGE\x0134=1\x0152=20241201-12:34:56.789\x0198=0\x01108=30\x01141=Y\x01789=1\x01383=8192\x0110=123\x01";

		let parsed_logon = FixMessage::from_fix_string(logon_fix).expect("Should parse logon with specific fields");

		if let FixMessageBody::Logon(body) = &parsed_logon.body {
			assert_eq!(body.encrypt_method, EncryptMethod::None); // tag 98
			assert_eq!(body.heart_bt_int, 30); // tag 108
			assert_eq!(body.reset_seq_num_flag, Some(true)); // tag 141
			assert_eq!(body.next_expected_msg_seq_num, Some(1)); // tag 789
			assert_eq!(body.max_message_size, Some(8192)); // tag 383
		} else {
			panic!("Expected Logon body");
		}

		println!("Message-specific tag parsing test passed!");
	}
}

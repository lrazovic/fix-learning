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

use crate::common::validation::WriteTo;

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

		// Parse fields into key-value pairs
		let mut field_map = HashMap::new();
		for field in fields {
			if let Some((tag, value)) = field.split_once('=') {
				field_map.insert(tag, value);
			}
		}

		// Parse required header fields
		let begin_string = field_map.get("8").ok_or("Missing BeginString (8)")?;
		if *begin_string != "FIX.4.2" {
			return Err(format!("Unsupported FIX version: {}", begin_string));
		}

		let body_length: u32 =
			field_map.get("9").ok_or("Missing BodyLength (9)")?.parse().map_err(|_| "Invalid BodyLength")?;

		let msg_type_str = field_map.get("35").ok_or("Missing MsgType (35)")?;
		let msg_type = msg_type_str.parse().map_err(|_| "Invalid MsgType")?;

		let sender_comp_id = field_map.get("49").ok_or("Missing SenderCompID (49)")?.to_string();
		let target_comp_id = field_map.get("56").ok_or("Missing TargetCompID (56)")?.to_string();

		let msg_seq_num: u32 =
			field_map.get("34").ok_or("Missing MsgSeqNum (34)")?.parse().map_err(|_| "Invalid MsgSeqNum")?;

		let sending_time_str = field_map.get("52").ok_or("Missing SendingTime (52)")?;
		let sending_time = parse_fix_timestamp(sending_time_str)?;

		// Create message
		let mut message = Self::new(msg_type, sender_comp_id, target_comp_id, msg_seq_num);
		message.header.body_length = body_length;
		message.header.sending_time = sending_time;

		// Parse optional header fields
		if let Some(flag_str) = field_map.get("43") {
			message.header.poss_dup_flag = Some(*flag_str == "Y");
		}
		if let Some(flag_str) = field_map.get("97") {
			message.header.poss_resend = Some(*flag_str == "Y");
		}
		if let Some(time_str) = field_map.get("122") {
			message.header.orig_sending_time = Some(parse_fix_timestamp(time_str)?);
		}

		// Parse body based on message type
		match &mut message.body {
			FixMessageBody::Heartbeat(_) =>
				if let Some(test_req_id) = field_map.get("112") {
					message.body.parse_field(112, test_req_id).map_err(|e| format!("Parse error: {}", e))?;
				},
			FixMessageBody::Logon(_) => {
				// Parse required logon fields
				if let Some(encrypt_str) = field_map.get("98") {
					message.body.parse_field(98, encrypt_str).map_err(|e| format!("Parse error: {}", e))?;
				}
				if let Some(heartbt_str) = field_map.get("108") {
					message.body.parse_field(108, heartbt_str).map_err(|e| format!("Parse error: {}", e))?;
				}
				// Parse optional logon fields
				if let Some(flag_str) = field_map.get("141") {
					message.body.parse_field(141, flag_str).map_err(|e| format!("Parse error: {}", e))?;
				}
				if let Some(seq_str) = field_map.get("789") {
					message.body.parse_field(789, seq_str).map_err(|e| format!("Parse error: {}", e))?;
				}
				if let Some(size_str) = field_map.get("383") {
					message.body.parse_field(383, size_str).map_err(|e| format!("Parse error: {}", e))?;
				}
			},
			FixMessageBody::NewOrderSingle(_) => {
				todo!();
			},
			FixMessageBody::Other => {
				// No specific parsing for other message types
			},
		}

		// Parse trailer
		if let Some(checksum_str) = field_map.get("10") {
			message.trailer.checksum = checksum_str.to_string();
		}
		if let Some(sig_len_str) = field_map.get("93") {
			message.trailer.signature_length = Some(sig_len_str.parse().map_err(|_| "Invalid SignatureLength")?);
		}
		if let Some(signature) = field_map.get("89") {
			message.trailer.signature = Some(signature.to_string());
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

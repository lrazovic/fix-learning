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
use time::{format_description::BorrowedFormatItem, macros::format_description};

// Re-export commonly used types
pub use builder::FixMessageBuilder;
pub use common::{
	EncryptMethod, FixHeader, FixTrailer, MsgType, OrdStatus, SOH, Side, Validate, ValidationError, parse_fix_timestamp,
};
pub use messages::{FixMessageBody, HeartbeatBody, LogonBody};

/// Time/date combination format for FIX timestamps
static FORMAT_TIME: &[BorrowedFormatItem<'_>] =
	format_description!("[year][month][day]-[hour]:[minute]:[second].[subsecond digits:3]");

/// Main FIX 4.2 Message structure
///
/// This is the primary structure representing a complete FIX message with
/// header, body, and trailer. The message body uses an enum to provide
/// memory-efficient storage by only allocating fields needed for each
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

	/// Calculate body length (excludes header tags 8, 9, 10 and the checksum)
	pub fn calculate_body_length(&self) -> u32 {
		let body_string = self.serialize_body_and_trailer_without_checksum();
		body_string.len() as u32
	}

	/// Calculate FIX checksum (sum of all bytes modulo 256)
	pub fn calculate_checksum(&self) -> String {
		let message_without_checksum = self.serialize_without_checksum();
		let checksum: u32 = message_without_checksum.bytes().map(|b| b as u32).sum::<u32>() % 256;
		format!("{:03}", checksum)
	}

	/// Serialize message without checksum for checksum calculation
	pub fn serialize_without_checksum(&self) -> String {
		let mut result = String::new();

		// Header
		result.push_str(&format!("8={}{}", self.header.begin_string, SOH));
		result.push_str(&format!("9={}{}", self.calculate_body_length(), SOH));
		result.push_str(&self.serialize_body_and_trailer_without_checksum());

		result
	}

	/// Serialize body and trailer without checksum
	pub fn serialize_body_and_trailer_without_checksum(&self) -> String {
		let mut result = String::new();

		// Message type
		result.push_str(&format!("35={}{}", self.header.msg_type, SOH));

		// Required header fields
		result.push_str(&format!("49={}{}", self.header.sender_comp_id, SOH));
		result.push_str(&format!("56={}{}", self.header.target_comp_id, SOH));
		result.push_str(&format!("34={}{}", self.header.msg_seq_num, SOH));

		// Format sending time
		let sending_time_str = self.header.sending_time.format(&FORMAT_TIME).unwrap_or_default();
		result.push_str(&format!("52={}{}", sending_time_str, SOH));

		// Optional header fields
		if let Some(flag) = self.header.poss_dup_flag {
			result.push_str(&format!("43={}{}", if flag { "Y" } else { "N" }, SOH));
		}
		if let Some(flag) = self.header.poss_resend {
			result.push_str(&format!("97={}{}", if flag { "Y" } else { "N" }, SOH));
		}
		if let Some(time) = self.header.orig_sending_time {
			let time_str = time.format(&FORMAT_TIME).unwrap_or_default();
			result.push_str(&format!("122={}{}", time_str, SOH));
		}

		// Body fields
		result.push_str(&self.body.serialize_fields());

		// Optional trailer fields (excluding checksum)
		if let Some(sig_len) = self.trailer.signature_length {
			result.push_str(&format!("93={}{}", sig_len, SOH));
		}
		if let Some(ref signature) = self.trailer.signature {
			result.push_str(&format!("89={}{}", signature, SOH));
		}

		result
	}

	/// Serialize the complete message to FIX wire format
	pub fn to_fix_string(&self) -> String {
		let mut result = self.serialize_without_checksum();
		let checksum = self.calculate_checksum();
		result.push_str(&format!("10={}{}", checksum, SOH));
		result
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

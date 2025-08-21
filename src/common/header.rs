//! FIX message header structures
//!
//! This module contains the standard FIX message header structure
//! that is common to all FIX messages, along with its validation logic.

use crate::{
	FORMAT_TIME, SOH,
	common::{
		enums::MsgType,
		validation::{Validate, ValidationError, WriteTo},
	},
};
use std::fmt::Write;
use time::{Duration, OffsetDateTime, PrimitiveDateTime, UtcOffset, macros::format_description};

/// Standard FIX message header
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct FixHeader {
	// Required Header Fields
	pub begin_string: &'static str,   // Tag 8 - Always "FIX.4.2"
	pub body_length: u32,             // Tag 9 - Length of message body
	pub msg_type: MsgType,            // Tag 35 - Message type
	pub sender_comp_id: String,       // Tag 49 - Sender's company ID
	pub target_comp_id: String,       // Tag 56 - Target's company ID
	pub msg_seq_num: u32,             // Tag 34 - Message sequence number
	pub sending_time: OffsetDateTime, // Tag 52 - Time of message transmission

	// Optional Header Fields
	pub poss_dup_flag: Option<bool>,               // Tag 43 - Possible duplicate flag
	pub poss_resend: Option<bool>,                 // Tag 97 - Possible resend flag
	pub orig_sending_time: Option<OffsetDateTime>, // Tag 122 - Original sending time
}

impl FixHeader {
	/// Create a new FIX header with required fields
	pub fn new(
		msg_type: MsgType,
		sender_comp_id: impl Into<String>,
		target_comp_id: impl Into<String>,
		msg_seq_num: u32,
	) -> Self {
		Self {
			begin_string: "FIX.4.2",
			body_length: 0, // Will be calculated later
			msg_type,
			sender_comp_id: sender_comp_id.into(),
			target_comp_id: target_comp_id.into(),
			msg_seq_num,
			sending_time: OffsetDateTime::now_utc(),
			poss_dup_flag: None,
			poss_resend: None,
			orig_sending_time: None,
		}
	}
}

impl Validate for FixHeader {
	fn validate(&self) -> Result<(), ValidationError> {
		if self.begin_string != "FIX.4.2" {
			return Err(ValidationError::VersionMismatch);
		}
		if self.sender_comp_id.is_empty() {
			return Err(ValidationError::EmptyMessage);
		}
		if self.target_comp_id.is_empty() {
			return Err(ValidationError::EmptyMessage);
		}
		if self.msg_seq_num == 0 {
			return Err(ValidationError::EmptyMessage);
		}
		if self.sending_time.year() < 1970 {
			return Err(ValidationError::EmptyMessage);
		}
		Ok(())
	}
}

impl WriteTo for FixHeader {
	fn write_to(&self, buffer: &mut String) {
		write!(buffer, "8={}{}", self.begin_string, SOH).unwrap();
		write!(buffer, "9={}{}", self.body_length, SOH).unwrap();
		write!(buffer, "35={}{}", self.msg_type, SOH).unwrap();
		write!(buffer, "49={}{}", self.sender_comp_id, SOH).unwrap();
		write!(buffer, "56={}{}", self.target_comp_id, SOH).unwrap();
		write!(buffer, "34={}{}", self.msg_seq_num, SOH).unwrap();
		write!(buffer, "52={}{}", self.sending_time, SOH).unwrap();
		if let Some(ref poss_dup_flag) = self.poss_dup_flag {
			write!(buffer, "43={}{}", poss_dup_flag, SOH).unwrap();
		}
		if let Some(ref poss_resend) = self.poss_resend {
			write!(buffer, "97={}{}", poss_resend, SOH).unwrap();
		}
		if let Some(ref orig_sending_time) = self.orig_sending_time {
			write!(buffer, "122={}{}", orig_sending_time, SOH).unwrap();
		}
	}
}

/// Time parsing utilities for FIX timestamps
pub fn parse_fix_timestamp(s: &str) -> Result<OffsetDateTime, String> {
	// Leap second handling
	let (s, leap) = if s.contains(":60") { (s.replace(":60", ":59"), true) } else { (s.to_string(), false) };

	let fmt_millis = FORMAT_TIME;
	let fmt_seconds = format_description!("[year][month][day]-[hour]:[minute]:[second]");

	let parsed = PrimitiveDateTime::parse(&s, &fmt_millis)
		.or_else(|_| PrimitiveDateTime::parse(&s, &fmt_seconds))
		.map_err(|e| format!("Invalid timestamp '{}': {}", s, e))?;

	let mut dt = parsed.assume_offset(UtcOffset::UTC);
	if leap {
		dt += Duration::seconds(1);
	}
	Ok(dt)
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::common::enums::MsgType;

	#[test]
	fn test_header_creation() {
		let header = FixHeader::new(MsgType::Heartbeat, "SENDER", "TARGET", 1);

		assert_eq!(header.begin_string, "FIX.4.2");
		assert_eq!(header.msg_type, MsgType::Heartbeat);
		assert_eq!(header.sender_comp_id, "SENDER");
		assert_eq!(header.target_comp_id, "TARGET");
		assert_eq!(header.msg_seq_num, 1);
		assert!(header.is_valid());
	}

	#[test]
	fn test_header_validation() {
		// Valid header
		let valid_header = FixHeader::new(MsgType::Heartbeat, "SENDER", "TARGET", 1);
		assert!(valid_header.is_valid());

		// Invalid header - empty sender
		let invalid_header = FixHeader::new(MsgType::Heartbeat, "", "TARGET", 1);
		assert!(!invalid_header.is_valid());

		// Invalid header - empty target
		let invalid_header = FixHeader::new(MsgType::Heartbeat, "SENDER", "", 1);
		assert!(!invalid_header.is_valid());

		// Invalid header - zero sequence number
		let invalid_header = FixHeader::new(MsgType::Heartbeat, "SENDER", "TARGET", 0);
		assert!(!invalid_header.is_valid());
	}

	#[test]
	fn test_timestamp_parsing() {
		// Valid timestamps
		assert!(parse_fix_timestamp("20241201-12:34:56.789").is_ok());
		assert!(parse_fix_timestamp("20241201-12:34:56").is_ok());

		// Leap second handling
		assert!(parse_fix_timestamp("20241201-12:34:60.000").is_ok());

		// Invalid timestamps
		assert!(parse_fix_timestamp("invalid").is_err());
		assert!(parse_fix_timestamp("20241301-12:34:56").is_err()); // Invalid month
	}
}

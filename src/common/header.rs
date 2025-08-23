//! FIX message header structures
//!
//! This module contains the standard FIX message header structure
//! that is common to all FIX messages, along with its validation logic.

use crate::{
	SOH,
	common::{
		enums::MsgType,
		validation::{FixFieldHandler, Validate, ValidationError, WriteTo},
		write_tag_timestamp,
	},
};
use std::fmt::Write;
use time::{Duration, OffsetDateTime};

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
		self.write_body_fields(buffer);
	}
}

impl FixFieldHandler for FixHeader {
	fn parse_field(&mut self, tag: u32, value: &str) -> Result<(), String> {
		match tag {
			8 => {
				if value != "FIX.4.2" {
					return Err(format!("Unsupported FIX version: {}", value));
				}
				// begin_string is already set as constant
			},
			9 => {
				self.body_length = value.parse().map_err(|_| "Invalid BodyLength")?;
			},
			35 => {
				// MsgType is immutable after creation, so we skip parsing it here
				// The caller should ensure the message type matches
			},
			49 => {
				self.sender_comp_id = value.to_string();
			},
			56 => {
				self.target_comp_id = value.to_string();
			},
			34 => {
				self.msg_seq_num = value.parse().map_err(|_| "Invalid MsgSeqNum")?;
			},
			52 => {
				self.sending_time = parse_fix_timestamp(value)?;
			},
			43 => {
				self.poss_dup_flag = Some(value == "Y");
			},
			97 => {
				self.poss_resend = Some(value == "Y");
			},
			122 => {
				self.orig_sending_time = Some(parse_fix_timestamp(value)?);
			},
			_ => return Err(format!("Unknown header field: {}", tag)),
		}
		Ok(())
	}

	fn write_body_fields(&self, buffer: &mut String) {
		write!(buffer, "35={}{}", self.msg_type, SOH).unwrap();
		write!(buffer, "49={}{}", self.sender_comp_id, SOH).unwrap();
		write!(buffer, "56={}{}", self.target_comp_id, SOH).unwrap();
		write!(buffer, "34={}{}", self.msg_seq_num, SOH).unwrap();
		write_tag_timestamp(buffer, 52, self.sending_time);
		if let Some(ref poss_dup_flag) = self.poss_dup_flag {
			write!(buffer, "43={}{}", poss_dup_flag, SOH).unwrap();
		}
		if let Some(ref poss_resend) = self.poss_resend {
			write!(buffer, "97={}{}", poss_resend, SOH).unwrap();
		}
		if let Some(ref orig_sending_time) = self.orig_sending_time {
			write_tag_timestamp(buffer, 122, *orig_sending_time);
		}
	}
}

/// Time parsing utilities for FIX timestamps
pub fn parse_fix_timestamp(s: &str) -> Result<OffsetDateTime, String> {
	// FIX timestamps are always: YYYYMMDD-HH:MM:SS[.sss]
	if s.len() < 17 {
		return Err(format!("Timestamp too short: {}", s));
	}

	// Just use parse() on each substring - it's cleaner than manual digit parsing
	let year: i32 = s[0..4].parse().map_err(|_| format!("Invalid year: {}", s))?;
	let month: u8 = s[4..6].parse().map_err(|_| format!("Invalid month: {}", s))?;
	let day: u8 = s[6..8].parse().map_err(|_| format!("Invalid day: {}", s))?;
	let hour: u8 = s[9..11].parse().map_err(|_| format!("Invalid hour: {}", s))?;
	let minute: u8 = s[12..14].parse().map_err(|_| format!("Invalid minute: {}", s))?;
	let second: u8 = s[15..17].parse().map_err(|_| format!("Invalid second: {}", s))?;

	let millisecond = if s.len() >= 21 && s.chars().nth(17) == Some('.') {
		s[18..21].parse().map_err(|_| format!("Invalid milliseconds: {}", s))?
	} else {
		0
	};

	// Handle leap second
	let (second, leap) = if second == 60 { (59, true) } else { (second, false) };

	use time::{Date, Month, PrimitiveDateTime, Time};

	let month = Month::try_from(month).map_err(|e| format!("Invalid month: {:?}", e))?;

	let date = Date::from_calendar_date(year, month, day).map_err(|e| format!("Invalid date: {:?}", e))?;
	let time = Time::from_hms_milli(hour, minute, second, millisecond).map_err(|e| format!("Invalid time: {:?}", e))?;

	let mut dt = PrimitiveDateTime::new(date, time).assume_utc();
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

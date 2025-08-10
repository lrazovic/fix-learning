//! FIX message builder for fluent construction
//!
//! This module provides a builder pattern for constructing FIX messages
//! with a fluent, type-safe API. The builder automatically handles
//! body length calculation and checksum generation.

use crate::{
	FixMessage,
	common::{EncryptMethod, FixHeader, FixTrailer, MsgType},
	messages::{FixMessageBody, HeartbeatBody, LogonBody},
};
use time::OffsetDateTime;

/// Builder for constructing FIX messages with a fluent API
#[derive(Debug)]
pub struct FixMessageBuilder {
	message: FixMessage,
}

impl FixMessageBuilder {
	/// Create a new builder with required fields
	pub fn new(
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

		Self { message: FixMessage { header, body, trailer } }
	}

	/// Create a builder from an existing message
	pub const fn from_message(message: FixMessage) -> Self {
		Self { message }
	}

	// Header field setters

	/// Set the possible duplicate flag
	pub const fn poss_dup_flag(mut self, flag: bool) -> Self {
		self.message.header.poss_dup_flag = Some(flag);
		self
	}

	/// Set the possible resend flag
	pub const fn poss_resend(mut self, flag: bool) -> Self {
		self.message.header.poss_resend = Some(flag);
		self
	}

	/// Set the original sending time
	pub const fn orig_sending_time(mut self, time: OffsetDateTime) -> Self {
		self.message.header.orig_sending_time = Some(time);
		self
	}

	/// Set the sending time
	pub const fn sending_time(mut self, time: OffsetDateTime) -> Self {
		self.message.header.sending_time = time;
		self
	}

	// Heartbeat body setters

	/// Set the test request ID for heartbeat messages
	pub fn test_req_id(mut self, test_req_id: impl Into<String>) -> Self {
		if let FixMessageBody::Heartbeat(ref mut body) = self.message.body {
			body.test_req_id = Some(test_req_id.into());
		}
		self
	}

	// Logon body setters

	/// Set the encryption method for logon messages
	pub fn encrypt_method(mut self, method: EncryptMethod) -> Self {
		if let FixMessageBody::Logon(body) = &mut self.message.body {
			body.encrypt_method = method;
		} else {
			println!("You are setting an encryption method in an unsupported message :)")
		}
		self
	}

	/// Set the heartbeat interval for logon messages
	pub fn heart_bt_int(mut self, interval: u32) -> Self {
		if let FixMessageBody::Logon(body) = &mut self.message.body {
			body.heart_bt_int = interval;
		}
		self
	}

	/// Set the reset sequence number flag for logon messages
	pub fn reset_seq_num_flag(mut self, flag: bool) -> Self {
		if let FixMessageBody::Logon(body) = &mut self.message.body {
			body.reset_seq_num_flag = Some(flag);
		}
		self
	}

	/// Set the next expected message sequence number for logon messages
	pub fn next_expected_msg_seq_num(mut self, seq_num: u32) -> Self {
		if let FixMessageBody::Logon(body) = &mut self.message.body {
			body.next_expected_msg_seq_num = Some(seq_num);
		}
		self
	}

	/// Set the maximum message size for logon messages
	pub fn max_message_size(mut self, size: u32) -> Self {
		if let FixMessageBody::Logon(body) = &mut self.message.body {
			body.max_message_size = Some(size);
		}
		self
	}

	/// Build the final message with calculated body length and checksum
	pub fn build(mut self) -> FixMessage {
		// Calculate body length
		self.message.header.body_length = self.message.calculate_body_length();

		// Calculate checksum
		self.message.trailer.checksum = self.message.calculate_checksum();

		self.message
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::common::{EncryptMethod, MsgType};

	#[test]
	fn test_builder_creation() {
		let message = FixMessageBuilder::new(MsgType::Heartbeat, "SENDER", "TARGET", 1).build();

		assert_eq!(message.header.msg_type, MsgType::Heartbeat);
		assert_eq!(message.header.sender_comp_id, "SENDER");
		assert_eq!(message.header.target_comp_id, "TARGET");
		assert_eq!(message.header.msg_seq_num, 1);
		assert!(message.is_valid());
	}

	#[test]
	fn test_heartbeat_builder() {
		let message = FixMessageBuilder::new(MsgType::Heartbeat, "CLIENT", "SERVER", 5)
			.poss_dup_flag(true)
			.test_req_id("TEST_REQ_123")
			.build();

		assert_eq!(message.header.msg_type, MsgType::Heartbeat);
		assert_eq!(message.header.poss_dup_flag, Some(true));

		if let FixMessageBody::Heartbeat(body) = &message.body {
			assert_eq!(body.test_req_id, Some("TEST_REQ_123".to_string()));
		} else {
			panic!("Expected Heartbeat body");
		}
	}

	#[test]
	fn test_logon_builder() {
		let message = FixMessageBuilder::new(MsgType::Logon, "TRADER", "EXCHANGE", 1)
			.encrypt_method(EncryptMethod::Des)
			.heart_bt_int(60)
			.reset_seq_num_flag(true)
			.next_expected_msg_seq_num(1)
			.max_message_size(8192)
			.build();

		assert_eq!(message.header.msg_type, MsgType::Logon);

		if let FixMessageBody::Logon(body) = &message.body {
			assert_eq!(body.encrypt_method, EncryptMethod::Des);
			assert_eq!(body.heart_bt_int, 60);
			assert_eq!(body.reset_seq_num_flag, Some(true));
			assert_eq!(body.next_expected_msg_seq_num, Some(1));
			assert_eq!(body.max_message_size, Some(8192));
		} else {
			panic!("Expected Logon body");
		}
	}

	#[test]
	fn test_builder_with_header_fields() {
		let now = OffsetDateTime::now_utc();
		let orig_time = OffsetDateTime::now_utc();

		let message = FixMessageBuilder::new(MsgType::Heartbeat, "SENDER", "TARGET", 42)
			.sending_time(now)
			.poss_dup_flag(false)
			.poss_resend(true)
			.orig_sending_time(orig_time)
			.build();

		assert_eq!(message.header.sending_time, now);
		assert_eq!(message.header.poss_dup_flag, Some(false));
		assert_eq!(message.header.poss_resend, Some(true));
		assert_eq!(message.header.orig_sending_time, Some(orig_time));
	}

	#[test]
	fn test_builder_from_existing_message() {
		let original = FixMessageBuilder::new(MsgType::Heartbeat, "ORIGINAL", "TARGET", 1).build();

		let modified = FixMessageBuilder::from_message(original.clone()).poss_dup_flag(true).build();

		assert_eq!(modified.header.sender_comp_id, original.header.sender_comp_id);
		assert_eq!(modified.header.target_comp_id, original.header.target_comp_id);
		assert_eq!(modified.header.msg_seq_num, original.header.msg_seq_num);
		assert_eq!(modified.header.poss_dup_flag, Some(true));
		assert_ne!(modified.header.poss_dup_flag, original.header.poss_dup_flag);
	}

	#[test]
	fn test_builder_calculates_body_length_and_checksum() {
		let message = FixMessageBuilder::new(MsgType::Logon, "CLIENT", "BROKER", 1)
			.encrypt_method(EncryptMethod::None)
			.heart_bt_int(30)
			.build();

		// Body length should be calculated
		assert!(message.header.body_length > 0);

		// Checksum should be calculated and properly formatted
		assert_eq!(message.trailer.checksum.len(), 3);
		assert!(message.trailer.checksum.chars().all(|c| c.is_ascii_digit()));

		// Verify calculated values are correct
		let expected_body_length = message.calculate_body_length();
		let expected_checksum = message.calculate_checksum();

		assert_eq!(message.header.body_length, expected_body_length);
		assert_eq!(message.trailer.checksum, expected_checksum);
	}

	#[test]
	fn test_builder_validates_messages() {
		// Valid message
		let valid_message = FixMessageBuilder::new(MsgType::Heartbeat, "SENDER", "TARGET", 1).build();
		assert!(valid_message.is_valid());

		// Builder should still create the message even if it would be invalid
		// (validation happens at the message level, not builder level)
		let potentially_invalid = FixMessageBuilder::new(MsgType::Logon, "", "TARGET", 1)
			.encrypt_method(EncryptMethod::None)
			.heart_bt_int(0) // This would make it invalid
			.build();

		// The message should be created but invalid
		assert!(!potentially_invalid.is_valid());
	}
}

//! Tests for the FIX message builder pattern and serialization functionality
//!
//! These tests verify that the builder pattern works correctly and that
//! messages can be serialized to and parsed from FIX wire format.

use fix_learning::{EncryptMethod, FixMessage, FixMessageBody, MsgType};
use time::OffsetDateTime;

#[cfg(test)]
mod builder_pattern_tests {
	use super::*;

	#[test]
	fn basic_builder_creation() {
		let message = FixMessage::builder(MsgType::Heartbeat, "SENDER", "TARGET", 1).build();

		assert_eq!(message.header.msg_type, MsgType::Heartbeat);
		assert_eq!(message.header.sender_comp_id, "SENDER");
		assert_eq!(message.header.target_comp_id, "TARGET");
		assert_eq!(message.header.msg_seq_num, 1);
		assert!(message.is_valid());
	}

	#[test]
	fn builder_fluent_interface() {
		let now = OffsetDateTime::now_utc();
		let message = FixMessage::builder(MsgType::Heartbeat, "CLIENT", "BROKER", 5)
			.sending_time(now)
			.poss_dup_flag(true)
			.test_req_id("TEST_REQ_001")
			.build();

		assert_eq!(message.header.msg_type, MsgType::Heartbeat);
		assert_eq!(message.header.sender_comp_id, "CLIENT");
		assert_eq!(message.header.target_comp_id, "BROKER");
		assert_eq!(message.header.msg_seq_num, 5);
		assert_eq!(message.header.sending_time, now);
		assert_eq!(message.header.poss_dup_flag, Some(true));

		if let FixMessageBody::Heartbeat(body) = &message.body {
			assert_eq!(body.test_req_id, Some("TEST_REQ_001".to_string()));
		} else {
			panic!("Expected Heartbeat body");
		}
	}

	#[test]
	fn logon_builder() {
		let message = FixMessage::builder(MsgType::Logon, "TRADER", "EXCHANGE", 1)
			.reset_seq_num_flag(true)
			.next_expected_msg_seq_num(1)
			.max_message_size(4096)
			.build();

		assert_eq!(message.header.msg_type, MsgType::Logon);
		assert!(message.is_valid());

		if let FixMessageBody::Logon(body) = &message.body {
			assert_eq!(body.encrypt_method, EncryptMethod::None);
			assert_eq!(body.heart_bt_int, 30);
			assert_eq!(body.reset_seq_num_flag, Some(true));
			assert_eq!(body.next_expected_msg_seq_num, Some(1));
			assert_eq!(body.max_message_size, Some(4096));
		} else {
			panic!("Expected Logon body");
		}
	}

	#[test]
	fn builder_all_standard_fields() {
		let orig_time = OffsetDateTime::now_utc();
		let send_time = OffsetDateTime::now_utc();

		let message = FixMessage::builder(MsgType::Heartbeat, "SENDER", "TARGET", 42)
			.sending_time(send_time)
			.poss_dup_flag(true)
			.poss_resend(false)
			.orig_sending_time(orig_time)
			.test_req_id("HEARTBEAT_001")
			.build();

		// Verify header fields
		assert_eq!(message.header.begin_string, "FIX.4.2");
		assert_eq!(message.header.msg_type, MsgType::Heartbeat);
		assert_eq!(message.header.sender_comp_id, "SENDER");
		assert_eq!(message.header.target_comp_id, "TARGET");
		assert_eq!(message.header.msg_seq_num, 42);
		assert_eq!(message.header.sending_time, send_time);
		assert_eq!(message.header.poss_dup_flag, Some(true));
		assert_eq!(message.header.poss_resend, Some(false));
		assert_eq!(message.header.orig_sending_time, Some(orig_time));

		// Verify body length and checksum are calculated
		assert!(message.header.body_length > 0);
		assert_eq!(message.trailer.checksum.len(), 3);
		assert!(message.trailer.checksum.chars().all(|c| c.is_ascii_digit()));

		assert!(message.is_valid());
	}

	#[test]
	fn builder_from_existing_message() {
		let original = FixMessage::builder(MsgType::Heartbeat, "ORIGINAL", "TARGET", 1).build();

		let modified = FixMessage::builder(
			original.header.msg_type.clone(),
			original.header.sender_comp_id.clone(),
			"NEW_TARGET".to_string(),
			original.header.msg_seq_num + 1,
		)
		.poss_dup_flag(true)
		.build();

		assert_eq!(modified.header.msg_type, original.header.msg_type);
		assert_eq!(modified.header.sender_comp_id, original.header.sender_comp_id);
		assert_eq!(modified.header.target_comp_id, "NEW_TARGET");
		assert_eq!(modified.header.msg_seq_num, original.header.msg_seq_num + 1);
		assert_eq!(modified.header.poss_dup_flag, Some(true));
	}

	#[test]
	fn builder_optional_header_fields() {
		let message = FixMessage::builder(MsgType::Heartbeat, "SENDER", "TARGET", 1)
			.poss_dup_flag(false)
			.poss_resend(true)
			.build();

		assert_eq!(message.header.poss_dup_flag, Some(false));
		assert_eq!(message.header.poss_resend, Some(true));
		assert!(message.header.orig_sending_time.is_none());
	}
}

#[cfg(test)]
mod serialization_tests {
	use super::*;

	#[test]
	fn simple_heartbeat_serialization() {
		let message = FixMessage::builder(MsgType::Heartbeat, "SENDER", "TARGET", 1).build();
		let serialized = message.to_fix_string();

		// Should start with standard header
		assert!(serialized.starts_with("8=FIX.4.2\x01"));
		assert!(serialized.contains("35=0\x01")); // MsgType=Heartbeat
		assert!(serialized.contains("49=SENDER\x01"));
		assert!(serialized.contains("56=TARGET\x01"));
		assert!(serialized.contains("34=1\x01"));

		// Should end with checksum
		assert!(serialized.ends_with(&format!("10={}\x01", message.trailer.checksum)));
	}

	#[test]
	fn logon_serialization() {
		let message = FixMessage::builder(MsgType::Logon, "CLIENT", "BROKER", 1)
			.reset_seq_num_flag(true)
			.max_message_size(1024)
			.build();

		let serialized = message.to_fix_string();

		assert!(serialized.contains("35=A\x01")); // MsgType=Logon
		assert!(serialized.contains("98=0\x01")); // EncryptMethod=None
		assert!(serialized.contains("108=30\x01")); // HeartBtInt=30
		assert!(serialized.contains("141=Y\x01")); // ResetSeqNumFlag=Y
		assert!(serialized.contains("383=1024\x01")); // MaxMessageSize=1024
	}

	#[test]
	fn heartbeat_with_test_req_id_serialization() {
		let message =
			FixMessage::builder(MsgType::Heartbeat, "SENDER", "TARGET", 5).test_req_id("TEST_REQ_123").build();

		let serialized = message.to_fix_string();

		assert!(serialized.contains("35=0\x01")); // MsgType=Heartbeat
		assert!(serialized.contains("112=TEST_REQ_123\x01")); // TestReqID
	}

	#[test]
	fn checksum_calculation() {
		let message = FixMessage::builder(MsgType::Heartbeat, "SENDER", "TARGET", 1).build();
		let checksum = message.calculate_checksum();

		assert_eq!(checksum.len(), 3);
		assert!(checksum.chars().all(|c| c.is_ascii_digit()));
		assert_eq!(message.trailer.checksum, checksum);
	}

	#[test]
	fn body_length_calculation() {
		let message = FixMessage::builder(MsgType::Logon, "CLIENT", "BROKER", 1).reset_seq_num_flag(true).build();

		let calculated_length = message.calculate_body_length();
		assert_eq!(message.header.body_length, calculated_length);
		assert!(calculated_length > 0);

		// Body length should be accurate
		let body_content = message.serialize_body_and_trailer_without_checksum();
		assert_eq!(calculated_length as usize, body_content.len());
	}

	#[test]
	fn field_ordering() {
		let message = FixMessage::builder(MsgType::Heartbeat, "SENDER", "TARGET", 1)
			.poss_dup_flag(true)
			.test_req_id("TEST123")
			.build();

		let serialized = message.to_fix_string();
		let fields: Vec<&str> = serialized.split('\x01').filter(|s| !s.is_empty()).collect();

		// Verify proper field ordering
		assert!(fields[0].starts_with("8=")); // BeginString first
		assert!(fields[1].starts_with("9=")); // BodyLength second

		// MsgType should be early in body
		let msg_type_pos = fields.iter().position(|&f| f.starts_with("35=")).unwrap();
		assert!(msg_type_pos < 10); // Should be near the beginning
	}
}

#[cfg(test)]
mod parsing_tests {
	use super::*;

	#[test]
	fn parse_simple_heartbeat() {
		let original = FixMessage::builder(MsgType::Heartbeat, "SENDER", "TARGET", 1).build();
		let serialized = original.to_fix_string();
		let parsed = FixMessage::from_fix_string(&serialized).unwrap();

		assert_eq!(parsed.header.msg_type, MsgType::Heartbeat);
		assert_eq!(parsed.header.sender_comp_id, "SENDER");
		assert_eq!(parsed.header.target_comp_id, "TARGET");
		assert_eq!(parsed.header.msg_seq_num, 1);
		assert!(parsed.is_valid());
	}

	#[test]
	fn parse_logon_message() {
		let original = FixMessage::builder(MsgType::Logon, "CLIENT", "BROKER", 1).reset_seq_num_flag(true).build();

		let serialized = original.to_fix_string();
		let parsed = FixMessage::from_fix_string(&serialized).unwrap();

		assert_eq!(parsed.header.msg_type, MsgType::Logon);
		if let FixMessageBody::Logon(body) = &parsed.body {
			assert_eq!(body.encrypt_method, EncryptMethod::None);
			assert_eq!(body.heart_bt_int, 30);
			assert_eq!(body.reset_seq_num_flag, Some(true));
		} else {
			panic!("Expected Logon body");
		}
	}

	#[test]
	fn parse_heartbeat_with_test_req_id() {
		let original =
			FixMessage::builder(MsgType::Heartbeat, "SENDER", "TARGET", 5).test_req_id("TEST_REQ_123").build();

		let serialized = original.to_fix_string();
		let parsed = FixMessage::from_fix_string(&serialized).unwrap();

		if let FixMessageBody::Heartbeat(body) = &parsed.body {
			assert_eq!(body.test_req_id, Some("TEST_REQ_123".to_string()));
		} else {
			panic!("Expected Heartbeat body");
		}
	}

	#[test]
	fn parse_empty_message() {
		let result = FixMessage::from_fix_string("");
		assert!(result.is_err());
	}

	#[test]
	fn parse_malformed_field() {
		let malformed = "8=FIX.4.2\x019=50\x0135=0\x0149=SENDER\x0156=TARGET\x0134=1\x0152=20241201-12:00:00.000\x01invalid_field\x0110=123\x01";
		let result = FixMessage::from_fix_string(malformed);
		// Should still parse successfully, ignoring malformed fields
		assert!(result.is_ok());
	}

	#[test]
	fn round_trip_serialization() {
		let messages = vec![
			FixMessage::builder(MsgType::Heartbeat, "CLIENT", "BROKER", 1).build(),
			FixMessage::builder(MsgType::Heartbeat, "SENDER", "TARGET", 5).test_req_id("TEST123").build(),
			FixMessage::builder(MsgType::Logon, "TRADER", "EXCHANGE", 1)
				.reset_seq_num_flag(true)
				.max_message_size(2048)
				.build(),
		];

		for original in messages {
			let serialized = original.to_fix_string();
			let parsed = FixMessage::from_fix_string(&serialized).unwrap();

			assert_eq!(original.header.msg_type, parsed.header.msg_type);
			assert_eq!(original.header.sender_comp_id, parsed.header.sender_comp_id);
			assert_eq!(original.header.target_comp_id, parsed.header.target_comp_id);
			assert_eq!(original.header.msg_seq_num, parsed.header.msg_seq_num);
			assert!(parsed.is_valid());
		}
	}
}

#[cfg(test)]
mod real_world_examples {
	use super::*;

	#[test]
	fn session_initiation_workflow() {
		// Client sends logon
		let logon = FixMessage::builder(MsgType::Logon, "TRADER_001", "EXCHANGE_SYS", 1)
			.reset_seq_num_flag(true)
			.next_expected_msg_seq_num(1)
			.max_message_size(4096)
			.build();

		assert!(logon.is_valid());

		// Server responds with logon
		let response = FixMessage::builder(MsgType::Logon, "EXCHANGE_SYS", "TRADER_001", 1).build();

		assert!(response.is_valid());
		assert_eq!(response.header.sender_comp_id, "EXCHANGE_SYS");
		assert_eq!(response.header.target_comp_id, "TRADER_001");
	}

	#[test]
	fn heartbeat_exchange() {
		// Regular heartbeat
		let heartbeat = FixMessage::builder(MsgType::Heartbeat, "CLIENT", "SERVER", 10).build();
		assert!(heartbeat.is_valid());

		// Heartbeat in response to test request
		let test_response =
			FixMessage::builder(MsgType::Heartbeat, "CLIENT", "SERVER", 11).test_req_id("TEST_REQ_001").build();

		if let FixMessageBody::Heartbeat(body) = &test_response.body {
			assert_eq!(body.test_req_id, Some("TEST_REQ_001".to_string()));
		}
	}

	#[test]
	fn message_validation_scenarios() {
		// Valid messages
		let valid_heartbeat = FixMessage::builder(MsgType::Heartbeat, "SENDER", "TARGET", 1).build();
		assert!(valid_heartbeat.is_valid());

		let valid_logon = FixMessage::builder(MsgType::Logon, "CLIENT", "BROKER", 1).build();
		assert!(valid_logon.is_valid());

		// Invalid messages
		let invalid_sender = FixMessage::builder(MsgType::Heartbeat, "", "TARGET", 1).build();
		assert!(!invalid_sender.is_valid());

		let invalid_target = FixMessage::builder(MsgType::Heartbeat, "SENDER", "", 1).build();
		assert!(!invalid_target.is_valid());
	}

	#[test]
	fn performance_validation() {
		// Test that message creation and validation is efficient
		let start = std::time::Instant::now();

		for i in 1..=1000 {
			let msg = FixMessage::builder(MsgType::Heartbeat, "PERF_TEST", "TARGET", i).build();
			assert!(msg.is_valid());
		}

		let duration = start.elapsed();
		println!("Created and validated 1000 heartbeat messages in {:?}", duration);

		// Should be very fast for simple messages
		assert!(duration.as_millis() < 100);
	}

	#[test]
	fn memory_efficiency_test() {
		// Test that the enum body approach saves memory
		let heartbeat = FixMessage::builder(MsgType::Heartbeat, "SENDER", "TARGET", 1).build();
		let logon = FixMessage::builder(MsgType::Logon, "CLIENT", "BROKER", 1).build();

		// Verify that heartbeat doesn't have logon fields and vice versa
		match &heartbeat.body {
			FixMessageBody::Heartbeat(_) => {}, // Expected
			_ => panic!("Heartbeat should have Heartbeat body"),
		}

		match &logon.body {
			FixMessageBody::Logon(_) => {}, // Expected
			_ => panic!("Logon should have Logon body"),
		}

		// Both should be valid despite having different body structures
		assert!(heartbeat.is_valid());
		assert!(logon.is_valid());
	}
}

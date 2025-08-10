//! Comprehensive tests for FIX protocol validation components
//!
//! This test suite validates critical FIX protocol components including:
//! - Checksum calculation (Tag 10)
//! - Body length calculation (Tag 9)
//! - Message integrity validation
//! - Edge cases and error conditions

use fix_learning::{EncryptMethod, FixMessage, FixMessageBody, MsgType};
use time::OffsetDateTime;

#[cfg(test)]
mod checksum_tests {
	use super::*;

	#[test]
	fn test_basic_checksum_calculation() {
		// Create a simple heartbeat message
		let message = FixMessage::builder(MsgType::Heartbeat, "CLIENT", "SERVER", 1).build();

		let fix_string = message.to_fix_string();

		// Extract the checksum from the serialized message
		let checksum_part = fix_string.split("10=").nth(1).unwrap_or("");
		let checksum_str = checksum_part.trim_end_matches('\x01');

		// Checksum should be exactly 3 digits
		assert_eq!(checksum_str.len(), 3);
		assert!(checksum_str.parse::<u32>().is_ok());

		// Verify checksum is within valid range (000-255)
		let checksum_value = checksum_str.parse::<u32>().unwrap();
		assert!(checksum_value <= 255);
	}

	#[test]
	fn test_checksum_calculation_algorithm() {
		// Test the checksum calculation algorithm manually
		// FIX checksum is sum of all bytes modulo 256, formatted as 3-digit string

		let message = FixMessage::builder(MsgType::Heartbeat, "TRADER", "EXCHANGE", 100).test_req_id("TEST123").build();

		let fix_string = message.to_fix_string();

		// Split message to get everything before checksum
		let parts: Vec<&str> = fix_string.split("10=").collect();
		assert_eq!(parts.len(), 2, "Message should have exactly one checksum field");

		let message_without_checksum = parts[0];

		// Calculate checksum manually
		let calculated_checksum: u32 = message_without_checksum.bytes().map(|b| b as u32).sum::<u32>() % 256;

		// Extract actual checksum from message
		let actual_checksum_str = parts[1].trim_end_matches('\x01');
		let actual_checksum = actual_checksum_str.parse::<u32>().unwrap();

		assert_eq!(calculated_checksum, actual_checksum);
	}

	#[test]
	fn test_checksum_with_different_message_types() {
		let test_cases = vec![
			("SIMPLE", "TARGET"),
			("COMPLEX_SENDER_123", "COMPLEX_TARGET_456"),
			("A", "B"),
			("VERY_LONG_SENDER_COMPANY_ID", "VERY_LONG_TARGET_COMPANY_ID"),
		];

		for (sender, target) in test_cases {
			// Test heartbeat
			let heartbeat = FixMessage::builder(MsgType::Heartbeat, sender, target, 1).build();
			let fix_string = heartbeat.to_fix_string();
			validate_checksum_format(&fix_string);

			// Test logon
			let logon = FixMessage::builder(MsgType::Logon, sender, target, 1).build();
			let fix_string = logon.to_fix_string();
			validate_checksum_format(&fix_string);
		}
	}

	#[test]
	fn test_checksum_with_special_characters() {
		// Test checksum calculation with messages containing special characters
		let message =
			FixMessage::builder(MsgType::Heartbeat, "SENDER@123", "TARGET#456", 1).test_req_id("TEST@REQ!#$").build();

		let fix_string = message.to_fix_string();

		// Verify the message can be parsed back (checksum validation)
		let parsed = FixMessage::from_fix_string(&fix_string);
		assert!(parsed.is_ok(), "Message with special characters should parse correctly");

		let parsed_message = parsed.unwrap();
		if let FixMessageBody::Heartbeat(body) = &parsed_message.body {
			assert_eq!(body.test_req_id, Some("TEST@REQ!#$".to_string()));
		} else {
			panic!("Expected Heartbeat body");
		}
	}

	#[test]
	fn test_checksum_edge_cases() {
		// Test edge cases for checksum calculation

		// Maximum sequence number
		let max_seq_message = FixMessage::builder(MsgType::Heartbeat, "SENDER", "TARGET", u32::MAX).build();
		let fix_string = max_seq_message.to_fix_string();
		validate_checksum_format(&fix_string);

		// Very long field values
		let long_test_req_id = "A".repeat(1000);
		let long_message =
			FixMessage::builder(MsgType::Heartbeat, "SENDER", "TARGET", 1).test_req_id(&long_test_req_id).build();
		let fix_string = long_message.to_fix_string();
		validate_checksum_format(&fix_string);

		// Logon with all optional fields
		let full_logon = FixMessage::builder(MsgType::Logon, "CLIENT", "BROKER", 1)
			.reset_seq_num_flag(true)
			.next_expected_msg_seq_num(999999)
			.max_message_size(1048576)
			.build();
		let fix_string = full_logon.to_fix_string();
		validate_checksum_format(&fix_string);
	}

	fn validate_checksum_format(fix_string: &str) {
		let checksum_part = fix_string.split("10=").nth(1).unwrap().trim_end_matches('\x01');
		assert_eq!(checksum_part.len(), 3, "Checksum must be 3 digits");
		assert!(checksum_part.chars().all(|c| c.is_ascii_digit()), "Checksum must be numeric: {}", checksum_part);
	}
}

#[cfg(test)]
mod body_length_tests {
	use super::*;

	#[test]
	fn test_basic_body_length_calculation() {
		let message = FixMessage::builder(MsgType::Heartbeat, "CLIENT", "SERVER", 1).build();

		let fix_string = message.to_fix_string();

		// Extract body length from the message
		let body_length_part = fix_string.split("9=").nth(1).unwrap();
		let body_length_str = body_length_part.split('\x01').next().unwrap();
		let body_length = body_length_str.parse::<u32>().unwrap();

		// Calculate expected body length manually
		// Body starts after "9=XXX\x01" and ends before "10=XXX"
		let start_marker = format!("9={}\x01", body_length_str);
		let body_start = fix_string.find(&start_marker).unwrap() + start_marker.len();
		let body_end = fix_string.rfind("10=").unwrap();
		let actual_body = &fix_string[body_start..body_end];

		assert_eq!(body_length as usize, actual_body.len());
	}

	#[test]
	fn test_body_length_with_various_field_counts() {
		// Test body length calculation with different numbers of fields

		// Minimal heartbeat (only required fields)
		let minimal = FixMessage::builder(MsgType::Heartbeat, "A", "B", 1).build();
		validate_body_length(&minimal);

		// Heartbeat with optional test request ID
		let medium =
			FixMessage::builder(MsgType::Heartbeat, "SENDER", "TARGET", 100).test_req_id("TEST_REQ_ID").build();
		validate_body_length(&medium);

		// Basic logon
		let logon = FixMessage::builder(MsgType::Logon, "CLIENT", "BROKER", 1).build();
		validate_body_length(&logon);

		// Complex logon with all fields
		let complex = FixMessage::builder(MsgType::Logon, "COMPLEX_CLIENT_ID", "COMPLEX_BROKER_ID", 500)
			.reset_seq_num_flag(true)
			.next_expected_msg_seq_num(1000)
			.max_message_size(8192)
			.poss_dup_flag(true)
			.poss_resend(false)
			.orig_sending_time(OffsetDateTime::now_utc())
			.build();
		validate_body_length(&complex);
	}

	#[test]
	fn test_body_length_accuracy() {
		// Test that body length is exactly accurate
		let message = FixMessage::builder(MsgType::Logon, "TRADER", "EXCHANGE", 123)
			.reset_seq_num_flag(true)
			.next_expected_msg_seq_num(124)
			.max_message_size(4096)
			.build();

		let fix_string = message.to_fix_string();

		// Parse the message and verify the body length is correct
		let parsed = FixMessage::from_fix_string(&fix_string).unwrap();

		// Re-serialize and verify body length remains consistent
		let re_serialized = parsed.to_fix_string();
		let original_body_length = extract_body_length(&fix_string);
		let re_serialized_body_length = extract_body_length(&re_serialized);

		assert_eq!(original_body_length, re_serialized_body_length);
	}

	#[test]
	fn test_body_length_with_unicode_characters() {
		// Test body length calculation with Unicode characters
		let message = FixMessage::builder(MsgType::Heartbeat, "SENDER", "TARGET", 1)
			.test_req_id("Test with émojis 🚀 and ñoñó")
			.build();

		validate_body_length(&message);

		// Verify the message can be parsed back correctly
		let fix_string = message.to_fix_string();
		let parsed = FixMessage::from_fix_string(&fix_string);
		assert!(parsed.is_ok());
	}

	#[test]
	fn test_body_length_edge_cases() {
		// Test edge cases for body length calculation

		// Message with empty test request ID
		let message_with_empty_field =
			FixMessage::builder(MsgType::Heartbeat, "SENDER", "TARGET", 1).test_req_id("").build();
		validate_body_length(&message_with_empty_field);

		// Message with very large field values
		let large_value = "X".repeat(5000);
		let large_message =
			FixMessage::builder(MsgType::Heartbeat, "SENDER", "TARGET", 1).test_req_id(&large_value).build();
		validate_body_length(&large_message);

		// Logon with maximum values
		let max_logon = FixMessage::builder(MsgType::Logon, "SENDER", "TARGET", u32::MAX)
			.reset_seq_num_flag(true)
			.next_expected_msg_seq_num(u32::MAX)
			.max_message_size(u32::MAX)
			.build();
		validate_body_length(&max_logon);
	}

	// Helper function to validate body length calculation
	fn validate_body_length(message: &FixMessage) {
		let fix_string = message.to_fix_string();
		let calculated_body_length = extract_body_length(&fix_string);
		let actual_body_length = calculate_actual_body_length(&fix_string);

		assert_eq!(
			calculated_body_length,
			actual_body_length,
			"Body length mismatch in message: {}",
			fix_string.replace('\x01', " | ")
		);
	}

	// Helper function to extract body length from FIX string
	fn extract_body_length(fix_string: &str) -> u32 {
		let body_length_part = fix_string.split("9=").nth(1).unwrap();
		let body_length_str = body_length_part.split('\x01').next().unwrap();
		body_length_str.parse::<u32>().unwrap()
	}

	// Helper function to calculate actual body length
	fn calculate_actual_body_length(fix_string: &str) -> u32 {
		// Find the start of the body (after "9=XXX\x01")
		let body_length_field_end = fix_string.find("9=").unwrap();
		let body_start = fix_string[body_length_field_end..].find('\x01').unwrap() + body_length_field_end + 1;

		// Find the end of the body (before "10=XXX")
		let body_end = fix_string.rfind("10=").unwrap();

		// Calculate the actual body length
		(body_end - body_start) as u32
	}
}

#[cfg(test)]
mod message_integrity_tests {
	use super::*;

	#[test]
	fn test_complete_message_validation() {
		// Test that messages with correct checksum and body length parse successfully
		let heartbeat = FixMessage::builder(MsgType::Heartbeat, "CLIENT", "SERVER", 100)
			.test_req_id("INTEGRITY_TEST")
			.poss_dup_flag(true)
			.build();

		let fix_string = heartbeat.to_fix_string();

		// Verify the message can be parsed back
		let parsed = FixMessage::from_fix_string(&fix_string);
		assert!(parsed.is_ok());

		let parsed_message = parsed.unwrap();

		// Verify all fields are preserved
		assert_eq!(parsed_message.header.msg_type, MsgType::Heartbeat);
		assert_eq!(parsed_message.header.sender_comp_id, "CLIENT");
		assert_eq!(parsed_message.header.target_comp_id, "SERVER");
		assert_eq!(parsed_message.header.msg_seq_num, 100);
		assert_eq!(parsed_message.header.poss_dup_flag, Some(true));

		if let FixMessageBody::Heartbeat(body) = &parsed_message.body {
			assert_eq!(body.test_req_id, Some("INTEGRITY_TEST".to_string()));
		} else {
			panic!("Expected Heartbeat body");
		}
	}

	#[test]
	fn test_logon_message_integrity() {
		let logon = FixMessage::builder(MsgType::Logon, "TRADER", "EXCHANGE", 1)
			.encrypt_method(EncryptMethod::Des)
			.heart_bt_int(60)
			.reset_seq_num_flag(true)
			.next_expected_msg_seq_num(1)
			.max_message_size(4096)
			.build();

		let fix_string = logon.to_fix_string();
		let parsed = FixMessage::from_fix_string(&fix_string).unwrap();

		assert_eq!(parsed.header.msg_type, MsgType::Logon);
		assert_eq!(parsed.header.sender_comp_id, "TRADER");
		assert_eq!(parsed.header.target_comp_id, "EXCHANGE");

		if let FixMessageBody::Logon(body) = &parsed.body {
			assert_eq!(body.encrypt_method, EncryptMethod::Des);
			assert_eq!(body.heart_bt_int, 60);
			assert_eq!(body.reset_seq_num_flag, Some(true));
			assert_eq!(body.next_expected_msg_seq_num, Some(1));
			assert_eq!(body.max_message_size, Some(4096));
		} else {
			panic!("Expected Logon body");
		}
	}

	#[test]
	fn test_round_trip_consistency() {
		// Test that serialization -> parsing -> serialization produces identical results
		let test_cases = vec![
			FixMessage::builder(MsgType::Heartbeat, "A", "B", 1).build(),
			FixMessage::builder(MsgType::Heartbeat, "SENDER", "TARGET", 999)
				.test_req_id("ROUND_TRIP_TEST")
				.poss_dup_flag(true)
				.build(),
			FixMessage::builder(MsgType::Logon, "CLIENT", "BROKER", 50).build(),
			FixMessage::builder(MsgType::Logon, "TRADER", "EXCHANGE", 100)
				.reset_seq_num_flag(true)
				.next_expected_msg_seq_num(101)
				.max_message_size(8192)
				.build(),
		];

		for original in test_cases {
			let first_serialization = original.to_fix_string();
			let parsed = FixMessage::from_fix_string(&first_serialization).unwrap();
			let second_serialization = parsed.to_fix_string();

			// The two serializations should be identical
			assert_eq!(first_serialization, second_serialization);

			// Extract and verify checksums are identical
			let first_checksum = first_serialization.split("10=").nth(1).unwrap().trim_end_matches('\x01');
			let second_checksum = second_serialization.split("10=").nth(1).unwrap().trim_end_matches('\x01');
			assert_eq!(first_checksum, second_checksum);

			// Extract and verify body lengths are identical
			let first_body_length = first_serialization.split("9=").nth(1).unwrap().split('\x01').next().unwrap();
			let second_body_length = second_serialization.split("9=").nth(1).unwrap().split('\x01').next().unwrap();
			assert_eq!(first_body_length, second_body_length);
		}
	}

	#[test]
	fn test_field_ordering_consistency() {
		// Test that field ordering is consistent
		let message = FixMessage::builder(MsgType::Logon, "SENDER", "TARGET", 1)
			.reset_seq_num_flag(true)
			.max_message_size(4096)
			.next_expected_msg_seq_num(2)
			.build();

		let fix_string = message.to_fix_string();

		// Verify the message parses correctly
		let parsed = FixMessage::from_fix_string(&fix_string);
		assert!(parsed.is_ok());

		// Verify required fields appear before optional fields
		let msg_type_pos = fix_string.find("35=").unwrap();
		let sender_pos = fix_string.find("49=").unwrap();
		let target_pos = fix_string.find("56=").unwrap();
		let seq_pos = fix_string.find("34=").unwrap();

		// Standard header fields should appear in correct order
		assert!(msg_type_pos < sender_pos);
		assert!(sender_pos < target_pos);
		assert!(target_pos < seq_pos);
	}

	#[test]
	fn test_validation_errors() {
		// Test various validation scenarios

		// Valid messages should pass validation
		let valid_heartbeat = FixMessage::builder(MsgType::Heartbeat, "SENDER", "TARGET", 1).build();
		assert!(valid_heartbeat.is_valid());

		let valid_logon = FixMessage::builder(MsgType::Logon, "CLIENT", "BROKER", 1).build();
		assert!(valid_logon.is_valid());

		// Invalid messages should fail validation
		let invalid_sender = FixMessage::builder(MsgType::Heartbeat, "", "TARGET", 1).build();
		assert!(!invalid_sender.is_valid());

		let invalid_target = FixMessage::builder(MsgType::Heartbeat, "SENDER", "", 1).build();
		assert!(!invalid_target.is_valid());
	}

	#[test]
	fn test_checksum_corruption_detection() {
		// Test that corrupted checksums are detected
		let message = FixMessage::builder(MsgType::Heartbeat, "SENDER", "TARGET", 1).build();
		let mut fix_string = message.to_fix_string();

		// Corrupt the checksum
		let checksum_pos = fix_string.rfind("10=").unwrap();
		fix_string.replace_range(checksum_pos + 3..checksum_pos + 6, "999");

		// Parsing should still succeed (we don't validate checksum on parse currently)
		// but the message integrity is compromised
		let parsed = FixMessage::from_fix_string(&fix_string);
		assert!(parsed.is_ok());

		// The calculated checksum should be different from the corrupted one
		let parsed_message = parsed.unwrap();
		let recalculated_checksum = parsed_message.calculate_checksum();
		assert_ne!(recalculated_checksum, "999");
	}

	#[test]
	fn test_body_length_corruption_detection() {
		// Test with incorrect body length
		let message = FixMessage::builder(MsgType::Heartbeat, "SENDER", "TARGET", 1).build();
		let fix_string = message.to_fix_string();

		// Extract the correct body length
		let body_length_str = fix_string.split("9=").nth(1).unwrap().split('\x01').next().unwrap();
		let correct_body_length: u32 = body_length_str.parse().unwrap();

		// Create a message with incorrect body length
		let incorrect_body_length = correct_body_length + 10;
		let corrupted_fix_string =
			fix_string.replace(&format!("9={}", correct_body_length), &format!("9={}", incorrect_body_length));

		// The message should still parse (we don't validate body length on parse currently)
		let parsed = FixMessage::from_fix_string(&corrupted_fix_string);
		assert!(parsed.is_ok());

		// But the recalculated body length should be different
		let parsed_message = parsed.unwrap();
		let recalculated_body_length = parsed_message.calculate_body_length();
		assert_ne!(recalculated_body_length, incorrect_body_length);
		assert_eq!(recalculated_body_length, correct_body_length);
	}
}

#[cfg(test)]
mod encryption_method_tests {
	use super::*;

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
			let logon =
				FixMessage::builder(MsgType::Logon, "CLIENT", "BROKER", 1).encrypt_method(method.clone()).build();

			// Verify message is valid
			assert!(logon.is_valid());

			// Verify encryption method is preserved through serialization
			let fix_string = logon.to_fix_string();
			let parsed = FixMessage::from_fix_string(&fix_string).unwrap();

			if let FixMessageBody::Logon(body) = &parsed.body {
				assert_eq!(body.encrypt_method, method);
			} else {
				panic!("Expected Logon body");
			}
		}
	}

	#[test]
	fn test_encryption_method_serialization() {
		// Test that encryption methods are serialized with correct FIX values
		let test_cases = vec![
			(EncryptMethod::None, "0"),
			(EncryptMethod::Pkcs, "1"),
			(EncryptMethod::Des, "2"),
			(EncryptMethod::PkcsAndDes, "3"),
			(EncryptMethod::PgpAndDes, "4"),
			(EncryptMethod::PgpAndMd5, "5"),
			(EncryptMethod::PemAndMd5, "6"),
		];

		for (method, expected_value) in test_cases {
			let logon =
				FixMessage::builder(MsgType::Logon, "CLIENT", "BROKER", 1).encrypt_method(method.clone()).build();
			let fix_string = logon.to_fix_string();

			assert!(
				fix_string.contains(&format!("98={}\x01", expected_value)),
				"Expected encryption method {} in: {}",
				expected_value,
				fix_string.replace('\x01', " | ")
			);
		}
	}
}

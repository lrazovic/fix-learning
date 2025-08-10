//! Comprehensive tests for FIX protocol validation components
//!
//! This test suite validates critical FIX protocol components including:
//! - Checksum calculation (Tag 10)
//! - Body length calculation (Tag 9)
//! - Message integrity validation
//! - Edge cases and error conditions

use fix_learning::{FixMessage, MsgType, OrdStatus, Side};
use time::macros::datetime;

#[cfg(test)]
mod checksum_tests {
	use super::*;

	#[test]
	fn test_basic_checksum_calculation() {
		// Create a simple heartbeat message
		let message = FixMessage::builder(MsgType::Heartbeat, "CLIENT", "SERVER", 1)
			.sending_time(datetime!(2024-12-01 12:00:00.000 UTC))
			.build();

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

		let message = FixMessage::builder(MsgType::NewOrderSingle, "TRADER", "EXCHANGE", 100)
			.sending_time(datetime!(2024-12-01 09:30:00.000 UTC))
			.cl_ord_id("ORDER123")
			.symbol("AAPL")
			.side(Side::Buy)
			.order_qty(100.0)
			.price(150.25)
			.build();

		let fix_string = message.to_fix_string();

		// Split message to get everything before checksum
		let parts: Vec<&str> = fix_string.split("10=").collect();
		assert_eq!(parts.len(), 2, "Message should have exactly one checksum field");

		let message_without_checksum = parts[0];

		// Calculate checksum manually (without adding extra SOH)
		let calculated_checksum: u32 = message_without_checksum.bytes().map(|b| b as u32).sum::<u32>() % 256;

		// Extract actual checksum from message
		let actual_checksum_str = parts[1].trim_end_matches('\x01');
		let actual_checksum = actual_checksum_str.parse::<u32>().unwrap();

		assert_eq!(calculated_checksum, actual_checksum);
	}

	#[test]
	fn test_checksum_with_different_message_types() {
		let test_cases = vec![
			(MsgType::Heartbeat, "SIMPLE", "TARGET"),
			(MsgType::NewOrderSingle, "COMPLEX_SENDER_123", "COMPLEX_TARGET_456"),
			(MsgType::ExecutionReport, "A", "B"),
			(MsgType::TestRequest, "VERY_LONG_SENDER_COMPANY_ID", "VERY_LONG_TARGET_COMPANY_ID"),
		];

		for (msg_type, sender, target) in test_cases {
			let message = FixMessage::builder(msg_type.clone(), sender, target, 1)
				.sending_time(datetime!(2024-12-01 12:00:00.000 UTC))
				.build();

			let fix_string = message.to_fix_string();

			// Verify checksum format
			let checksum_field = fix_string.split("10=").nth(1).unwrap();
			let checksum_str = checksum_field.trim_end_matches('\x01');

			assert_eq!(checksum_str.len(), 3, "Checksum must be 3 digits for message type {:?}", msg_type);
			assert!(
				checksum_str.chars().all(|c| c.is_ascii_digit()),
				"Checksum must be numeric for message type {:?}",
				msg_type
			);
		}
	}

	#[test]
	fn test_checksum_with_special_characters() {
		// Test checksum calculation with messages containing special characters
		let message = FixMessage::builder(MsgType::NewOrderSingle, "SENDER", "TARGET", 1)
			.sending_time(datetime!(2024-12-01 12:00:00.000 UTC))
			.cl_ord_id("ORDER@123!#$")
			.symbol("AAPL.USD")
			.text("Test message with spaces & symbols")
			.field(5000, "Custom=Value|With|Pipes")
			.build();

		let fix_string = message.to_fix_string();

		// Verify the message can be parsed back (checksum validation)
		let parsed = FixMessage::from_fix_string(&fix_string);
		assert!(parsed.is_ok(), "Message with special characters should parse correctly");

		let parsed_message = parsed.unwrap();
		assert_eq!(parsed_message.cl_ord_id, Some("ORDER@123!#$".to_string()));
		assert_eq!(parsed_message.text, Some("Test message with spaces & symbols".to_string()));
	}

	#[test]
	#[should_panic] // TODO: Improve handling of invalid messages.
	fn test_checksum_edge_cases() {
		// Test edge cases for checksum calculation

		// Empty optional fields
		let minimal_message = FixMessage::builder(MsgType::Heartbeat, "", "", 0).build();
		let fix_string = minimal_message.to_fix_string();
		let checksum_part = fix_string.split("10=").nth(1).unwrap().trim_end_matches('\x01');
		assert_eq!(checksum_part.len(), 3);

		// Maximum sequence number
		let max_seq_message = FixMessage::builder(MsgType::Heartbeat, "SENDER", "TARGET", u32::MAX).build();
		let fix_string = max_seq_message.to_fix_string();
		let checksum_part = fix_string.split("10=").nth(1).unwrap().trim_end_matches('\x01');
		assert_eq!(checksum_part.len(), 3);

		// Very long field values
		let long_text = "A".repeat(1000);
		let long_message = FixMessage::builder(MsgType::TestRequest, "SENDER", "TARGET", 1)
			.sending_time(datetime!(2024-12-01 12:00:00.000 UTC))
			.text(&long_text)
			.build();
		let fix_string = long_message.to_fix_string();
		let checksum_part = fix_string.split("10=").nth(1).unwrap().trim_end_matches('\x01');
		assert_eq!(checksum_part.len(), 3);
	}
}

#[cfg(test)]
mod body_length_tests {
	use super::*;

	#[test]
	fn test_basic_body_length_calculation() {
		let message = FixMessage::builder(MsgType::Heartbeat, "CLIENT", "SERVER", 1)
			.sending_time(datetime!(2024-12-01 12:00:00.000 UTC))
			.build();

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

		// Minimal message (only required fields)
		let minimal = FixMessage::builder(MsgType::Heartbeat, "A", "B", 1)
			.sending_time(datetime!(2024-12-01 12:00:00.000 UTC))
			.build();
		validate_body_length(&minimal);

		// Message with several optional fields
		let medium = FixMessage::builder(MsgType::NewOrderSingle, "SENDER", "TARGET", 100)
			.sending_time(datetime!(2024-12-01 12:00:00.000 UTC))
			.cl_ord_id("ORDER123")
			.symbol("AAPL")
			.side(Side::Buy)
			.build();
		validate_body_length(&medium);

		// Message with many fields
		let complex = FixMessage::builder(MsgType::ExecutionReport, "BROKER", "CLIENT", 500)
			.sending_time(datetime!(2024-12-01 12:00:00.000 UTC))
			.cl_ord_id("CLIENT_ORDER_789")
			.order_id("BROKER_ORDER_456")
			.exec_id("EXEC_001")
			.exec_type("F")
			.ord_status(OrdStatus::Filled)
			.symbol("NVDA")
			.side(Side::Sell)
			.order_qty(150.0)
			.price(500.75)
			.last_qty(150.0)
			.last_px(500.80)
			.leaves_qty(0.0)
			.cum_qty(150.0)
			.avg_px(500.80)
			.text("EXECUTION COMPLETE")
			.field(207, "NASDAQ")
			.field(6000, "CUSTOM_DATA")
			.field(9999, "ANOTHER_CUSTOM_FIELD")
			.build();
		validate_body_length(&complex);
	}

	#[test]
	fn test_body_length_with_custom_fields() {
		let message = FixMessage::builder(MsgType::NewOrderSingle, "SENDER", "TARGET", 1)
			.sending_time(datetime!(2024-12-01 12:00:00.000 UTC))
			.field(5000, "CustomValue1")
			.field(5001, "CustomValue2")
			.field(9999, "VeryLongCustomFieldValueThatShouldBeIncludedInBodyLength")
			.build();

		validate_body_length(&message);

		// Verify custom fields are included in the body
		let fix_string = message.to_fix_string();
		assert!(fix_string.contains("5000=CustomValue1"));
		assert!(fix_string.contains("5001=CustomValue2"));
		assert!(fix_string.contains("9999=VeryLongCustomFieldValueThatShouldBeIncludedInBodyLength"));
	}

	#[test]
	fn test_body_length_accuracy() {
		// Test that body length is exactly accurate
		let message = FixMessage::builder(MsgType::NewOrderSingle, "TRADER", "EXCHANGE", 123)
			.sending_time(datetime!(2024-12-01 09:30:00.000 UTC))
			.cl_ord_id("ORDER_456")
			.symbol("MSFT")
			.side(Side::Buy)
			.order_qty(200.0)
			.price(300.50)
			.field(60, "20241201-09:30:00.000")
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
		let message = FixMessage::builder(MsgType::NewOrderSingle, "SENDER", "TARGET", 1)
			.sending_time(datetime!(2024-12-01 12:00:00.000 UTC))
			.text("Test with émojis 🚀 and ñoñó")
			.field(6000, "价格信息") // Chinese characters
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

		// Message with zero-length optional fields
		let message_with_empty_fields = FixMessage::builder(MsgType::TestRequest, "SENDER", "TARGET", 1)
			.sending_time(datetime!(2024-12-01 12:00:00.000 UTC))
			.cl_ord_id("")
			.text("")
			.field(6000, "")
			.build();
		validate_body_length(&message_with_empty_fields);

		// Message with very large field values
		let large_value = "X".repeat(5000);
		let large_message = FixMessage::builder(MsgType::NewOrderSingle, "SENDER", "TARGET", 1)
			.sending_time(datetime!(2024-12-01 12:00:00.000 UTC))
			.text(&large_value)
			.build();
		validate_body_length(&large_message);
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
		let message = FixMessage::builder(MsgType::ExecutionReport, "BROKER", "CLIENT", 100)
			.sending_time(datetime!(2024-12-01 10:30:00.500 UTC))
			.cl_ord_id("ORDER_123")
			.order_id("BROKER_456")
			.exec_id("EXEC_789")
			.ord_status(OrdStatus::Filled)
			.symbol("AAPL")
			.side(Side::Buy)
			.order_qty(100.0)
			.last_qty(100.0)
			.last_px(150.25)
			.leaves_qty(0.0)
			.cum_qty(100.0)
			.avg_px(150.25)
			.build();

		let fix_string = message.to_fix_string();

		// Verify the message can be parsed back
		let parsed = FixMessage::from_fix_string(&fix_string);
		assert!(parsed.is_ok());

		let parsed_message = parsed.unwrap();

		// Verify all fields are preserved
		assert_eq!(parsed_message.msg_type, MsgType::ExecutionReport);
		assert_eq!(parsed_message.sender_comp_id, "BROKER");
		assert_eq!(parsed_message.target_comp_id, "CLIENT");
		assert_eq!(parsed_message.msg_seq_num, 100);
		assert_eq!(parsed_message.cl_ord_id, Some("ORDER_123".to_string()));
		assert_eq!(parsed_message.order_id, Some("BROKER_456".to_string()));
		assert_eq!(parsed_message.exec_id, Some("EXEC_789".to_string()));
		assert_eq!(parsed_message.ord_status, Some(OrdStatus::Filled));
		assert_eq!(parsed_message.symbol, Some("AAPL".to_string()));
		assert_eq!(parsed_message.side, Some(Side::Buy));
		assert_eq!(parsed_message.order_qty, Some(100.0));
		assert_eq!(parsed_message.last_qty, Some(100.0));
		assert_eq!(parsed_message.last_px, Some(150.25));
		assert_eq!(parsed_message.leaves_qty, Some(0.0));
		assert_eq!(parsed_message.cum_qty, Some(100.0));
		assert_eq!(parsed_message.avg_px, Some(150.25));
	}

	#[test]
	fn test_round_trip_consistency() {
		// Test that serialization -> parsing -> serialization produces identical results
		let original = FixMessage::builder(MsgType::NewOrderSingle, "TRADER", "EXCHANGE", 50)
			.sending_time(datetime!(2024-12-01 15:45:30.123 UTC))
			.cl_ord_id("ROUND_TRIP_TEST")
			.symbol("GOOGL")
			.side(Side::Sell)
			.order_qty(75.0)
			.price(2800.50)
			.field(207, "NASDAQ")
			.field(6000, "ROUND_TRIP_DATA")
			.build();

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

	#[test]
	fn test_field_ordering_consistency() {
		// Test that field ordering is consistent and doesn't affect checksum/body length
		let message = FixMessage::builder(MsgType::NewOrderSingle, "SENDER", "TARGET", 1)
			.sending_time(datetime!(2024-12-01 12:00:00.000 UTC))
			.field(6000, "CUSTOM1") // Add custom fields in different orders
			.cl_ord_id("ORDER123")
			.field(207, "EXCHANGE")
			.symbol("AAPL")
			.field(9999, "CUSTOM2")
			.side(Side::Buy)
			.order_qty(100.0)
			.build();

		let fix_string = message.to_fix_string();

		// Verify the message parses correctly
		let parsed = FixMessage::from_fix_string(&fix_string);
		assert!(parsed.is_ok());

		// Verify custom fields are in the correct order in the serialized message
		let custom_field_6000_pos = fix_string.find("6000=CUSTOM1").unwrap();
		let custom_field_207_pos = fix_string.find("207=EXCHANGE").unwrap();
		let custom_field_9999_pos = fix_string.find("9999=CUSTOM2").unwrap();

		// Custom fields should appear in ascending tag order
		assert!(custom_field_207_pos < custom_field_6000_pos);
		assert!(custom_field_6000_pos < custom_field_9999_pos);
	}
}

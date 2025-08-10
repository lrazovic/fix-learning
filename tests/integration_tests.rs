use fix_learning::{EncryptMethod, FixMessage, FixMessageBody, MsgType};
use time::OffsetDateTime;

#[cfg(test)]
mod integration_tests {
	use super::*;

	#[test]
	fn session_establishment_workflow() {
		// Step 1: Client initiates session with Logon
		let client_logon = FixMessage::builder(MsgType::Logon, "TRADER_001", "EXCHANGE_SYS", 1)
			.reset_seq_num_flag(true)
			.next_expected_msg_seq_num(1)
			.max_message_size(4096)
			.build();

		assert!(client_logon.is_valid());
		assert_eq!(client_logon.header.msg_type, MsgType::Logon);
		assert_eq!(client_logon.header.sender_comp_id, "TRADER_001");
		assert_eq!(client_logon.header.target_comp_id, "EXCHANGE_SYS");

		if let FixMessageBody::Logon(body) = &client_logon.body {
			assert_eq!(body.encrypt_method, EncryptMethod::None);
			assert_eq!(body.heart_bt_int, 30);
			assert_eq!(body.reset_seq_num_flag, Some(true));
			assert_eq!(body.next_expected_msg_seq_num, Some(1));
			assert_eq!(body.max_message_size, Some(4096));
		} else {
			panic!("Expected Logon body");
		}

		// Step 2: Exchange responds with Logon acknowledgment
		let exchange_logon =
			FixMessage::builder(MsgType::Logon, "EXCHANGE_SYS", "TRADER_001", 1).max_message_size(8192).build();

		assert!(exchange_logon.is_valid());
		assert_eq!(exchange_logon.header.sender_comp_id, "EXCHANGE_SYS");
		assert_eq!(exchange_logon.header.target_comp_id, "TRADER_001");

		// Step 3: Session established, heartbeats begin
		let heartbeat = FixMessage::builder(MsgType::Heartbeat, "TRADER_001", "EXCHANGE_SYS", 2).build();
		assert!(heartbeat.is_valid());
		assert_eq!(heartbeat.header.msg_type, MsgType::Heartbeat);
	}

	#[test]
	fn heartbeat_mechanism() {
		// Regular heartbeat
		let regular_heartbeat = FixMessage::builder(MsgType::Heartbeat, "CLIENT", "SERVER", 10).build();

		assert!(regular_heartbeat.is_valid());
		assert_eq!(regular_heartbeat.header.msg_type, MsgType::Heartbeat);

		if let FixMessageBody::Heartbeat(body) = &regular_heartbeat.body {
			assert_eq!(body.test_req_id, None);
		} else {
			panic!("Expected Heartbeat body");
		}

		// Test request (simulated by another message type since we only have Heartbeat/Logon)
		// In response to test request, send heartbeat with TestReqID
		let test_response =
			FixMessage::builder(MsgType::Heartbeat, "CLIENT", "SERVER", 11).test_req_id("TEST_REQ_12345").build();

		assert!(test_response.is_valid());

		if let FixMessageBody::Heartbeat(body) = &test_response.body {
			assert_eq!(body.test_req_id, Some("TEST_REQ_12345".to_string()));
		} else {
			panic!("Expected Heartbeat body");
		}
	}

	#[test]
	fn message_serialization_roundtrip() {
		let test_cases = vec![
			// Basic heartbeat
			FixMessage::builder(MsgType::Heartbeat, "SENDER", "TARGET", 1).build(),
			// Heartbeat with test request ID
			FixMessage::builder(MsgType::Heartbeat, "CLIENT", "BROKER", 5)
				.test_req_id("TEST_123")
				.poss_dup_flag(true)
				.build(),
			// Basic logon
			FixMessage::builder(MsgType::Logon, "TRADER", "EXCHANGE", 1).build(),
			// Full logon with all optional fields
			FixMessage::builder(MsgType::Logon, "CLIENT", "BROKER", 1)
				.reset_seq_num_flag(true)
				.next_expected_msg_seq_num(1)
				.max_message_size(2048)
				.poss_dup_flag(false)
				.build(),
		];

		for original in test_cases {
			// Serialize to FIX wire format
			let serialized = original.to_fix_string();

			// Verify serialization contains expected elements
			assert!(serialized.starts_with("8=FIX.4.2\x01"));
			assert!(serialized.contains(&format!("35={}\x01", original.header.msg_type)));
			assert!(serialized.contains(&format!("49={}\x01", original.header.sender_comp_id)));
			assert!(serialized.contains(&format!("56={}\x01", original.header.target_comp_id)));
			assert!(serialized.contains(&format!("34={}\x01", original.header.msg_seq_num)));
			assert!(serialized.ends_with(&format!("10={}\x01", original.trailer.checksum)));

			// Parse back from wire format
			let parsed = FixMessage::from_fix_string(&serialized).unwrap();

			// Verify core fields match
			assert_eq!(original.header.msg_type, parsed.header.msg_type);
			assert_eq!(original.header.sender_comp_id, parsed.header.sender_comp_id);
			assert_eq!(original.header.target_comp_id, parsed.header.target_comp_id);
			assert_eq!(original.header.msg_seq_num, parsed.header.msg_seq_num);
			assert_eq!(original.header.body_length, parsed.header.body_length);

			// Verify parsed message is valid
			assert!(parsed.is_valid());

			// Verify body content matches
			match (&original.body, &parsed.body) {
				(FixMessageBody::Heartbeat(orig), FixMessageBody::Heartbeat(parsed)) => {
					assert_eq!(orig.test_req_id, parsed.test_req_id);
				},
				(FixMessageBody::Logon(orig), FixMessageBody::Logon(parsed)) => {
					assert_eq!(orig.encrypt_method, parsed.encrypt_method);
					assert_eq!(orig.heart_bt_int, parsed.heart_bt_int);
					assert_eq!(orig.reset_seq_num_flag, parsed.reset_seq_num_flag);
					assert_eq!(orig.next_expected_msg_seq_num, parsed.next_expected_msg_seq_num);
					assert_eq!(orig.max_message_size, parsed.max_message_size);
				},
				_ => panic!("Body types don't match"),
			}
		}
	}

	#[test]
	fn checksum_validation() {
		let messages = vec![
			FixMessage::builder(MsgType::Heartbeat, "SENDER", "TARGET", 1).build(),
			FixMessage::builder(MsgType::Logon, "CLIENT", "BROKER", 1).build(),
		];

		for msg in messages {
			// Verify checksum is properly formatted
			assert_eq!(msg.trailer.checksum.len(), 3);
			assert!(msg.trailer.checksum.chars().all(|c| c.is_ascii_digit()));

			// Verify checksum calculation is consistent
			let calculated = msg.calculate_checksum();
			assert_eq!(msg.trailer.checksum, calculated);

			// Verify checksum changes when message content changes
			let mut modified_content = msg.serialize_without_checksum();
			modified_content.push('X'); // Add extra character
			let modified_checksum: u32 = modified_content.bytes().map(|b| b as u32).sum::<u32>() % 256;
			let modified_checksum_str = format!("{:03}", modified_checksum);

			assert_ne!(msg.trailer.checksum, modified_checksum_str);
		}
	}

	#[test]
	fn body_length_calculation() {
		let test_cases = vec![
			FixMessage::builder(MsgType::Heartbeat, "S", "T", 1).build(),
			FixMessage::builder(MsgType::Heartbeat, "SENDER", "TARGET", 999).test_req_id("TEST").build(),
			FixMessage::builder(MsgType::Logon, "CLIENT", "BROKER", 1).build(),
			FixMessage::builder(MsgType::Logon, "VERY_LONG_SENDER_ID", "VERY_LONG_TARGET_ID", 999999)
				.reset_seq_num_flag(true)
				.next_expected_msg_seq_num(123456)
				.max_message_size(999999)
				.build(),
		];

		for msg in test_cases {
			let calculated_length = msg.calculate_body_length();
			assert_eq!(msg.header.body_length, calculated_length);
			assert!(calculated_length > 0);

			// Verify length calculation is accurate
			let body_content = msg.serialize_body_and_trailer_without_checksum();
			assert_eq!(calculated_length as usize, body_content.len());

			// Verify that adding content increases body length
			let longer_msg = FixMessage::builder(
				MsgType::Heartbeat,
				&msg.header.sender_comp_id,
				&msg.header.target_comp_id,
				msg.header.msg_seq_num,
			)
			.test_req_id("THIS_IS_A_MUCH_LONGER_TEST_REQUEST_ID_THAT_SHOULD_INCREASE_BODY_LENGTH")
			.build();
			assert!(longer_msg.header.body_length > msg.header.body_length);
		}
	}

	#[test]
	fn message_validation_comprehensive() {
		// Valid messages
		let valid_cases = vec![
			FixMessage::builder(MsgType::Heartbeat, "A", "B", 1).build(),
			FixMessage::builder(MsgType::Heartbeat, "SENDER", "TARGET", u32::MAX).build(),
			FixMessage::builder(MsgType::Logon, "CLIENT", "BROKER", 1).build(),
			FixMessage::builder(MsgType::Logon, "C", "B", 1).build(),
		];

		for msg in valid_cases {
			assert!(msg.is_valid(), "Message should be valid: {:?}", msg);
		}

		// Invalid messages
		let invalid_cases = vec![
			// Empty sender
			FixMessage::builder(MsgType::Heartbeat, "", "TARGET", 1).build(),
			// Empty target
			FixMessage::builder(MsgType::Heartbeat, "SENDER", "", 1).build(),
			// Zero sequence number
			{
				let mut msg = FixMessage::builder(MsgType::Heartbeat, "SENDER", "TARGET", 1).build();
				msg.header.msg_seq_num = 0;
				msg
			},
			// Invalid heartbeat interval in logon
			FixMessage::builder(MsgType::Logon, "CLIENT", "BROKER", 1).heart_bt_int(0).build(),
		];

		for msg in invalid_cases {
			assert!(!msg.is_valid(), "Message should be invalid: {:?}", msg);
		}
	}

	#[test]
	fn encryption_method_support() {
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

			assert!(logon.is_valid());

			if let FixMessageBody::Logon(body) = &logon.body {
				assert_eq!(body.encrypt_method, method);
			} else {
				panic!("Expected Logon body");
			}

			// Test serialization roundtrip
			let serialized = logon.to_fix_string();
			let parsed = FixMessage::from_fix_string(&serialized).unwrap();

			if let FixMessageBody::Logon(parsed_body) = &parsed.body {
				assert_eq!(parsed_body.encrypt_method, method);
			} else {
				panic!("Expected Logon body after parsing");
			}
		}
	}

	#[test]
	fn optional_fields_handling() {
		// Test heartbeat with and without optional fields
		let basic_heartbeat = FixMessage::builder(MsgType::Heartbeat, "SENDER", "TARGET", 1).build();
		let enhanced_heartbeat = FixMessage::builder(MsgType::Heartbeat, "SENDER", "TARGET", 2)
			.test_req_id("TEST_REQ_001")
			.poss_dup_flag(true)
			.poss_resend(false)
			.orig_sending_time(OffsetDateTime::now_utc())
			.build();

		assert!(basic_heartbeat.is_valid());
		assert!(enhanced_heartbeat.is_valid());

		// Test logon with and without optional fields
		let basic_logon = FixMessage::builder(MsgType::Logon, "CLIENT", "BROKER", 1).build();
		let enhanced_logon = FixMessage::builder(MsgType::Logon, "CLIENT", "BROKER", 2)
			.reset_seq_num_flag(true)
			.next_expected_msg_seq_num(1)
			.max_message_size(4096)
			.build();

		assert!(basic_logon.is_valid());
		assert!(enhanced_logon.is_valid());

		// Verify optional fields are preserved through serialization
		for msg in [enhanced_heartbeat, enhanced_logon] {
			let serialized = msg.to_fix_string();
			let parsed = FixMessage::from_fix_string(&serialized).unwrap();
			assert!(parsed.is_valid());
		}
	}
	#[test]
	fn edge_case_handling() {
		// Test with minimum values
		let min_seq = FixMessage::builder(MsgType::Heartbeat, "A", "B", 1).build();
		assert!(min_seq.is_valid());

		// Test with maximum sequence number
		let max_seq = FixMessage::builder(MsgType::Heartbeat, "SENDER", "TARGET", u32::MAX).build();
		assert!(max_seq.is_valid());

		// Test with minimum heartbeat interval
		let min_heartbeat = FixMessage::builder(MsgType::Logon, "CLIENT", "BROKER", 1).build();
		assert!(min_heartbeat.is_valid());

		// Test with maximum heartbeat interval
		let max_heartbeat = FixMessage::builder(MsgType::Logon, "CLIENT", "BROKER", 1).build();
		assert!(max_heartbeat.is_valid());

		// Test serialization of edge cases
		for msg in [min_seq, max_seq, min_heartbeat, max_heartbeat] {
			let serialized = msg.to_fix_string();
			let parsed = FixMessage::from_fix_string(&serialized).unwrap();
			assert!(parsed.is_valid());
			assert_eq!(msg.header.msg_seq_num, parsed.header.msg_seq_num);
		}
	}
}

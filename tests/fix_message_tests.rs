use fix_learning::{EncryptMethod, FixMessage, FixMessageBody, MsgType, OrdStatus, Side};
use std::str::FromStr;

#[cfg(test)]
mod msg_type_tests {
	use super::*;

	#[test]
	fn msg_type_from_str_valid_values() {
		assert_eq!(MsgType::from_str("0").unwrap(), MsgType::Heartbeat);
		assert_eq!(MsgType::from_str("A").unwrap(), MsgType::Logon);
	}

	#[test]
	fn msg_type_from_str_unknown_value() {
		match MsgType::from_str("Z").unwrap() {
			MsgType::Other(s) => assert_eq!(s, "Z"),
			_ => panic!("Expected Other variant"),
		}
	}

	#[test]
	fn msg_type_display() {
		assert_eq!(format!("{}", MsgType::Heartbeat), "0");
		assert_eq!(format!("{}", MsgType::Other("Z".to_string())), "Z");
		assert_eq!(format!("{}", MsgType::Logon), "A");
	}

	#[test]
	fn msg_type_round_trip() {
		let original_types = vec![MsgType::Heartbeat, MsgType::Logon, MsgType::Other("CUSTOM".to_string())];

		for msg_type in original_types {
			let str_repr = format!("{}", msg_type);
			let parsed = MsgType::from_str(&str_repr).unwrap();
			assert_eq!(msg_type, parsed);
		}
	}
}

#[cfg(test)]
mod side_tests {
	use super::*;

	#[test]
	fn side_from_str_valid() {
		assert_eq!(Side::from_str("1").unwrap(), Side::Buy);
		assert_eq!(Side::from_str("2").unwrap(), Side::Sell);
	}

	#[test]
	fn side_from_str_invalid() {
		assert!(Side::from_str("0").is_err());
		assert!(Side::from_str("3").is_err());
		assert!(Side::from_str("B").is_err());
		assert!(Side::from_str("").is_err());
	}

	#[test]
	fn side_display() {
		assert_eq!(format!("{}", Side::Buy), "1");
		assert_eq!(format!("{}", Side::Sell), "2");
	}

	#[test]
	fn side_round_trip() {
		let sides = vec![Side::Buy, Side::Sell];
		for side in sides {
			let str_repr = format!("{}", side);
			let parsed = Side::from_str(&str_repr).unwrap();
			assert_eq!(side, parsed);
		}
	}
}

#[cfg(test)]
mod ord_status_tests {
	use super::*;

	#[test]
	fn ord_status_from_str_numeric() {
		assert_eq!(OrdStatus::from_str("0").unwrap(), OrdStatus::New);
		assert_eq!(OrdStatus::from_str("1").unwrap(), OrdStatus::PartiallyFilled);
		assert_eq!(OrdStatus::from_str("2").unwrap(), OrdStatus::Filled);
		assert_eq!(OrdStatus::from_str("4").unwrap(), OrdStatus::Canceled);
		assert_eq!(OrdStatus::from_str("8").unwrap(), OrdStatus::Rejected);
	}

	#[test]
	fn ord_status_from_str_alpha() {
		assert_eq!(OrdStatus::from_str("A").unwrap(), OrdStatus::PendingNew);
		assert_eq!(OrdStatus::from_str("B").unwrap(), OrdStatus::Calculated);
		assert_eq!(OrdStatus::from_str("C").unwrap(), OrdStatus::Expired);
		assert_eq!(OrdStatus::from_str("D").unwrap(), OrdStatus::AcceptedForBidding);
		assert_eq!(OrdStatus::from_str("E").unwrap(), OrdStatus::PendingReplace);
	}

	#[test]
	fn ord_status_from_str_invalid() {
		assert!(OrdStatus::from_str("F").is_err());
		assert!(OrdStatus::from_str("Z").is_err());
		assert!(OrdStatus::from_str("10").is_err());
		assert!(OrdStatus::from_str("").is_err());
	}

	#[test]
	fn ord_status_display() {
		assert_eq!(format!("{}", OrdStatus::New), "0");
		assert_eq!(format!("{}", OrdStatus::Filled), "2");
		assert_eq!(format!("{}", OrdStatus::Canceled), "4");
		assert_eq!(format!("{}", OrdStatus::PendingNew), "A");
		assert_eq!(format!("{}", OrdStatus::PendingReplace), "E");
	}

	#[test]
	fn ord_status_round_trip() {
		let statuses = vec![
			OrdStatus::New,
			OrdStatus::PartiallyFilled,
			OrdStatus::Filled,
			OrdStatus::Canceled,
			OrdStatus::PendingNew,
			OrdStatus::PendingReplace,
		];

		for status in statuses {
			let str_repr = format!("{}", status);
			let parsed = OrdStatus::from_str(&str_repr).unwrap();
			assert_eq!(status, parsed);
		}
	}
}

#[cfg(test)]
mod fix_message_tests {
	use super::*;

	#[test]
	fn new_fix_message_creation() {
		let msg = FixMessage::builder(MsgType::Heartbeat, "SENDER".to_string(), "TARGET".to_string(), 123).build();

		assert_eq!(msg.header.begin_string, "FIX.4.2");
		assert_eq!(msg.header.msg_type, MsgType::Heartbeat);
		assert_eq!(msg.header.sender_comp_id, "SENDER");
		assert_eq!(msg.header.target_comp_id, "TARGET");
		assert_eq!(msg.header.msg_seq_num, 123);
		assert!(msg.header.body_length > 0);
		assert_eq!(msg.trailer.checksum.len(), 3);
	}

	#[test]
	fn default_fix_message() {
		let msg = FixMessage::default();

		assert_eq!(msg.header.begin_string, "FIX.4.2");
		assert_eq!(msg.header.msg_type, MsgType::Heartbeat);
		assert_eq!(msg.header.sender_comp_id, "SENDER");
		assert_eq!(msg.header.target_comp_id, "TARGET");
		assert_eq!(msg.header.msg_seq_num, 1);
	}

	#[test]
	fn heartbeat_message_creation() {
		let msg = FixMessage::builder(MsgType::Heartbeat, "CLIENT", "BROKER", 1).test_req_id("TEST123").build();

		assert_eq!(msg.header.msg_type, MsgType::Heartbeat);

		if let FixMessageBody::Heartbeat(body) = &msg.body {
			assert_eq!(body.test_req_id, Some("TEST123".to_string()));
		} else {
			panic!("Expected Heartbeat body");
		}
	}

	#[test]
	fn logon_message_creation() {
		let msg = FixMessage::builder(MsgType::Logon, "CLIENT", "BROKER", 1)
			.reset_seq_num_flag(true)
			.max_message_size(1024)
			.build();

		assert_eq!(msg.header.msg_type, MsgType::Logon);

		if let FixMessageBody::Logon(body) = &msg.body {
			assert_eq!(body.encrypt_method, EncryptMethod::None);
			assert_eq!(body.heart_bt_int, 30);
			assert_eq!(body.reset_seq_num_flag, Some(true));
			assert_eq!(body.max_message_size, Some(1024));
		} else {
			panic!("Expected Logon body");
		}
	}

	#[test]
	fn message_validation() {
		// Valid message
		let valid_msg = FixMessage::builder(MsgType::Heartbeat, "SENDER", "TARGET", 1).build();
		assert!(valid_msg.is_valid());

		// Invalid message - empty sender
		let mut invalid_msg = FixMessage::builder(MsgType::Heartbeat, "", "TARGET", 1).build();
		assert!(!invalid_msg.is_valid());

		// Invalid message - empty target
		invalid_msg = FixMessage::builder(MsgType::Heartbeat, "SENDER", "", 1).build();
		assert!(!invalid_msg.is_valid());
	}

	#[test]
	fn message_equality() {
		let now = time::OffsetDateTime::now_utc();
		let msg1 = FixMessage::builder(MsgType::Heartbeat, "SENDER", "TARGET", 1).sending_time(now).build();
		let msg2 = FixMessage::builder(MsgType::Heartbeat, "SENDER", "TARGET", 1).sending_time(now).build();

		let msg3 = FixMessage::builder(
			MsgType::Other("DIFFERENT".to_string()), // Different message type
			"SENDER",
			"TARGET",
			1,
		)
		.build();

		assert_eq!(msg1.header.msg_type, msg2.header.msg_type);
		assert_eq!(msg1.header.sender_comp_id, msg2.header.sender_comp_id);
		assert_ne!(msg1.header.msg_type, msg3.header.msg_type);
	}

	#[test]
	fn message_cloning() {
		let original = FixMessage::builder(MsgType::Heartbeat, "SENDER", "TARGET", 1).build();
		let cloned = original.clone();

		assert_eq!(original.header.msg_type, cloned.header.msg_type);
		assert_eq!(original.header.sender_comp_id, cloned.header.sender_comp_id);
		assert_eq!(original.body, cloned.body);
	}
}

#[cfg(test)]
mod integration_tests {
	use super::*;

	#[test]
	fn heartbeat_workflow() {
		// Create a basic heartbeat
		let heartbeat = FixMessage::builder(MsgType::Heartbeat, "CLIENT", "BROKER", 10).build();

		assert!(heartbeat.is_valid());
		assert_eq!(heartbeat.header.msg_type, MsgType::Heartbeat);
		assert_eq!(heartbeat.header.msg_seq_num, 10);

		// Create a heartbeat in response to test request
		let test_response =
			FixMessage::builder(MsgType::Heartbeat, "CLIENT", "BROKER", 11).test_req_id("TEST_REQ_001").build();

		if let FixMessageBody::Heartbeat(body) = &test_response.body {
			assert_eq!(body.test_req_id, Some("TEST_REQ_001".to_string()));
		} else {
			panic!("Expected Heartbeat body");
		}
	}

	#[test]
	fn logon_workflow() {
		// Create a logon message
		let logon = FixMessage::builder(MsgType::Logon, "TRADER", "EXCHANGE", 1)
			.reset_seq_num_flag(true)
			.next_expected_msg_seq_num(1)
			.max_message_size(4096)
			.build();

		assert!(logon.is_valid());
		assert_eq!(logon.header.msg_type, MsgType::Logon);

		if let FixMessageBody::Logon(body) = &logon.body {
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
	fn message_serialization_and_parsing() {
		// Test heartbeat serialization
		let heartbeat = FixMessage::builder(MsgType::Heartbeat, "SENDER", "TARGET", 1).test_req_id("TEST123").build();

		let serialized = heartbeat.to_fix_string();
		let parsed = FixMessage::from_fix_string(&serialized).unwrap();

		assert_eq!(heartbeat.header.msg_type, parsed.header.msg_type);
		assert_eq!(heartbeat.header.sender_comp_id, parsed.header.sender_comp_id);
		assert_eq!(heartbeat.header.target_comp_id, parsed.header.target_comp_id);

		// Test logon serialization
		let logon = FixMessage::builder(MsgType::Logon, "CLIENT", "BROKER", 1).reset_seq_num_flag(true).build();

		let serialized = logon.to_fix_string();
		let parsed = FixMessage::from_fix_string(&serialized).unwrap();

		assert_eq!(logon.header.msg_type, parsed.header.msg_type);
		assert_eq!(logon.header.sender_comp_id, parsed.header.sender_comp_id);
	}

	#[test]
	fn checksum_validation() {
		let msg = FixMessage::builder(MsgType::Heartbeat, "SENDER", "TARGET", 1).build();
		let calculated_checksum = msg.calculate_checksum();

		assert_eq!(msg.trailer.checksum, calculated_checksum);
		assert_eq!(msg.trailer.checksum.len(), 3);
		assert!(msg.trailer.checksum.chars().all(|c| c.is_ascii_digit()));
	}

	#[test]
	fn body_length_calculation() {
		let msg = FixMessage::builder(MsgType::Logon, "CLIENT", "BROKER", 1).build();
		let calculated_length = msg.calculate_body_length();

		assert_eq!(msg.header.body_length, calculated_length);
		assert!(msg.header.body_length > 0);
	}
}

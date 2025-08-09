use fix_learning::{FixMessage, MsgType, OrdStatus, Side};
use std::str::FromStr;

#[cfg(test)]
mod msg_type_tests {
	use super::*;

	#[test]
	fn test_msg_type_from_str_valid_values() {
		assert_eq!(MsgType::from_str("0").unwrap(), MsgType::Heartbeat);
		assert_eq!(MsgType::from_str("1").unwrap(), MsgType::TestRequest);
		assert_eq!(MsgType::from_str("8").unwrap(), MsgType::ExecutionReport);
		assert_eq!(MsgType::from_str("D").unwrap(), MsgType::NewOrderSingle);
		assert_eq!(MsgType::from_str("F").unwrap(), MsgType::OrderCancelRequest);
		assert_eq!(MsgType::from_str("V").unwrap(), MsgType::MarketDataRequest);
	}

	#[test]
	fn test_msg_type_from_str_unknown_value() {
		match MsgType::from_str("Z").unwrap() {
			MsgType::Other(s) => assert_eq!(s, "Z"),
			_ => panic!("Expected Other variant"),
		}
	}

	#[test]
	fn test_msg_type_display() {
		assert_eq!(format!("{}", MsgType::Heartbeat), "0");
		assert_eq!(format!("{}", MsgType::TestRequest), "1");
		assert_eq!(format!("{}", MsgType::ExecutionReport), "8");
		assert_eq!(format!("{}", MsgType::NewOrderSingle), "D");
		assert_eq!(format!("{}", MsgType::OrderCancelRequest), "F");
		assert_eq!(format!("{}", MsgType::Other("Z".to_string())), "Z");
	}

	#[test]
	fn test_msg_type_round_trip() {
		let original_types = vec![
			MsgType::Heartbeat,
			MsgType::ExecutionReport,
			MsgType::NewOrderSingle,
			MsgType::Other("CUSTOM".to_string()),
		];

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
	fn test_side_from_str_valid() {
		assert_eq!(Side::from_str("1").unwrap(), Side::Buy);
		assert_eq!(Side::from_str("2").unwrap(), Side::Sell);
	}

	#[test]
	fn test_side_from_str_invalid() {
		assert!(Side::from_str("0").is_err());
		assert!(Side::from_str("3").is_err());
		assert!(Side::from_str("B").is_err());
		assert!(Side::from_str("").is_err());
	}

	#[test]
	fn test_side_display() {
		assert_eq!(format!("{}", Side::Buy), "1");
		assert_eq!(format!("{}", Side::Sell), "2");
	}

	#[test]
	fn test_side_round_trip() {
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
	fn test_ord_status_from_str_numeric() {
		assert_eq!(OrdStatus::from_str("0").unwrap(), OrdStatus::New);
		assert_eq!(OrdStatus::from_str("1").unwrap(), OrdStatus::PartiallyFilled);
		assert_eq!(OrdStatus::from_str("2").unwrap(), OrdStatus::Filled);
		assert_eq!(OrdStatus::from_str("4").unwrap(), OrdStatus::Canceled);
		assert_eq!(OrdStatus::from_str("8").unwrap(), OrdStatus::Rejected);
	}

	#[test]
	fn test_ord_status_from_str_alpha() {
		assert_eq!(OrdStatus::from_str("A").unwrap(), OrdStatus::PendingNew);
		assert_eq!(OrdStatus::from_str("B").unwrap(), OrdStatus::Calculated);
		assert_eq!(OrdStatus::from_str("C").unwrap(), OrdStatus::Expired);
		assert_eq!(OrdStatus::from_str("D").unwrap(), OrdStatus::AcceptedForBidding);
		assert_eq!(OrdStatus::from_str("E").unwrap(), OrdStatus::PendingReplace);
	}

	#[test]
	fn test_ord_status_from_str_invalid() {
		assert!(OrdStatus::from_str("F").is_err());
		assert!(OrdStatus::from_str("Z").is_err());
		assert!(OrdStatus::from_str("10").is_err());
		assert!(OrdStatus::from_str("").is_err());
	}

	#[test]
	fn test_ord_status_display() {
		assert_eq!(format!("{}", OrdStatus::New), "0");
		assert_eq!(format!("{}", OrdStatus::Filled), "2");
		assert_eq!(format!("{}", OrdStatus::Canceled), "4");
		assert_eq!(format!("{}", OrdStatus::PendingNew), "A");
		assert_eq!(format!("{}", OrdStatus::PendingReplace), "E");
	}

	#[test]
	fn test_ord_status_round_trip() {
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
	fn test_new_fix_message_creation() {
		let msg = FixMessage::new(
			MsgType::Heartbeat,
			"SENDER".to_string(),
			"TARGET".to_string(),
			123,
			"20241201-12:00:00.000".to_string(),
		);

		assert_eq!(msg.begin_string, "FIX.4.2");
		assert_eq!(msg.msg_type, MsgType::Heartbeat);
		assert_eq!(msg.sender_comp_id, "SENDER");
		assert_eq!(msg.target_comp_id, "TARGET");
		assert_eq!(msg.msg_seq_num, 123);
		assert_eq!(msg.sending_time, "20241201-12:00:00.000");
		assert_eq!(msg.body_length, 0);
		assert_eq!(msg.checksum, "000");
	}

	#[test]
	fn test_default_fix_message() {
		let msg = FixMessage::default();

		assert_eq!(msg.begin_string, "FIX.4.2");
		assert_eq!(msg.msg_type, MsgType::Heartbeat);
		assert_eq!(msg.sender_comp_id, "SENDER");
		assert_eq!(msg.target_comp_id, "TARGET");
		assert_eq!(msg.msg_seq_num, 1);
		assert_eq!(msg.sending_time, "20240101-00:00:00.000");
	}

	#[test]
	fn test_fix_message_optional_fields() {
		let mut msg = FixMessage::new(
			MsgType::NewOrderSingle,
			"CLIENT".to_string(),
			"BROKER".to_string(),
			1,
			"20241201-12:00:00.000".to_string(),
		);

		// Initially, optional fields should be None
		assert_eq!(msg.cl_ord_id, None);
		assert_eq!(msg.symbol, None);
		assert_eq!(msg.side, None);
		assert_eq!(msg.order_qty, None);

		// Set some optional fields
		msg.cl_ord_id = Some("CLIENT123".to_string());
		msg.symbol = Some("AAPL".to_string());
		msg.side = Some(Side::Buy);
		msg.order_qty = Some(100.0);

		assert_eq!(msg.cl_ord_id, Some("CLIENT123".to_string()));
		assert_eq!(msg.symbol, Some("AAPL".to_string()));
		assert_eq!(msg.side, Some(Side::Buy));
		assert_eq!(msg.order_qty, Some(100.0));
	}

	#[test]
	fn test_additional_fields() {
		let mut msg = FixMessage::default();

		// Test setting and getting custom fields
		msg.set_field(9999, "custom_value".to_string());
		msg.set_field(8888, "another_value".to_string());

		assert_eq!(msg.get_field(9999), Some(&"custom_value".to_string()));
		assert_eq!(msg.get_field(8888), Some(&"another_value".to_string()));
		assert_eq!(msg.get_field(7777), None);

		// Test overwriting a field
		msg.set_field(9999, "updated_value".to_string());
		assert_eq!(msg.get_field(9999), Some(&"updated_value".to_string()));
	}

	#[test]
	fn test_is_valid() {
		// Valid message
		let valid_msg = FixMessage::new(
			MsgType::Heartbeat,
			"SENDER".to_string(),
			"TARGET".to_string(),
			1,
			"20241201-12:00:00.000".to_string(),
		);
		assert!(valid_msg.is_valid());

		// Invalid message - empty sender
		let invalid_msg = FixMessage::new(
			MsgType::Heartbeat,
			"".to_string(), // Empty sender
			"TARGET".to_string(),
			1,
			"20241201-12:00:00.000".to_string(),
		);
		assert!(!invalid_msg.is_valid());

		// Invalid message - empty target
		let invalid_msg2 = FixMessage::new(
			MsgType::Heartbeat,
			"SENDER".to_string(),
			"".to_string(), // Empty target
			1,
			"20241201-12:00:00.000".to_string(),
		);
		assert!(!invalid_msg2.is_valid());

		// Invalid message - empty sending time
		let invalid_msg3 = FixMessage::new(
			MsgType::Heartbeat,
			"SENDER".to_string(),
			"TARGET".to_string(),
			1,
			"".to_string(), // Empty sending time
		);
		assert!(!invalid_msg3.is_valid());
	}

	#[test]
	fn test_message_equality() {
		let msg1 = FixMessage::new(
			MsgType::Heartbeat,
			"SENDER".to_string(),
			"TARGET".to_string(),
			1,
			"20241201-12:00:00.000".to_string(),
		);

		let msg2 = FixMessage::new(
			MsgType::Heartbeat,
			"SENDER".to_string(),
			"TARGET".to_string(),
			1,
			"20241201-12:00:00.000".to_string(),
		);

		let msg3 = FixMessage::new(
			MsgType::TestRequest, // Different message type
			"SENDER".to_string(),
			"TARGET".to_string(),
			1,
			"20241201-12:00:00.000".to_string(),
		);

		assert_eq!(msg1, msg2);
		assert_ne!(msg1, msg3);
	}

	#[test]
	fn test_message_cloning() {
		let original = FixMessage::default();
		let cloned = original.clone();

		assert_eq!(original, cloned);
		// Ensure they are separate instances
		assert_eq!(original.cl_ord_id, cloned.cl_ord_id);
		assert_eq!(original.symbol, cloned.symbol);
	}
}

#[cfg(test)]
mod integration_tests {
	use super::*;

	#[test]
	fn test_new_order_single_workflow() {
		// Create a New Order Single message
		let mut new_order = FixMessage::new(
			MsgType::NewOrderSingle,
			"CLIENT".to_string(),
			"BROKER".to_string(),
			1,
			"20241201-12:00:00.000".to_string(),
		);

		// Set order fields
		new_order.cl_ord_id = Some("ORDER123".to_string());
		new_order.symbol = Some("MSFT".to_string());
		new_order.side = Some(Side::Buy);
		new_order.order_qty = Some(100.0);
		new_order.ord_type = Some("2".to_string()); // Limit order
		new_order.price = Some(100.50);
		new_order.time_in_force = Some("0".to_string()); // Day

		assert!(new_order.is_valid());
		assert_eq!(new_order.msg_type, MsgType::NewOrderSingle);
		assert_eq!(new_order.cl_ord_id, Some("ORDER123".to_string()));
		assert_eq!(new_order.symbol, Some("MSFT".to_string()));
		assert_eq!(new_order.side, Some(Side::Buy));
	}

	#[test]
	fn test_execution_report_workflow() {
		// Create an Execution Report in response to the order
		let mut exec_report = FixMessage::new(
			MsgType::ExecutionReport,
			"BROKER".to_string(),
			"CLIENT".to_string(),
			1,
			"20241201-12:00:01.000".to_string(),
		);

		// Set execution fields
		exec_report.cl_ord_id = Some("ORDER123".to_string());
		exec_report.order_id = Some("BROKER123".to_string());
		exec_report.exec_id = Some("EXEC456".to_string());
		exec_report.exec_type = Some("0".to_string()); // New
		exec_report.ord_status = Some(OrdStatus::New);
		exec_report.symbol = Some("MSFT".to_string());
		exec_report.side = Some(Side::Buy);
		exec_report.order_qty = Some(100.0);
		exec_report.leaves_qty = Some(100.0);
		exec_report.cum_qty = Some(0.0);

		assert!(exec_report.is_valid());
		assert_eq!(exec_report.msg_type, MsgType::ExecutionReport);
		assert_eq!(exec_report.ord_status, Some(OrdStatus::New));
	}

	#[test]
	fn test_fill_execution_report() {
		// Create a fill execution report
		let mut fill_report = FixMessage::new(
			MsgType::ExecutionReport,
			"BROKER".to_string(),
			"CLIENT".to_string(),
			2,
			"20241201-12:00:02.000".to_string(),
		);

		fill_report.cl_ord_id = Some("ORDER123".to_string());
		fill_report.order_id = Some("BROKER123".to_string());
		fill_report.exec_id = Some("EXEC789".to_string());
		fill_report.exec_type = Some("F".to_string()); // Fill
		fill_report.ord_status = Some(OrdStatus::Filled);
		fill_report.symbol = Some("MSFT".to_string());
		fill_report.side = Some(Side::Buy);
		fill_report.order_qty = Some(100.0);
		fill_report.last_qty = Some(100.0);
		fill_report.last_px = Some(100.25);
		fill_report.leaves_qty = Some(0.0);
		fill_report.cum_qty = Some(100.0);
		fill_report.avg_px = Some(100.25);

		assert!(fill_report.is_valid());
		assert_eq!(fill_report.ord_status, Some(OrdStatus::Filled));
		assert_eq!(fill_report.leaves_qty, Some(0.0));
		assert_eq!(fill_report.cum_qty, Some(100.0));
	}

	#[test]
	fn test_heartbeat_message() {
		let heartbeat = FixMessage::new(
			MsgType::Heartbeat,
			"CLIENT".to_string(),
			"BROKER".to_string(),
			10,
			"20241201-12:05:00.000".to_string(),
		);

		assert!(heartbeat.is_valid());
		assert_eq!(heartbeat.msg_type, MsgType::Heartbeat);
		assert_eq!(heartbeat.msg_seq_num, 10);
	}
}

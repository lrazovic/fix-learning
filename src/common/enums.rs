//! Common FIX 4.2 enums and types
//!
//! This module contains all the shared enum types used across different
//! FIX message types, including message types, encryption methods, and
//! trading-related enums.

use crate::macros::fix_enum;

// FIX 4.2 Message Types
fix_enum!(Loose MsgType {
	Heartbeat => "0",
	Logon => "A",
	NewOrderSingle => "D",
	ExecutionReport => "8",
	OrderCancelRequest => "F",
	MarketDataRequest => "V",
});

// Trading side enumeration
fix_enum!(Strict Side {
	Buy  => "1",
	Sell => "2",
});

// Order status enumeration
fix_enum!(Strict OrdStatus {
	New                => "0",
	PartiallyFilled    => "1",
	Filled             => "2",
	DoneForDay         => "3",
	Canceled           => "4",
	Replaced           => "5",
	PendingCancel      => "6",
	Stopped            => "7",
	Rejected           => "8",
	Suspended          => "9",
	PendingNew         => "A",
	Calculated         => "B",
	Expired            => "C",
	AcceptedForBidding => "D",
	PendingReplace     => "E",
});

// FIX 4.2 Encryption Methods for Logon messages
fix_enum!(Strict EncryptMethod {
	None => "0",
	Pkcs => "1",
	Des => "2",
	PkcsAndDes => "3",
	PgpAndDes => "4",
	PgpAndMd5 => "5",
	PemAndMd5 => "6",
});

#[cfg(test)]
mod tests {
	use super::*;
	use std::str::FromStr;

	#[test]
	fn test_msg_type_parsing() {
		assert_eq!(MsgType::from_str("0").unwrap(), MsgType::Heartbeat);
		assert_eq!(MsgType::from_str("A").unwrap(), MsgType::Logon);

		// Test loose mode - unknown values are stored in Other
		match MsgType::from_str("Z").unwrap() {
			MsgType::Other(s) => assert_eq!(s, "Z"),
			_ => panic!("Expected Other variant"),
		}
	}

	#[test]
	fn test_msg_type_display() {
		assert_eq!(format!("{}", MsgType::Heartbeat), "0");
		assert_eq!(format!("{}", MsgType::Logon), "A");
		assert_eq!(format!("{}", MsgType::Other("CUSTOM".to_string())), "CUSTOM");
	}

	#[test]
	fn test_side_parsing() {
		assert_eq!(Side::from_str("1").unwrap(), Side::Buy);
		assert_eq!(Side::from_str("2").unwrap(), Side::Sell);

		// Test strict mode - invalid values return error
		assert!(Side::from_str("3").is_err());
		assert!(Side::from_str("invalid").is_err());
	}

	#[test]
	fn test_side_display() {
		assert_eq!(format!("{}", Side::Buy), "1");
		assert_eq!(format!("{}", Side::Sell), "2");
	}

	#[test]
	fn test_ord_status_parsing() {
		assert_eq!(OrdStatus::from_str("0").unwrap(), OrdStatus::New);
		assert_eq!(OrdStatus::from_str("2").unwrap(), OrdStatus::Filled);
		assert_eq!(OrdStatus::from_str("A").unwrap(), OrdStatus::PendingNew);

		// Test strict mode
		assert!(OrdStatus::from_str("Z").is_err());
	}

	#[test]
	fn test_ord_status_display() {
		assert_eq!(format!("{}", OrdStatus::New), "0");
		assert_eq!(format!("{}", OrdStatus::Filled), "2");
		assert_eq!(format!("{}", OrdStatus::PendingNew), "A");
	}

	#[test]
	fn test_encrypt_method_parsing() {
		assert_eq!(EncryptMethod::from_str("0").unwrap(), EncryptMethod::None);
		assert_eq!(EncryptMethod::from_str("1").unwrap(), EncryptMethod::Pkcs);
		assert_eq!(EncryptMethod::from_str("6").unwrap(), EncryptMethod::PemAndMd5);

		// Test strict mode
		assert!(EncryptMethod::from_str("9").is_err());
	}

	#[test]
	fn test_encrypt_method_display() {
		assert_eq!(format!("{}", EncryptMethod::None), "0");
		assert_eq!(format!("{}", EncryptMethod::Pkcs), "1");
		assert_eq!(format!("{}", EncryptMethod::PemAndMd5), "6");
	}

	#[test]
	fn test_round_trip_conversions() {
		// Test that parsing and displaying are symmetric
		let msg_types = vec![MsgType::Heartbeat, MsgType::Logon, MsgType::Other("CUSTOM".to_string())];

		for msg_type in msg_types {
			let str_repr = format!("{}", msg_type);
			let parsed = MsgType::from_str(&str_repr).unwrap();
			assert_eq!(msg_type, parsed);
		}

		let sides = vec![Side::Buy, Side::Sell];
		for side in sides {
			let str_repr = format!("{}", side);
			let parsed = Side::from_str(&str_repr).unwrap();
			assert_eq!(side, parsed);
		}
	}
}

//! Integration tests for real-world FIX message scenarios
//!
//! These tests simulate actual trading workflows and message sequences
//! that would occur in a real FIX session.

use fix_learning::{FixMessage, MsgType, OrdStatus, Side};
use std::str::FromStr;

#[cfg(test)]
mod trading_workflow_tests {
	use super::*;

	#[test]
	fn test_complete_order_lifecycle() {
		// 1. Client sends New Order Single
		let mut new_order = FixMessage::new(
			MsgType::NewOrderSingle,
			"CLIENT_TRADER".to_string(),
			"PRIME_BROKER".to_string(),
			1,
			"20241201-09:30:00.000".to_string(),
		);

		new_order.cl_ord_id = Some("CLIENT_ORD_001".to_string());
		new_order.symbol = Some("AAPL".to_string());
		new_order.side = Some(Side::Buy);
		new_order.order_qty = Some(1000.0);
		new_order.ord_type = Some("2".into()); // Limit order
		new_order.price = Some(150.25);
		new_order.time_in_force = Some("0".into()); // Day order

		assert!(new_order.is_valid());
		assert_eq!(new_order.msg_type, MsgType::NewOrderSingle);

		// 2. Broker responds with Execution Report (New)
		let mut exec_new = FixMessage::new(
			MsgType::ExecutionReport,
			"PRIME_BROKER".to_string(),
			"CLIENT_TRADER".to_string(),
			1,
			"20241201-09:30:00.100".to_string(),
		);

		exec_new.cl_ord_id = Some("CLIENT_ORD_001".to_string());
		exec_new.order_id = Some("BROKER_ORD_12345".to_string());
		exec_new.exec_id = Some("EXEC_001".to_string());
		exec_new.exec_type = Some("0".into()); // New
		exec_new.ord_status = Some(OrdStatus::New);
		exec_new.symbol = Some("AAPL".to_string());
		exec_new.side = Some(Side::Buy);
		exec_new.order_qty = Some(1000.0);
		exec_new.price = Some(150.25);
		exec_new.leaves_qty = Some(1000.0);
		exec_new.cum_qty = Some(0.0);

		assert_eq!(exec_new.ord_status, Some(OrdStatus::New));
		assert_eq!(exec_new.leaves_qty, Some(1000.0));

		// 3. Partial fill occurs
		let mut exec_partial = FixMessage::new(
			MsgType::ExecutionReport,
			"PRIME_BROKER".to_string(),
			"CLIENT_TRADER".to_string(),
			2,
			"20241201-09:30:15.250".to_string(),
		);

		exec_partial.cl_ord_id = Some("CLIENT_ORD_001".to_string());
		exec_partial.order_id = Some("BROKER_ORD_12345".to_string());
		exec_partial.exec_id = Some("EXEC_002".to_string());
		exec_partial.exec_type = Some("F".into()); // Trade
		exec_partial.ord_status = Some(OrdStatus::PartiallyFilled);
		exec_partial.symbol = Some("AAPL".to_string());
		exec_partial.side = Some(Side::Buy);
		exec_partial.order_qty = Some(1000.0);
		exec_partial.last_qty = Some(400.0);
		exec_partial.last_px = Some(150.20);
		exec_partial.leaves_qty = Some(600.0);
		exec_partial.cum_qty = Some(400.0);
		exec_partial.avg_px = Some(150.20);

		assert_eq!(exec_partial.ord_status, Some(OrdStatus::PartiallyFilled));
		assert_eq!(exec_partial.cum_qty, Some(400.0));
		assert_eq!(exec_partial.leaves_qty, Some(600.0));

		// 4. Final fill completes the order
		let mut exec_filled = FixMessage::new(
			MsgType::ExecutionReport,
			"PRIME_BROKER".to_string(),
			"CLIENT_TRADER".to_string(),
			3,
			"20241201-09:30:45.500".to_string(),
		);

		exec_filled.cl_ord_id = Some("CLIENT_ORD_001".to_string());
		exec_filled.order_id = Some("BROKER_ORD_12345".to_string());
		exec_filled.exec_id = Some("EXEC_003".to_string());
		exec_filled.exec_type = Some("F".into()); // Trade
		exec_filled.ord_status = Some(OrdStatus::Filled);
		exec_filled.symbol = Some("AAPL".to_string());
		exec_filled.side = Some(Side::Buy);
		exec_filled.order_qty = Some(1000.0);
		exec_filled.last_qty = Some(600.0);
		exec_filled.last_px = Some(150.18);
		exec_filled.leaves_qty = Some(0.0);
		exec_filled.cum_qty = Some(1000.0);
		exec_filled.avg_px = Some(150.19); // Weighted average

		assert_eq!(exec_filled.ord_status, Some(OrdStatus::Filled));
		assert_eq!(exec_filled.cum_qty, Some(1000.0));
		assert_eq!(exec_filled.leaves_qty, Some(0.0));
	}

	#[test]
	fn test_order_cancel_workflow() {
		// 1. Original order
		let mut original_order = FixMessage::new(
			MsgType::NewOrderSingle,
			"HEDGE_FUND".to_string(),
			"ECN_BROKER".to_string(),
			10,
			"20241201-10:15:00.000".to_string(),
		);

		original_order.cl_ord_id = Some("HF_ORDER_500".to_string());
		original_order.symbol = Some("TSLA".to_string());
		original_order.side = Some(Side::Sell);
		original_order.order_qty = Some(2000.0);
		original_order.ord_type = Some("2".into()); // Limit
		original_order.price = Some(245.75);

		// 2. Order acknowledged
		let mut ack_report = FixMessage::new(
			MsgType::ExecutionReport,
			"ECN_BROKER".to_string(),
			"HEDGE_FUND".to_string(),
			10,
			"20241201-10:15:00.050".to_string(),
		);

		ack_report.cl_ord_id = Some("HF_ORDER_500".to_string());
		ack_report.order_id = Some("ECN_12345".to_string());
		ack_report.exec_id = Some("ACK_001".to_string());
		ack_report.exec_type = Some("0".into()); // New
		ack_report.ord_status = Some(OrdStatus::New);
		ack_report.leaves_qty = Some(2000.0);
		ack_report.cum_qty = Some(0.0);

		// 3. Client decides to cancel
		let mut cancel_request = FixMessage::new(
			MsgType::OrderCancelRequest,
			"HEDGE_FUND".to_string(),
			"ECN_BROKER".to_string(),
			11,
			"20241201-10:16:30.000".to_string(),
		);

		cancel_request.cl_ord_id = Some("HF_ORDER_500".to_string());
		cancel_request.order_id = Some("ECN_12345".to_string());
		cancel_request.symbol = Some("TSLA".to_string());
		cancel_request.side = Some(Side::Sell);
		cancel_request.order_qty = Some(2000.0);

		// 4. Broker confirms cancellation
		let mut cancel_report = FixMessage::new(
			MsgType::ExecutionReport,
			"ECN_BROKER".to_string(),
			"HEDGE_FUND".to_string(),
			11,
			"20241201-10:16:30.100".to_string(),
		);

		cancel_report.cl_ord_id = Some("HF_ORDER_500".to_string());
		cancel_report.order_id = Some("ECN_12345".to_string());
		cancel_report.exec_id = Some("CANCEL_001".to_string());
		cancel_report.exec_type = Some("4".to_string()); // Canceled
		cancel_report.ord_status = Some(OrdStatus::Canceled);
		cancel_report.symbol = Some("TSLA".to_string());
		cancel_report.side = Some(Side::Sell);
		cancel_report.order_qty = Some(2000.0);
		cancel_report.leaves_qty = Some(0.0);
		cancel_report.cum_qty = Some(0.0);

		assert_eq!(cancel_report.ord_status, Some(OrdStatus::Canceled));
		assert_eq!(cancel_report.leaves_qty, Some(0.0));
	}

	#[test]
	fn test_order_replace_workflow() {
		// 1. Original order
		let original_cl_ord_id = "PROP_TRADE_100";
		let new_cl_ord_id = "PROP_TRADE_100_R1";

		// 2. Order replace request
		let mut replace_request = FixMessage::new(
			MsgType::OrderCancelReplaceRequest,
			"PROP_DESK".to_string(),
			"DARK_POOL".to_string(),
			25,
			"20241201-11:30:00.000".to_string(),
		);

		replace_request.cl_ord_id = Some(new_cl_ord_id.to_string());
		replace_request.order_id = Some("DARK_567890".to_string());
		replace_request.symbol = Some("NVDA".to_string());
		replace_request.side = Some(Side::Buy);
		replace_request.order_qty = Some(1500.0); // Increased from 1000
		replace_request.ord_type = Some("2".into());
		replace_request.price = Some(520.50); // Increased price
		// Add original client order ID in additional fields
		replace_request.set_field(41, original_cl_ord_id.to_string()); // OrigClOrdID

		// 3. Replace confirmation
		let mut replace_report = FixMessage::new(
			MsgType::ExecutionReport,
			"DARK_POOL".to_string(),
			"PROP_DESK".to_string(),
			25,
			"20241201-11:30:00.150".to_string(),
		);

		replace_report.cl_ord_id = Some(new_cl_ord_id.to_string());
		replace_report.order_id = Some("DARK_567890".to_string());
		replace_report.exec_id = Some("REPLACE_001".to_string());
		replace_report.exec_type = Some("5".to_string()); // Replace
		replace_report.ord_status = Some(OrdStatus::Replaced);
		replace_report.symbol = Some("NVDA".to_string());
		replace_report.side = Some(Side::Buy);
		replace_report.order_qty = Some(1500.0);
		replace_report.price = Some(520.50);
		replace_report.leaves_qty = Some(1500.0);
		replace_report.cum_qty = Some(0.0);

		assert_eq!(replace_report.ord_status, Some(OrdStatus::Replaced));
		assert_eq!(replace_report.order_qty, Some(1500.0));
		assert_eq!(replace_report.price, Some(520.50));
		assert_eq!(replace_request.get_field(41), Some(&original_cl_ord_id.to_string()));
	}

	#[test]
	fn test_heartbeat_sequence() {
		let mut sequence_num = 100;

		// Heartbeat from client to server
		let client_heartbeat = FixMessage::new(
			MsgType::Heartbeat,
			"CLIENT_SYSTEM".to_string(),
			"MARKET_DATA_PROVIDER".to_string(),
			sequence_num,
			"20241201-12:00:00.000".to_string(),
		);

		sequence_num += 1;

		// Heartbeat response from server
		let server_heartbeat = FixMessage::new(
			MsgType::Heartbeat,
			"MARKET_DATA_PROVIDER".to_string(),
			"CLIENT_SYSTEM".to_string(),
			sequence_num,
			"20241201-12:00:00.100".to_string(),
		);

		assert_eq!(client_heartbeat.msg_type, MsgType::Heartbeat);
		assert_eq!(server_heartbeat.msg_type, MsgType::Heartbeat);
		assert!(client_heartbeat.is_valid());
		assert!(server_heartbeat.is_valid());
	}

	#[test]
	fn test_market_data_request_workflow() {
		// Market data subscription request
		let mut md_request = FixMessage::new(
			MsgType::MarketDataRequest,
			"TRADING_ENGINE".to_string(),
			"MARKET_DATA_SERVICE".to_string(),
			50,
			"20241201-13:15:00.000".to_string(),
		);

		md_request.symbol = Some("SPY".to_string());
		md_request.set_field(262, "SPY_L1_FEED".to_string()); // MDReqID
		md_request.set_field(263, "1".to_string()); // SubscriptionRequestType (Snapshot + Updates)
		md_request.set_field(264, "1".to_string()); // MarketDepth (Top of book)

		// Market data snapshot response
		let mut md_snapshot = FixMessage::new(
			MsgType::MarketDataSnapshot,
			"MARKET_DATA_SERVICE".to_string(),
			"TRADING_ENGINE".to_string(),
			50,
			"20241201-13:15:00.050".to_string(),
		);

		md_snapshot.symbol = Some("SPY".to_string());
		md_snapshot.set_field(262, "SPY_L1_FEED".to_string()); // MDReqID
		md_snapshot.set_field(268, "2".to_string()); // NoMDEntries (Bid and Ask)
		md_snapshot.set_field(269, "0".to_string()); // MDEntryType (Bid)
		md_snapshot.set_field(270, "445.25".to_string()); // MDEntryPx (Bid price)
		md_snapshot.set_field(271, "1000".to_string()); // MDEntrySize (Bid size)

		assert_eq!(md_request.msg_type, MsgType::MarketDataRequest);
		assert_eq!(md_snapshot.msg_type, MsgType::MarketDataSnapshot);
		assert_eq!(md_request.get_field(262), Some(&"SPY_L1_FEED".to_string()));
		assert_eq!(md_snapshot.get_field(270), Some(&"445.25".to_string()));
	}

	#[test]
	fn test_reject_message() {
		// Invalid order that gets rejected
		let mut reject_msg = FixMessage::new(
			MsgType::Reject,
			"EXCHANGE".to_string(),
			"BAD_CLIENT".to_string(),
			999,
			"20241201-14:00:00.000".to_string(),
		);

		reject_msg.text = Some("Invalid symbol: INVALID_SYM".to_string());
		reject_msg.set_field(45, "1000".to_string()); // RefSeqNum - sequence number being rejected
		reject_msg.set_field(371, "35".to_string()); // RefTagID - tag 35 (MsgType) has issue
		reject_msg.set_field(372, "5".to_string()); // RefMsgType - value that was rejected
		reject_msg.set_field(373, "2".to_string()); // SessionRejectReason - invalid tag number

		assert_eq!(reject_msg.msg_type, MsgType::Reject);
		assert_eq!(reject_msg.text, Some("Invalid symbol: INVALID_SYM".to_string()));
		assert_eq!(reject_msg.get_field(45), Some(&"1000".to_string()));
	}

	#[test]
	fn test_security_definition_workflow() {
		// Security definition request
		let mut sec_def_request = FixMessage::new(
			MsgType::SecurityDefinitionRequest,
			"RISK_SYSTEM".to_string(),
			"REFERENCE_DATA".to_string(),
			200,
			"20241201-08:00:00.000".to_string(),
		);

		sec_def_request.symbol = Some("MSFT".to_string());
		sec_def_request.set_field(320, "SEC_DEF_REQ_001".to_string()); // SecurityReqID
		sec_def_request.set_field(321, "1".to_string()); // SecurityRequestType

		// Security definition response
		let mut sec_def_response = FixMessage::new(
			MsgType::SecurityDefinition,
			"REFERENCE_DATA".to_string(),
			"RISK_SYSTEM".to_string(),
			200,
			"20241201-08:00:00.100".to_string(),
		);

		sec_def_response.symbol = Some("MSFT".to_string());
		sec_def_response.security_type = Some("CS".to_string()); // Common Stock
		sec_def_response.set_field(320, "SEC_DEF_REQ_001".to_string()); // SecurityReqID
		sec_def_response.set_field(323, "1".to_string()); // SecurityResponseType (Accept)
		sec_def_response.set_field(107, "Microsoft Corporation".to_string()); // SecurityDesc

		assert_eq!(sec_def_request.msg_type, MsgType::SecurityDefinitionRequest);
		assert_eq!(sec_def_response.msg_type, MsgType::SecurityDefinition);
		assert_eq!(sec_def_response.get_field(107), Some(&"Microsoft Corporation".to_string()));
	}
}

#[cfg(test)]
mod message_parsing_tests {
	use super::*;

	#[test]
	fn test_parse_real_fix_message() {
		// Test parsing a real FIX message from https://robertwray.co.uk/blog/the-anatomy-of-a-fix-message
		let fix_string = "8=FIX.4.2\x019=171\x0135=R\x0134=3257\x0149=COMP-PRICES\x0152=20180508-09:02:43.968\x0156=BANK-PRICES\x01131=Q-EURGBP-BUY-3357-636613669639680362\x01146=1\x0155=EUR/GBP\x0115=EUR\x0138=3357\x0140=C\x0154=1\x0164=20180508\x01167=FOR\x0110=150";

		let result = FixMessage::from_fix_string(&fix_string);

		assert!(result.is_ok(), "Failed to parse FIX message: {:?}", result.err());

		let message = result.unwrap();

		// Verify standard header fields
		assert_eq!(message.begin_string, "FIX.4.2");
		assert_eq!(message.body_length, 171);
		assert_eq!(message.msg_type, MsgType::Other("R".to_string())); // Quote Request
		assert_eq!(message.msg_seq_num, 3257);
		assert_eq!(message.sender_comp_id, "COMP-PRICES");
		assert_eq!(message.sending_time, "20180508-09:02:43.968");
		assert_eq!(message.target_comp_id, "BANK-PRICES");

		// Verify message-specific fields
		assert_eq!(message.symbol, Some("EUR/GBP".to_string()));
		assert_eq!(message.side, Some(Side::Buy));
		assert_eq!(message.order_qty, Some(3357.0));
		assert_eq!(message.security_type, Some("FOR".to_string()));
		assert_eq!(message.checksum, "150");

		// Verify additional fields are captured
		assert_eq!(message.get_field(131), Some(&"Q-EURGBP-BUY-3357-636613669639680362".to_string()));
		assert_eq!(message.get_field(146), Some(&"1".to_string()));
		assert_eq!(message.get_field(15), Some(&"EUR".to_string()));
		assert_eq!(message.get_field(64), Some(&"20180508".to_string()));

		// Verify the message is considered valid
		assert!(message.is_valid());
	}
}

mod error_handling_tests {
	use super::*;

	#[test]
	fn test_invalid_message_validation() {
		// Test various invalid message scenarios
		let mut invalid_msg = FixMessage::default();

		// Empty sender should make it invalid
		invalid_msg.sender_comp_id = "".to_string();
		assert!(!invalid_msg.is_valid());

		// Fix sender, but empty target
		invalid_msg.sender_comp_id = "VALID_SENDER".to_string();
		invalid_msg.target_comp_id = "".to_string();
		assert!(!invalid_msg.is_valid());

		// Fix target, but empty sending time
		invalid_msg.target_comp_id = "VALID_TARGET".to_string();
		invalid_msg.sending_time = "".to_string();
		assert!(!invalid_msg.is_valid());

		// Fix sending time - should now be valid
		invalid_msg.sending_time = "20241201-12:00:00.000".to_string();
		assert!(invalid_msg.is_valid());
	}

	#[test]
	fn test_enum_edge_cases() {
		// Test Side enum edge cases
		assert!(Side::from_str("").is_err());
		assert!(Side::from_str("3").is_err());
		assert!(Side::from_str("B").is_err());
		assert!(Side::from_str("S").is_err());

		// Test OrdStatus enum edge cases
		assert!(OrdStatus::from_str("").is_err());
		assert!(OrdStatus::from_str("Z").is_err());
		assert!(OrdStatus::from_str("99").is_err());

		// Test MsgType with various inputs
		assert_eq!(MsgType::from_str("").unwrap(), MsgType::Other("".to_string()));
		assert_eq!(MsgType::from_str("CUSTOM").unwrap(), MsgType::Other("CUSTOM".to_string()));
		assert_eq!(MsgType::from_str("999").unwrap(), MsgType::Other("999".to_string()));
	}

	#[test]
	fn test_field_operations_edge_cases() {
		let mut msg = FixMessage::default();

		// Test getting non-existent field
		assert_eq!(msg.get_field(99999), None);

		// Test setting and overwriting fields
		msg.set_field(1000, "value1".to_string());
		assert_eq!(msg.get_field(1000), Some(&"value1".to_string()));

		msg.set_field(1000, "value2".to_string());
		assert_eq!(msg.get_field(1000), Some(&"value2".to_string()));

		// Test with zero tag
		msg.set_field(0, "zero_tag".to_string());
		assert_eq!(msg.get_field(0), Some(&"zero_tag".to_string()));
	}
}

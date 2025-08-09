//! Tests for the FIX message builder pattern and serialization functionality
//!
//! These tests verify that the builder pattern works correctly and that
//! messages can be serialized to and parsed from FIX wire format.

use fix_learning::{FixMessage, FixMessageBuilder, MsgType, OrdStatus, Side};
use std::str::FromStr;

#[cfg(test)]
mod builder_pattern_tests {
    use super::*;

    #[test]
    fn test_basic_builder_creation() {
        let message = FixMessage::builder(
            MsgType::Heartbeat,
            "SENDER".to_string(),
            "TARGET".to_string(),
            1,
            "20241201-12:00:00.000".to_string(),
        )
        .build();

        assert_eq!(message.msg_type, MsgType::Heartbeat);
        assert_eq!(message.sender_comp_id, "SENDER");
        assert_eq!(message.target_comp_id, "TARGET");
        assert_eq!(message.msg_seq_num, 1);
        assert_eq!(message.sending_time, "20241201-12:00:00.000");
    }

    #[test]
    fn test_builder_fluent_interface() {
        let message = FixMessage::builder(
            MsgType::NewOrderSingle,
            "CLIENT".to_string(),
            "BROKER".to_string(),
            100,
            "20241201-09:30:00.000".to_string(),
        )
        .cl_ord_id("ORDER123".to_string())
        .symbol("AAPL".to_string())
        .side(Side::Buy)
        .order_qty(100.0)
        .ord_type("2".to_string())
        .price(150.25)
        .time_in_force("0".to_string())
        .build();

        assert_eq!(message.cl_ord_id, Some("ORDER123".to_string()));
        assert_eq!(message.symbol, Some("AAPL".to_string()));
        assert_eq!(message.side, Some(Side::Buy));
        assert_eq!(message.order_qty, Some(100.0));
        assert_eq!(message.price, Some(150.25));
    }

    #[test]
    fn test_builder_with_custom_fields() {
        let message = FixMessage::builder(
            MsgType::ExecutionReport,
            "EXCHANGE".to_string(),
            "CLIENT".to_string(),
            200,
            "20241201-10:00:00.000".to_string(),
        )
        .symbol("MSFT".to_string())
        .field(207, "NASDAQ".to_string()) // SecurityExchange
        .field(6000, "CUSTOM_DATA".to_string())
        .field(9999, "TEST_FIELD".to_string())
        .build();

        assert_eq!(message.symbol, Some("MSFT".to_string()));
        assert_eq!(message.get_field(207), Some(&"NASDAQ".to_string()));
        assert_eq!(message.get_field(6000), Some(&"CUSTOM_DATA".to_string()));
        assert_eq!(message.get_field(9999), Some(&"TEST_FIELD".to_string()));
    }

    #[test]
    fn test_builder_all_standard_fields() {
        let message = FixMessage::builder(
            MsgType::ExecutionReport,
            "BROKER".to_string(),
            "CLIENT".to_string(),
            500,
            "20241201-11:30:00.000".to_string(),
        )
        .cl_ord_id("CLIENT_ORDER_1".to_string())
        .order_id("BROKER_ORDER_1".to_string())
        .exec_id("EXEC_001".to_string())
        .exec_type("F".to_string())
        .ord_status(OrdStatus::Filled)
        .symbol("TSLA".to_string())
        .security_type("CS".to_string())
        .side(Side::Sell)
        .order_qty(200.0)
        .ord_type("1".to_string())
        .price(250.50)
        .last_qty(200.0)
        .last_px(250.75)
        .leaves_qty(0.0)
        .cum_qty(200.0)
        .avg_px(250.75)
        .text("FILL COMPLETE".to_string())
        .time_in_force("0".to_string())
        .exec_inst("O".to_string())
        .handl_inst("1".to_string())
        .exec_ref_id("REF_001".to_string())
        .exec_trans_type("0".to_string())
        .build();

        // Verify all fields were set correctly
        assert_eq!(message.cl_ord_id, Some("CLIENT_ORDER_1".to_string()));
        assert_eq!(message.order_id, Some("BROKER_ORDER_1".to_string()));
        assert_eq!(message.exec_id, Some("EXEC_001".to_string()));
        assert_eq!(message.exec_type, Some("F".to_string()));
        assert_eq!(message.ord_status, Some(OrdStatus::Filled));
        assert_eq!(message.symbol, Some("TSLA".to_string()));
        assert_eq!(message.side, Some(Side::Sell));
        assert_eq!(message.order_qty, Some(200.0));
        assert_eq!(message.last_qty, Some(200.0));
        assert_eq!(message.cum_qty, Some(200.0));
        assert_eq!(message.text, Some("FILL COMPLETE".to_string()));
    }

    #[test]
    fn test_builder_from_existing_message() {
        let original = FixMessage::builder(
            MsgType::ExecutionReport,
            "PHLX".to_string(),
            "PERS".to_string(),
            1,
            "20071123-05:30:00.000".to_string(),
        )
        .cl_ord_id("ATOMNOCCC9990900".to_string())
        .symbol("MSFT".to_string())
        .price(15.0)
        .build();

        let modified = FixMessageBuilder::from_message(original.clone())
            .symbol("GOOGL".to_string())
            .price(2500.0)
            .field(5000, "MODIFIED".to_string())
            .build();

        // Original fields should be preserved
        assert_eq!(modified.sender_comp_id, original.sender_comp_id);
        assert_eq!(modified.target_comp_id, original.target_comp_id);
        assert_eq!(modified.cl_ord_id, original.cl_ord_id);

        // Modified fields should be updated
        assert_eq!(modified.symbol, Some("GOOGL".to_string()));
        assert_eq!(modified.price, Some(2500.0));
        assert_eq!(modified.get_field(5000), Some(&"MODIFIED".to_string()));

        // Original should remain unchanged
        assert_eq!(original.symbol, Some("MSFT".to_string()));
        assert_eq!(original.price, Some(15.0));
        // Verify the fields were actually modified
        assert_ne!(original.symbol, modified.symbol);
        assert_ne!(original.price, modified.price);
    }

    #[test]
    fn test_builder_optional_header_fields() {
        let message = FixMessage::builder(
            MsgType::TestRequest,
            "SENDER".to_string(),
            "TARGET".to_string(),
            10,
            "20241201-12:00:00.000".to_string(),
        )
        .poss_dup_flag(true)
        .poss_resend(false)
        .orig_sending_time("20241201-11:59:00.000".to_string())
        .build();

        assert_eq!(message.poss_dup_flag, Some(true));
        assert_eq!(message.poss_resend, Some(false));
        assert_eq!(
            message.orig_sending_time,
            Some("20241201-11:59:00.000".to_string())
        );
    }
}

#[cfg(test)]
mod serialization_tests {
    use super::*;

    #[test]
    fn test_simple_message_serialization() {
        let message = FixMessage::builder(
            MsgType::Heartbeat,
            "CLIENT".to_string(),
            "SERVER".to_string(),
            1,
            "20241201-12:00:00.000".to_string(),
        )
        .build();

        let fix_string = message.to_fix_string();

        // Should contain all required fields
        assert!(fix_string.contains("8=FIX.4.2"));
        assert!(fix_string.contains("35=0")); // Heartbeat
        assert!(fix_string.contains("34=1")); // Seq num
        assert!(fix_string.contains("49=CLIENT"));
        assert!(fix_string.contains("56=SERVER"));
        assert!(fix_string.contains("52=20241201-12:00:00.000"));
        assert!(fix_string.contains("10=")); // Checksum
    }

    #[test]
    fn test_new_order_single_serialization() {
        let message = FixMessage::builder(
            MsgType::NewOrderSingle,
            "TESTBUY3".to_string(),
            "TESTSELL3".to_string(),
            972,
            "20190206-16:25:10.403".to_string(),
        )
        .cl_ord_id("14163685067084226997921".to_string())
        .order_qty(100.0)
        .ord_type("1".to_string()) // Market order
        .side(Side::Buy)
        .symbol("AAPL".to_string())
        .field(60, "20190206-16:25:08.968".to_string()) // TransactTime
        .field(207, "TO".to_string()) // SecurityExchange
        .field(6000, "TEST1234".to_string()) // Custom field
        .build();

        let fix_string = message.to_fix_string();

        // Verify key fields are present
        assert!(fix_string.contains("8=FIX.4.2"));
        assert!(fix_string.contains("35=D")); // NewOrderSingle
        assert!(fix_string.contains("34=972"));
        assert!(fix_string.contains("49=TESTBUY3"));
        assert!(fix_string.contains("56=TESTSELL3"));
        assert!(fix_string.contains("11=14163685067084226997921"));
        assert!(fix_string.contains("38=100"));
        assert!(fix_string.contains("40=1"));
        assert!(fix_string.contains("54=1"));
        assert!(fix_string.contains("55=AAPL"));
        assert!(fix_string.contains("60=20190206-16:25:08.968"));
        assert!(fix_string.contains("207=TO"));
        assert!(fix_string.contains("6000=TEST1234"));
    }

    #[test]
    fn test_execution_report_serialization() {
        let message = FixMessage::builder(
            MsgType::ExecutionReport,
            "BROKER".to_string(),
            "CLIENT".to_string(),
            100,
            "20241201-10:30:00.000".to_string(),
        )
        .cl_ord_id("ORDER_123".to_string())
        .order_id("BROKER_456".to_string())
        .exec_id("EXEC_789".to_string())
        .exec_type("F".to_string())
        .ord_status(OrdStatus::Filled)
        .symbol("MSFT".to_string())
        .side(Side::Buy)
        .order_qty(500.0)
        .last_qty(500.0)
        .last_px(300.25)
        .leaves_qty(0.0)
        .cum_qty(500.0)
        .avg_px(300.25)
        .build();

        let fix_string = message.to_fix_string();

        assert!(fix_string.contains("35=8")); // ExecutionReport
        assert!(fix_string.contains("11=ORDER_123"));
        assert!(fix_string.contains("37=BROKER_456"));
        assert!(fix_string.contains("17=EXEC_789"));
        assert!(fix_string.contains("150=F"));
        assert!(fix_string.contains("39=2")); // Filled
        assert!(fix_string.contains("55=MSFT"));
        assert!(fix_string.contains("54=1")); // Buy
        assert!(fix_string.contains("38=500"));
        assert!(fix_string.contains("32=500"));
        assert!(fix_string.contains("31=300.25"));
    }

    #[test]
    fn test_checksum_calculation() {
        let message = FixMessage::builder(
            MsgType::Heartbeat,
            "TEST".to_string(),
            "DEST".to_string(),
            1,
            "20241201-12:00:00.000".to_string(),
        )
        .build();

        let fix_string = message.to_fix_string();
        let checksum_part = fix_string.split("10=").nth(1).unwrap_or("");
        let checksum_str = checksum_part.split('\x01').next().unwrap_or("");

        // Checksum should be a 3-digit number
        assert_eq!(checksum_str.len(), 3);
        assert!(checksum_str.parse::<u32>().is_ok());
    }

    #[test]
    fn test_body_length_calculation() {
        let message = FixMessage::builder(
            MsgType::NewOrderSingle,
            "SENDER".to_string(),
            "TARGET".to_string(),
            1,
            "20241201-12:00:00.000".to_string(),
        )
        .cl_ord_id("TEST123".to_string())
        .symbol("AAPL".to_string())
        .build();

        let fix_string = message.to_fix_string();

        // Extract body length from the message
        let body_length_part = fix_string.split("9=").nth(1).unwrap();
        let body_length_str = body_length_part.split('\x01').next().unwrap();
        let body_length: usize = body_length_str.parse().unwrap();

        // Body length should be reasonable (not zero, not huge)
        assert!(body_length > 0);
        assert!(body_length < 10000); // Reasonable upper bound for test
    }

    #[test]
    fn test_field_ordering() {
        let message = FixMessage::builder(
            MsgType::NewOrderSingle,
            "SENDER".to_string(),
            "TARGET".to_string(),
            1,
            "20241201-12:00:00.000".to_string(),
        )
        .field(6000, "CUSTOM1".to_string())
        .field(207, "EXCHANGE".to_string())
        .field(9999, "CUSTOM2".to_string())
        .build();

        let fix_string = message.to_fix_string();

        // Custom fields should appear in order
        let pos_207 = fix_string.find("207=EXCHANGE").unwrap();
        let pos_6000 = fix_string.find("6000=CUSTOM1").unwrap();
        let pos_9999 = fix_string.find("9999=CUSTOM2").unwrap();

        assert!(pos_207 < pos_6000);
        assert!(pos_6000 < pos_9999);
    }
}

#[cfg(test)]
mod parsing_tests {
    use super::*;

    #[test]
    fn test_parse_simple_message() {
        // Create a simple heartbeat message
        let original = FixMessage::builder(
            MsgType::Heartbeat,
            "CLIENT".to_string(),
            "SERVER".to_string(),
            1,
            "20241201-12:00:00.000".to_string(),
        )
        .build();

        let fix_string = original.to_fix_string();
        let parsed = FixMessage::from_fix_string(&fix_string).unwrap();

        assert_eq!(parsed.begin_string, "FIX.4.2");
        assert_eq!(parsed.msg_type, MsgType::Heartbeat);
        assert_eq!(parsed.sender_comp_id, "CLIENT");
        assert_eq!(parsed.target_comp_id, "SERVER");
        assert_eq!(parsed.msg_seq_num, 1);
        assert_eq!(parsed.sending_time, "20241201-12:00:00.000");
    }

    #[test]
    fn test_parse_new_order_single() {
        let fix_message = "8=FIX.4.2\x019=100\x0135=D\x0134=1\x0149=CLIENT\x0152=20241201-12:00:00.000\x0156=BROKER\x0111=ORDER123\x0138=100\x0140=2\x0154=1\x0155=AAPL\x0144=150.25\x0110=123\x01";

        let parsed = FixMessage::from_fix_string(fix_message).unwrap();

        assert_eq!(parsed.msg_type, MsgType::NewOrderSingle);
        assert_eq!(parsed.cl_ord_id, Some("ORDER123".to_string()));
        assert_eq!(parsed.order_qty, Some(100.0));
        assert_eq!(parsed.ord_type, Some("2".to_string()));
        assert_eq!(parsed.side, Some(Side::Buy));
        assert_eq!(parsed.symbol, Some("AAPL".to_string()));
        assert_eq!(parsed.price, Some(150.25));
    }

    #[test]
    fn test_parse_with_custom_fields() {
        let fix_message = "8=FIX.4.2\x019=50\x0135=8\x0134=1\x0149=BROKER\x0152=20241201-12:00:00.000\x0156=CLIENT\x01207=NASDAQ\x016000=CUSTOM\x0110=123\x01";

        let parsed = FixMessage::from_fix_string(fix_message).unwrap();

        assert_eq!(parsed.msg_type, MsgType::ExecutionReport);
        assert_eq!(parsed.get_field(207), Some(&"NASDAQ".to_string()));
        assert_eq!(parsed.get_field(6000), Some(&"CUSTOM".to_string()));
    }

    #[test]
    fn test_parse_empty_message() {
        let result = FixMessage::from_fix_string("");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Empty FIX message");
    }

    #[test]
    fn test_parse_malformed_field() {
        let fix_message = "8=FIX.4.2\x01INVALID_FIELD\x0135=0\x0110=123\x01";
        let parsed = FixMessage::from_fix_string(fix_message);

        // Should handle malformed fields gracefully by skipping them
        assert!(parsed.is_ok());
    }

    #[test]
    fn test_round_trip_serialization() {
        let original = FixMessage::builder(
            MsgType::ExecutionReport,
            "EXCHANGE".to_string(),
            "CLIENT".to_string(),
            500,
            "20241201-15:30:00.000".to_string(),
        )
        .cl_ord_id("CLIENT_ORDER_789".to_string())
        .symbol("NVDA".to_string())
        .side(Side::Sell)
        .order_qty(150.0)
        .ord_status(OrdStatus::PartiallyFilled)
        .field(207, "NASDAQ".to_string())
        .field(6000, "CUSTOM_DATA".to_string())
        .build();

        let fix_string = original.to_fix_string();
        let parsed = FixMessage::from_fix_string(&fix_string).unwrap();

        // Core fields should match
        assert_eq!(parsed.msg_type, original.msg_type);
        assert_eq!(parsed.sender_comp_id, original.sender_comp_id);
        assert_eq!(parsed.target_comp_id, original.target_comp_id);
        assert_eq!(parsed.cl_ord_id, original.cl_ord_id);
        assert_eq!(parsed.symbol, original.symbol);
        assert_eq!(parsed.side, original.side);
        assert_eq!(parsed.order_qty, original.order_qty);
        assert_eq!(parsed.ord_status, original.ord_status);

        // Custom fields should match
        assert_eq!(parsed.get_field(207), original.get_field(207));
        assert_eq!(parsed.get_field(6000), original.get_field(6000));
    }
}

#[cfg(test)]
mod real_world_examples {
    use super::*;

    #[test]
    fn test_user_provided_message_structure() {
        // Recreate the message structure from the user's example:
        // "8=FIX.4.29=16335=D34=97249=TESTBUY352=20190206-16:25:10.40356=TESTSELL311=14163685067084226997921=238=10040=154=155=AAPL60=20190206-16:25:08.968207=TO6000=TEST123410=106"

        let message = FixMessage::builder(
            MsgType::NewOrderSingle,
            "TESTBUY3".to_string(),
            "TESTSELL3".to_string(),
            972,
            "20190206-16:25:10.403".to_string(),
        )
        .cl_ord_id("14163685067084226997921".to_string())
        .field(21, "2".to_string()) // HandlInst
        .order_qty(100.0)
        .ord_type("1".to_string()) // Market order
        .side(Side::Buy)
        .symbol("AAPL".to_string())
        .field(60, "20190206-16:25:08.968".to_string()) // TransactTime
        .field(207, "TO".to_string()) // SecurityExchange
        .field(6000, "TEST1234".to_string()) // Custom field
        .build();

        let fix_string = message.to_fix_string();

        // Verify the structure matches expected format
        assert!(fix_string.starts_with("8=FIX.4.2\x01"));
        assert!(fix_string.contains("35=D\x01")); // NewOrderSingle
        assert!(fix_string.contains("34=972\x01"));
        assert!(fix_string.contains("49=TESTBUY3\x01"));
        assert!(fix_string.contains("56=TESTSELL3\x01"));
        assert!(fix_string.contains("11=14163685067084226997921\x01"));
        assert!(fix_string.contains("21=2\x01"));
        assert!(fix_string.contains("38=100\x01"));
        assert!(fix_string.contains("40=1\x01"));
        assert!(fix_string.contains("54=1\x01"));
        assert!(fix_string.contains("55=AAPL\x01"));
        assert!(fix_string.contains("60=20190206-16:25:08.968\x01"));
        assert!(fix_string.contains("207=TO\x01"));
        assert!(fix_string.contains("6000=TEST1234\x01"));
        // Checksum should be present (not checking exact value as it's calculated)
        assert!(fix_string.contains("10="));
    }

    #[test]
    fn test_market_data_subscription() {
        let message = FixMessage::builder(
            MsgType::MarketDataRequest,
            "TRADING_SYS".to_string(),
            "MD_PROVIDER".to_string(),
            250,
            "20241201-09:15:00.000".to_string(),
        )
        .symbol("SPY".to_string())
        .field(262, "MD_REQ_001".to_string()) // MDReqID
        .field(263, "1".to_string()) // SubscriptionRequestType
        .field(264, "0".to_string()) // MarketDepth
        .field(265, "1".to_string()) // MDUpdateType
        .build();

        let fix_string = message.to_fix_string();
        assert!(fix_string.contains("35=V")); // MarketDataRequest
        assert!(fix_string.contains("55=SPY"));
        assert!(fix_string.contains("262=MD_REQ_001"));

        // Test parsing back
        let parsed = FixMessage::from_fix_string(&fix_string).unwrap();
        assert_eq!(parsed.msg_type, MsgType::MarketDataRequest);
        assert_eq!(parsed.symbol, Some("SPY".to_string()));
        assert_eq!(parsed.get_field(262), Some(&"MD_REQ_001".to_string()));
    }

    #[cfg(test)]
    mod fromstr_display_tests {
        use super::*;

        #[test]
        fn test_fromstr_trait_usage() {
            // Test clean FromStr usage for MsgType
            let heartbeat: MsgType = "0".parse().unwrap();
            let new_order: MsgType = "D".parse().unwrap();
            let exec_report: MsgType = "8".parse().unwrap();

            assert_eq!(heartbeat, MsgType::Heartbeat);
            assert_eq!(new_order, MsgType::NewOrderSingle);
            assert_eq!(exec_report, MsgType::ExecutionReport);

            // Test clean FromStr usage for Side
            let buy_side: Side = "1".parse().unwrap();
            let sell_side: Side = "2".parse().unwrap();

            assert_eq!(buy_side, Side::Buy);
            assert_eq!(sell_side, Side::Sell);

            // Test clean FromStr usage for OrdStatus
            let new_status: OrdStatus = "0".parse().unwrap();
            let filled_status: OrdStatus = "2".parse().unwrap();
            let pending_new: OrdStatus = "A".parse().unwrap();

            assert_eq!(new_status, OrdStatus::New);
            assert_eq!(filled_status, OrdStatus::Filled);
            assert_eq!(pending_new, OrdStatus::PendingNew);
        }

        #[test]
        fn test_display_trait_usage() {
            // Test Display trait for MsgType
            assert_eq!(format!("{}", MsgType::Heartbeat), "0");
            assert_eq!(format!("{}", MsgType::NewOrderSingle), "D");
            assert_eq!(format!("{}", MsgType::ExecutionReport), "8");
            assert_eq!(
                format!("{}", MsgType::Other("CUSTOM".to_string())),
                "CUSTOM"
            );

            // Test Display trait for Side
            assert_eq!(format!("{}", Side::Buy), "1");
            assert_eq!(format!("{}", Side::Sell), "2");

            // Test Display trait for OrdStatus
            assert_eq!(format!("{}", OrdStatus::New), "0");
            assert_eq!(format!("{}", OrdStatus::Filled), "2");
            assert_eq!(format!("{}", OrdStatus::PendingNew), "A");
            assert_eq!(format!("{}", OrdStatus::PendingReplace), "E");
        }

        #[test]
        fn test_error_handling_with_fromstr() {
            // Test error handling for invalid values
            assert!(MsgType::from_str("INVALID").is_ok()); // MsgType never fails, creates Other
            assert!(Side::from_str("invalid").is_err());
            assert!(OrdStatus::from_str("invalid").is_err());

            // Test that MsgType::Other handles any string
            let custom_msg: MsgType = "CUSTOM_TYPE".parse().unwrap();
            match custom_msg {
                MsgType::Other(s) => assert_eq!(s, "CUSTOM_TYPE"),
                _ => panic!("Expected Other variant"),
            }
        }

        #[test]
        fn test_builder_with_parsed_enums() {
            // Demonstrate using parsed enums in builder pattern
            let msg_type: MsgType = "D".parse().unwrap();
            let side: Side = "1".parse().unwrap();
            let ord_status: OrdStatus = "0".parse().unwrap();

            let message = FixMessage::builder(
                msg_type,
                "TRADER".to_string(),
                "EXCHANGE".to_string(),
                1,
                "20241201-12:00:00.000".to_string(),
            )
            .side(side)
            .ord_status(ord_status)
            .build();

            assert_eq!(message.msg_type, MsgType::NewOrderSingle);
            assert_eq!(message.side, Some(Side::Buy));
            assert_eq!(message.ord_status, Some(OrdStatus::New));
        }

        #[test]
        fn test_round_trip_with_display_and_fromstr() {
            let original_types = vec![
                MsgType::Heartbeat,
                MsgType::NewOrderSingle,
                MsgType::ExecutionReport,
                MsgType::Other("CUSTOM".to_string()),
            ];

            for original in original_types {
                let display_str = format!("{}", original);
                let parsed: MsgType = display_str.parse().unwrap();
                assert_eq!(original, parsed);
            }

            let original_sides = vec![Side::Buy, Side::Sell];
            for original in original_sides {
                let display_str = format!("{}", original);
                let parsed: Side = display_str.parse().unwrap();
                assert_eq!(original, parsed);
            }

            let original_statuses = vec![
                OrdStatus::New,
                OrdStatus::Filled,
                OrdStatus::PendingNew,
                OrdStatus::PendingReplace,
            ];
            for original in original_statuses {
                let display_str = format!("{}", original);
                let parsed: OrdStatus = display_str.parse().unwrap();
                assert_eq!(original, parsed);
            }
        }

        #[test]
        fn test_automatic_string_conversion() {
            // Demonstrate that Display automatically provides to_string()
            let msg_type = MsgType::NewOrderSingle;
            let side = Side::Buy;
            let ord_status = OrdStatus::Filled;

            // These conversions work automatically due to Display trait
            let msg_type_string = msg_type.to_string();
            let side_string = side.to_string();
            let ord_status_string = ord_status.to_string();

            assert_eq!(msg_type_string, "D");
            assert_eq!(side_string, "1");
            assert_eq!(ord_status_string, "2");

            // Can also use format! macro directly
            let formatted = format!("{}-{}-{}", msg_type, side, ord_status);
            assert_eq!(formatted, "D-1-2");

            // No need for custom to_str() methods - Display trait provides everything
            assert_eq!(format!("{}", MsgType::ExecutionReport), "8");
            assert_eq!(MsgType::Heartbeat.to_string(), "0");
        }
    }

    #[test]
    fn test_order_cancel_request() {
        let message = FixMessage::builder(
            MsgType::OrderCancelRequest,
            "CLIENT_SYS".to_string(),
            "BROKER_SYS".to_string(),
            150,
            "20241201-10:45:00.000".to_string(),
        )
        .cl_ord_id("CANCEL_REQ_001".to_string())
        .field(41, "ORIGINAL_ORDER_123".to_string()) // OrigClOrdID
        .order_id("BROKER_ORDER_456".to_string())
        .symbol("TSLA".to_string())
        .side(Side::Buy)
        .field(60, "20241201-10:45:00.000".to_string()) // TransactTime
        .build();

        let fix_string = message.to_fix_string();
        assert!(fix_string.contains("35=F")); // OrderCancelRequest
        assert!(fix_string.contains("11=CANCEL_REQ_001"));
        assert!(fix_string.contains("41=ORIGINAL_ORDER_123"));
        assert!(fix_string.contains("37=BROKER_ORDER_456"));
        assert!(fix_string.contains("55=TSLA"));
        assert!(fix_string.contains("54=1")); // Buy side
    }
}

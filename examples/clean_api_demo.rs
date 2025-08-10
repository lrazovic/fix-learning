//! Demonstration of the clean API using standard Rust traits
//!
//! This example shows how the removal of custom to_str() methods
//! makes the API more idiomatic and cleaner to use.

use fix_learning::{FixMessage, MsgType, OrdStatus, Side, Validate};
use time::OffsetDateTime;

fn main() {
	println!("=== Clean API Demo: No More .to_string() Methods! ===\n");

	// Example 1: Clean enum parsing with FromStr
	println!("1. Clean Enum Parsing:");
	let msg_type: MsgType = "D".parse().unwrap();
	let side: Side = "1".parse().unwrap();
	let ord_status: OrdStatus = "2".parse().unwrap();

	println!("  Parsed: {:?} -> Display: {}", msg_type, msg_type);
	println!("  Parsed: {:?} -> Display: {}", side, side);
	println!("  Parsed: {:?} -> Display: {}", ord_status, ord_status);
	println!();

	// Example 2: Automatic string conversion via Display trait
	println!("2. Automatic String Conversion:");
	let msg_types =
		vec![MsgType::Heartbeat, MsgType::NewOrderSingle, MsgType::ExecutionReport, MsgType::Other("CUSTOM".into())];

	for msg_type in &msg_types {
		// All of these work automatically due to Display trait
		let as_string = msg_type.to_string();
		let formatted = format!("{}", msg_type);

		println!("  {:?} -> to_string(): '{}', format!: '{}'", msg_type, as_string, formatted);
	}
	println!();

	// Example 3: Building messages with clean syntax
	println!("3. Clean Builder Syntax:");
	let message = FixMessage::builder(
		"D".parse().unwrap(), // Clean parsing
		"TRADER",
		"EXCHANGE",
		1,
	)
	.sending_time(OffsetDateTime::now_utc())
	.cl_ord_id("ORDER123")
	.symbol("AAPL")
	.side("1".parse().unwrap()) // Clean parsing
	.order_qty(100.0)
	.price(150.25)
	.build();

	println!("  Built message type: {}", message.header.msg_type);
	println!();

	// Example 4: Format strings and interpolation
	println!("4. Clean String Formatting:");
	let buy_side = Side::Buy;
	let sell_side = Side::Sell;
	let filled_status = OrdStatus::Filled;
	let new_status = OrdStatus::New;

	println!("  Order: {} shares {} side, status: {}", 100, buy_side, new_status);
	println!("  Order: {} shares {} side, status: {}", 50, sell_side, filled_status);

	// Complex formatting
	let summary =
		format!("Trade Summary: {} -> {} -> {}", MsgType::NewOrderSingle, MsgType::ExecutionReport, OrdStatus::Filled);
	println!("  {}", summary);
	println!();

	// Example 5: Error handling with Result types
	println!("5. Clean Error Handling:");
	let test_values = vec!["1", "2", "invalid", ""];

	for value in test_values {
		match value.parse::<Side>() {
			Ok(side) => println!("  '{}' -> {} ({})", value, side, format!("{:?}", side)),
			Err(_) => println!("  '{}' -> Invalid side value", value),
		}
	}
	println!();

	// Example 6: Working with Collections:
	println!("6. Working with Collections:");
	let sides = vec![Side::Buy, Side::Sell];
	let side_strings: Vec<String> = sides.iter().map(|s| s.to_string()).collect();

	println!("  Sides as strings: {:?}", side_strings);

	let statuses = vec![OrdStatus::New, OrdStatus::PartiallyFilled, OrdStatus::Filled];

	println!("  Status progression: {}", statuses.iter().map(|s| format!("{}", s)).collect::<Vec<_>>().join(" -> "));
	println!();

	// Example 7: Serialization preview
	println!("7. Message Serialization:");
	let fix_string = message.to_fix_string();
	println!("  FIX String: {}", message);

	// Parse it back
	match FixMessage::from_fix_string(&fix_string) {
		Ok(parsed) => {
			println!("  Parsed back - Type: {}, Is Valid: {:?}", parsed.header.msg_type, parsed.body.is_valid());
		},
		Err(e) => println!("  Parse error: {}", e),
	}
}

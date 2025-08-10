//! Example showing how to build the exact FIX message from the user's original string
//!
//! Original message string (with SOH represented as |):
//! 8=FIX.4.2|9=163|35=D|34=972|49=TESTBUY3|52=20190206-16:25:10.403|56=TESTSELL3|11=14163685067084226997921|21=2|38=100|40=1|54=1|55=AAPL|60=20190206-16:25:08.968|207=TO|6000=TEST1234|10=106

use fix_learning::FixMessage;
use time::macros::datetime;

fn main() {
	println!("=== Building User's Exact FIX Message ===\n");

	// Original message without SOH separators for reference
	let original_string = "8=FIX.4.29=16335=D34=97249=TESTBUY352=20190206-16:25:10.40356=TESTSELL311=14163685067084226997921=238=10040=154=155=AAPL60=20190206-16:25:08.968207=TO6000=TEST123410=106";

	println!("Original FIX string (no SOH):");
	println!("{}\n", original_string);

	// Build the exact same message using the builder pattern with FromStr
	let user_message = FixMessage::builder(
		"D".parse().unwrap(), // 35=D (NewOrderSingle) - using FromStr
		"TESTBUY3",           // 49=TESTBUY3 (SenderCompID)
		"TESTSELL3",          // 56=TESTSELL3 (TargetCompID)
		972,                  // 34=972 (MsgSeqNum)
	)
	.sending_time(datetime!(2019-02-06 16:25:10.403 UTC)) // 52=20190206-16:25:10.403 (SendingTime)
	// Standard FIX fields
	.cl_ord_id("14163685067084226997921") // 11=14163685067084226997921 (ClOrdID)
	.order_qty(100.0) // 38=100 (OrderQty)
	.ord_type("1") // 40=1 (OrdType - Market order)
	.side("1".parse().unwrap()) // 54=1 (Side - Buy) - using FromStr
	.symbol("AAPL") // 55=AAPL (Symbol)
	// Custom fields using the field() method
	.field(21, "2") // 21=2 (HandlInst)
	.field(60, "20190206-16:25:08.968") // 60=20190206-16:25:08.968 (TransactTime)
	.field(207, "TO") // 207=TO (SecurityExchange)
	.field(6000, "TEST1234") // 6000=TEST1234 (Custom field)
	.build();

	// Serialize to FIX wire format
	let fix_wire = user_message.to_fix_string();
	println!("Built FIX message (wire format):");
	println!("{}\n", fix_wire);

	// Show in readable format
	println!("Built FIX message (readable):");
	println!("{}\n", user_message);

	// Demonstrate field breakdown
	println!("=== Field Breakdown ===");
	println!("8=FIX.4.2                    - BeginString (FIX version)");
	println!(
		"9={}                        - BodyLength (calculated automatically)",
		fix_wire.split('\x01').find(|s| s.starts_with("9=")).unwrap_or("9=?").split('=').nth(1).unwrap_or("?")
	);
	println!("35=D                         - MsgType (NewOrderSingle)");
	println!("34=972                       - MsgSeqNum");
	println!("49=TESTBUY3                  - SenderCompID");
	println!("52=20190206-16:25:10.403     - SendingTime");
	println!("56=TESTSELL3                 - TargetCompID");
	println!("11=14163685067084226997921   - ClOrdID (Client Order ID)");
	println!("21=2                         - HandlInst (Handling Instruction)");
	println!("38=100                       - OrderQty (Order Quantity)");
	println!("40=1                         - OrdType (Market Order)");
	println!("54=1                         - Side (Buy)");
	println!("55=AAPL                      - Symbol");
	println!("60=20190206-16:25:08.968     - TransactTime");
	println!("207=TO                       - SecurityExchange");
	println!("6000=TEST1234                - Custom Field");
	println!(
		"10={}                          - CheckSum (calculated automatically)",
		fix_wire.split('\x01').last().unwrap_or("10=?")
	);

	// Parse it back to verify
	println!("\n=== Verification (Parse Back) ===");
	match FixMessage::from_fix_string(&fix_wire) {
		Ok(parsed) => {
			println!("✓ Message parsed successfully!");
			println!("  Message Type: {:?}", parsed.msg_type);
			println!("  Sender: {}", parsed.sender_comp_id);
			println!("  Target: {}", parsed.target_comp_id);
			println!("  Seq Number: {}", parsed.msg_seq_num);
			println!("  Client Order ID: {:?}", parsed.cl_ord_id);
			println!("  Symbol: {:?}", parsed.symbol);
			println!("  Side: {:?}", parsed.side);
			println!("  Quantity: {:?}", parsed.order_qty);
			println!("  Order Type: {:?}", parsed.ord_type);
			println!("  HandlInst (Tag 21): {:?}", parsed.get_field(21));
			println!("  TransactTime (Tag 60): {:?}", parsed.get_field(60));
			println!("  SecurityExchange (Tag 207): {:?}", parsed.get_field(207));
			println!("  Custom Field (Tag 6000): {:?}", parsed.get_field(6000));
		},
		Err(e) => {
			println!("✗ Parse error: {}", e);
		},
	}

	println!("let msg = FixMessage::builder(\"D\".parse().unwrap(), ...)  // NewOrderSingle");
	println!("  .cl_ord_id(\"ORDER123\")");
	println!("  .symbol(\"AAPL\")");
	println!("  .side(\"1\".parse().unwrap())  // Buy");
	println!("  .build();");
	println!();
	println!("// Traditional usage");
	println!("let msg = FixMessage::builder(MsgType::NewOrderSingle, ...)");
	println!("  .side(Side::Buy)");
	println!("  .build();");
	println!();
	println!("// With custom fields");
	println!("let msg = FixMessage::builder(...)");
	println!("  .field(207, \"NASDAQ\")  // SecurityExchange");
	println!("  .field(6000, \"CUSTOM\")  // Custom tag");
	println!("  .build();");
	println!();
	println!("// Clean enum conversions");
	println!("let msg_type: MsgType = \"D\".parse().unwrap();");
	println!("let side: Side = \"1\".parse().unwrap();");
	println!("println!(\"MsgType: {{}}\", msg_type);  // Uses Display trait");
	println!();
	println!("// Serialize to FIX wire format");
	println!("let fix_string = msg.to_fix_string();");
	println!();
	println!("// Parse from FIX wire format");
	println!("let parsed = FixMessage::from_fix_string(&fix_string)?;");
}

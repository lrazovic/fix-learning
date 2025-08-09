//! FIX 4.2 Protocol Implementation
//!
//! This library provides structures and functionality for working with
//! Financial Information eXchange (FIX) 4.2 protocol messages.

pub mod macros;

use std::{borrow::Cow, collections::HashMap, str::FromStr};

// FIX 4.2 Message Types
fix_enum!(Loose MsgType {
	Heartbeat => "0",
	TestRequest => "1",
	ResendRequest => "2",
	Reject => "3",
	SequenceReset => "4",
	Logout => "5",
	ExecutionReport => "8",
	OrderCancelReject => "9",
	NewOrderSingle => "D",
	OrderCancelRequest => "F",
	OrderCancelReplaceRequest => "G",
	OrderStatusRequest => "H",
	MarketDataRequest => "V",
	MarketDataSnapshot => "W",
	MarketDataIncrementalRefresh => "X",
	SecurityDefinitionRequest => "c",
	SecurityDefinition => "d",
});

fix_enum!(Strict Side {
	Buy  => "1",
	Sell => "2",
});

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

const SOH: &str = &"\x01"; // SOH character

// Main FIX 4.2 Message struct
#[derive(Debug, Clone, PartialEq)]
pub struct FixMessage {
	// Standard Header Fields
	pub begin_string: String,   // Tag 8 - Always "FIX.4.2"
	pub body_length: u32,       // Tag 9 - Length of message body
	pub msg_type: MsgType,      // Tag 35 - Message type
	pub sender_comp_id: String, // Tag 49 - Sender's company ID
	pub target_comp_id: String, // Tag 56 - Target's company ID
	pub msg_seq_num: u32,       // Tag 34 - Message sequence number
	pub sending_time: String,   // Tag 52 - Time of message transmission

	// Optional Header Fields
	pub poss_dup_flag: Option<bool>,       // Tag 43 - Possible duplicate flag
	pub poss_resend: Option<bool>,         // Tag 97 - Possible resend flag
	pub orig_sending_time: Option<String>, // Tag 122 - Original sending time

	// Common Body Fields (varies by message type)
	pub cl_ord_id: Option<String>,       // Tag 11 - Client order ID
	pub order_id: Option<String>,        // Tag 37 - Order ID
	pub exec_id: Option<String>,         // Tag 17 - Execution ID
	pub exec_type: Option<String>,       // Tag 150 - Execution type
	pub ord_status: Option<OrdStatus>,   // Tag 39 - Order status
	pub symbol: Option<String>,          // Tag 55 - Symbol
	pub security_type: Option<String>,   // Tag 167 - Security type
	pub side: Option<Side>,              // Tag 54 - Side
	pub order_qty: Option<f64>,          // Tag 38 - Order quantity
	pub ord_type: Option<String>,        // Tag 40 - Order type
	pub price: Option<f64>,              // Tag 44 - Price
	pub last_qty: Option<f64>,           // Tag 32 - Last quantity
	pub last_px: Option<f64>,            // Tag 31 - Last price
	pub leaves_qty: Option<f64>,         // Tag 151 - Leaves quantity
	pub cum_qty: Option<f64>,            // Tag 14 - Cumulative quantity
	pub avg_px: Option<f64>,             // Tag 6 - Average price
	pub text: Option<String>,            // Tag 58 - Free format text
	pub time_in_force: Option<String>,   // Tag 59 - Time in force
	pub exec_inst: Option<String>,       // Tag 18 - Execution instructions
	pub handl_inst: Option<String>,      // Tag 21 - Handling instructions
	pub exec_ref_id: Option<String>,     // Tag 19 - Execution reference ID
	pub exec_trans_type: Option<String>, // Tag 20 - Execution transaction type

	// Additional fields as key-value pairs for extensibility
	pub additional_fields: HashMap<u32, String>,

	// Trailer
	pub checksum: Cow<'static, str>, // Tag 10 - Checksum
}

impl FixMessage {
	// Create a new FIX message with required fields
	pub fn new(
		msg_type: MsgType,
		sender_comp_id: impl Into<String>,
		target_comp_id: impl Into<String>,
		msg_seq_num: u32,
		sending_time: impl Into<String>,
	) -> Self {
		FixMessage {
			begin_string: "FIX.4.2".to_string(),
			body_length: 0, // Will be calculated when serializing
			msg_type,
			sender_comp_id: sender_comp_id.into(),
			target_comp_id: target_comp_id.into(),
			msg_seq_num,
			sending_time: sending_time.into(),
			poss_dup_flag: None,
			poss_resend: None,
			orig_sending_time: None,
			cl_ord_id: None,
			order_id: None,
			exec_id: None,
			exec_type: None,
			ord_status: None,
			symbol: None,
			security_type: None,
			side: None,
			order_qty: None,
			ord_type: None,
			price: None,
			last_qty: None,
			last_px: None,
			leaves_qty: None,
			cum_qty: None,
			avg_px: None,
			text: None,
			time_in_force: None,
			exec_inst: None,
			handl_inst: None,
			exec_ref_id: None,
			exec_trans_type: None,
			additional_fields: HashMap::new(),
			checksum: Cow::Borrowed("000"), // Will be calculated when serializing
		}
	}

	// Set a custom field
	pub fn set_field(&mut self, tag: u32, value: impl Into<String>) {
		self.additional_fields.insert(tag, value.into());
	}

	// Get a custom field
	pub fn get_field(&self, tag: u32) -> Option<&String> {
		self.additional_fields.get(&tag)
	}

	// Check if message is valid (basic validation)
	pub fn is_valid(&self) -> bool {
		!self.begin_string.is_empty() &&
			!self.sender_comp_id.is_empty() &&
			!self.target_comp_id.is_empty() &&
			!self.sending_time.is_empty()
	}
}

impl Default for FixMessage {
	fn default() -> Self {
		FixMessage::new(MsgType::Heartbeat, "SENDER", "TARGET", 1, "19700101-00:00:00.000")
	}
}

// Builder pattern for creating FIX messages
#[derive(Debug)]
pub struct FixMessageBuilder {
	message: FixMessage,
}

impl FixMessageBuilder {
	/// Create a new builder with required fields
	pub fn new(
		msg_type: MsgType,
		sender_comp_id: impl Into<String>,
		target_comp_id: impl Into<String>,
		msg_seq_num: u32,
		sending_time: impl Into<String>,
	) -> Self {
		Self { message: FixMessage::new(msg_type, sender_comp_id, target_comp_id, msg_seq_num, sending_time) }
	}

	/// Create a builder from an existing message
	pub fn from_message(message: FixMessage) -> Self {
		Self { message }
	}

	// Header field setters
	pub fn body_length(mut self, body_length: u32) -> Self {
		self.message.body_length = body_length;
		self
	}

	pub fn poss_dup_flag(mut self, flag: bool) -> Self {
		self.message.poss_dup_flag = Some(flag);
		self
	}

	pub fn poss_resend(mut self, flag: bool) -> Self {
		self.message.poss_resend = Some(flag);
		self
	}

	pub fn orig_sending_time(mut self, time: impl Into<String>) -> Self {
		self.message.orig_sending_time = Some(time.into());
		self
	}

	// Body field setters
	pub fn cl_ord_id(mut self, cl_ord_id: impl Into<String>) -> Self {
		self.message.cl_ord_id = Some(cl_ord_id.into());
		self
	}

	pub fn order_id(mut self, order_id: impl Into<String>) -> Self {
		self.message.order_id = Some(order_id.into());
		self
	}

	pub fn exec_id(mut self, exec_id: impl Into<String>) -> Self {
		self.message.exec_id = Some(exec_id.into());
		self
	}

	pub fn exec_type(mut self, exec_type: impl Into<String>) -> Self {
		self.message.exec_type = Some(exec_type.into());
		self
	}

	pub fn ord_status(mut self, ord_status: OrdStatus) -> Self {
		self.message.ord_status = Some(ord_status);
		self
	}

	pub fn symbol(mut self, symbol: impl Into<String>) -> Self {
		self.message.symbol = Some(symbol.into());
		self
	}

	pub fn security_type(mut self, security_type: impl Into<String>) -> Self {
		self.message.security_type = Some(security_type.into());
		self
	}

	pub fn side(mut self, side: Side) -> Self {
		self.message.side = Some(side);
		self
	}

	pub fn order_qty(mut self, qty: f64) -> Self {
		self.message.order_qty = Some(qty);
		self
	}

	pub fn ord_type(mut self, ord_type: impl Into<String>) -> Self {
		self.message.ord_type = Some(ord_type.into());
		self
	}

	pub fn price(mut self, price: f64) -> Self {
		self.message.price = Some(price);
		self
	}

	pub fn last_qty(mut self, qty: f64) -> Self {
		self.message.last_qty = Some(qty);
		self
	}

	pub fn last_px(mut self, price: f64) -> Self {
		self.message.last_px = Some(price);
		self
	}

	pub fn leaves_qty(mut self, qty: f64) -> Self {
		self.message.leaves_qty = Some(qty);
		self
	}

	pub fn cum_qty(mut self, qty: f64) -> Self {
		self.message.cum_qty = Some(qty);
		self
	}

	pub fn avg_px(mut self, price: f64) -> Self {
		self.message.avg_px = Some(price);
		self
	}

	pub fn text(mut self, text: impl Into<String>) -> Self {
		self.message.text = Some(text.into());
		self
	}

	pub fn time_in_force(mut self, tif: impl Into<String>) -> Self {
		self.message.time_in_force = Some(tif.into());
		self
	}

	pub fn exec_inst(mut self, exec_inst: impl Into<String>) -> Self {
		self.message.exec_inst = Some(exec_inst.into());
		self
	}

	pub fn handl_inst(mut self, handl_inst: impl Into<String>) -> Self {
		self.message.handl_inst = Some(handl_inst.into());
		self
	}

	pub fn exec_ref_id(mut self, exec_ref_id: impl Into<String>) -> Self {
		self.message.exec_ref_id = Some(exec_ref_id.into());
		self
	}

	pub fn exec_trans_type(mut self, exec_trans_type: impl Into<String>) -> Self {
		self.message.exec_trans_type = Some(exec_trans_type.into());
		self
	}

	// Custom field setter
	pub fn field(mut self, tag: u32, value: impl Into<String>) -> Self {
		self.message.set_field(tag, value);
		self
	}

	// Checksum setter
	pub fn checksum(mut self, checksum: impl Into<String>) -> Self {
		self.message.checksum = Cow::Owned(checksum.into());
		self
	}

	/// Build the final message
	pub fn build(self) -> FixMessage {
		self.message
	}
}

impl FixMessage {
	/// Create a new builder for this message type
	pub fn builder(
		msg_type: MsgType,
		sender_comp_id: impl Into<String>,
		target_comp_id: impl Into<String>,
		msg_seq_num: u32,
		sending_time: impl Into<String>,
	) -> FixMessageBuilder {
		FixMessageBuilder::new(msg_type, sender_comp_id, target_comp_id, msg_seq_num, sending_time)
	}

	/// Serialize the message to FIX wire format
	pub fn to_fix_string(&self) -> String {
		let mut fields = Vec::new();

		// Standard Header Fields (in order)
		fields.push(format!("8={}", self.begin_string));

		// We'll calculate body length after building the body
		let mut body_fields = Vec::new();

		// Message type
		body_fields.push(format!("35={}", self.msg_type));

		// Message sequence number
		body_fields.push(format!("34={}", self.msg_seq_num));

		// Sender and target
		body_fields.push(format!("49={}", self.sender_comp_id));
		body_fields.push(format!("52={}", self.sending_time));
		body_fields.push(format!("56={}", self.target_comp_id));

		// Optional header fields
		if let Some(flag) = self.poss_dup_flag {
			body_fields.push(format!("43={}", if flag { "Y" } else { "N" }));
		}
		if let Some(flag) = self.poss_resend {
			body_fields.push(format!("97={}", if flag { "Y" } else { "N" }));
		}
		if let Some(ref time) = self.orig_sending_time {
			body_fields.push(format!("122={}", time));
		}

		// Body fields (in tag order for consistency)
		if let Some(ref cl_ord_id) = self.cl_ord_id {
			body_fields.push(format!("11={}", cl_ord_id));
		}
		if let Some(ref exec_ref_id) = self.exec_ref_id {
			body_fields.push(format!("19={}", exec_ref_id));
		}
		if let Some(ref exec_trans_type) = self.exec_trans_type {
			body_fields.push(format!("20={}", exec_trans_type));
		}
		if let Some(ref handl_inst) = self.handl_inst {
			body_fields.push(format!("21={}", handl_inst));
		}
		if let Some(ref last_px) = self.last_px {
			body_fields.push(format!("31={}", last_px));
		}
		if let Some(ref last_qty) = self.last_qty {
			body_fields.push(format!("32={}", last_qty));
		}
		if let Some(ref order_id) = self.order_id {
			body_fields.push(format!("37={}", order_id));
		}
		if let Some(ref order_qty) = self.order_qty {
			body_fields.push(format!("38={}", order_qty));
		}
		if let Some(ref ord_status) = self.ord_status {
			body_fields.push(format!("39={}", ord_status));
		}
		if let Some(ref ord_type) = self.ord_type {
			body_fields.push(format!("40={}", ord_type));
		}
		if let Some(ref price) = self.price {
			body_fields.push(format!("44={}", price));
		}
		if let Some(ref side) = self.side {
			body_fields.push(format!("54={}", side));
		}
		if let Some(ref symbol) = self.symbol {
			body_fields.push(format!("55={}", symbol));
		}
		if let Some(ref text) = self.text {
			body_fields.push(format!("58={}", text));
		}
		if let Some(ref time_in_force) = self.time_in_force {
			body_fields.push(format!("59={}", time_in_force));
		}
		if let Some(ref avg_px) = self.avg_px {
			body_fields.push(format!("6={}", avg_px));
		}
		if let Some(ref cum_qty) = self.cum_qty {
			body_fields.push(format!("14={}", cum_qty));
		}
		if let Some(ref exec_id) = self.exec_id {
			body_fields.push(format!("17={}", exec_id));
		}
		if let Some(ref exec_inst) = self.exec_inst {
			body_fields.push(format!("18={}", exec_inst));
		}
		if let Some(ref exec_type) = self.exec_type {
			body_fields.push(format!("150={}", exec_type));
		}
		if let Some(ref leaves_qty) = self.leaves_qty {
			body_fields.push(format!("151={}", leaves_qty));
		}
		if let Some(ref security_type) = self.security_type {
			body_fields.push(format!("167={}", security_type));
		}

		// Add custom fields (sorted by tag number)
		let mut custom_fields: Vec<_> = self.additional_fields.iter().collect();
		custom_fields.sort_by_key(|(tag, _)| *tag);
		for (tag, value) in custom_fields {
			body_fields.push(format!("{}={}", tag, value));
		}

		// Calculate body length
		let body_string = body_fields.join(SOH);
		let body_length = body_string.len();
		fields.push(format!("9={}", body_length));

		// Add body fields
		fields.extend(body_fields);

		// Add checksum
		let message_without_checksum = fields.join(SOH) + SOH;
		let calculated_checksum = Self::calculate_checksum(&message_without_checksum);
		fields.push(format!("10={:03}", calculated_checksum));

		// Join all fields with SOH
		fields.join(SOH)
	}

	/// Calculate FIX checksum
	fn calculate_checksum(message: &str) -> u32 {
		message.bytes().map(|b| b as u32).sum::<u32>() % 256
	}

	/// Parse a FIX message from wire format
	pub fn from_fix_string(fix_string: &str) -> Result<Self, String> {
		let fields: Vec<&str> = fix_string.split(SOH).filter(|s| !s.is_empty()).collect();

		if fields.is_empty() {
			return Err("Empty FIX message".to_string());
		}

		let mut message = FixMessage::default();

		for field in fields {
			if let Some((tag_str, value)) = field.split_once('=') {
				match tag_str.parse::<u32>() {
					Ok(8) => message.begin_string = value.into(),
					Ok(9) => message.body_length = value.parse().unwrap_or(0),
					Ok(35) => message.msg_type = MsgType::from_str(value).unwrap_or(MsgType::Other(value.into())),
					Ok(34) => message.msg_seq_num = value.parse().unwrap_or(0),
					Ok(49) => message.sender_comp_id = value.into(),
					Ok(52) => message.sending_time = value.into(),
					Ok(56) => message.target_comp_id = value.into(),
					Ok(11) => message.cl_ord_id = Some(value.into()),
					Ok(37) => message.order_id = Some(value.into()),
					Ok(17) => message.exec_id = Some(value.into()),
					Ok(150) => message.exec_type = Some(value.into()),
					Ok(39) => message.ord_status = OrdStatus::from_str(value).ok(),
					Ok(55) => message.symbol = Some(value.into()),
					Ok(167) => message.security_type = Some(value.into()),
					Ok(54) => message.side = Side::from_str(value).ok(),
					Ok(38) => message.order_qty = value.parse().ok(),
					Ok(40) => message.ord_type = Some(value.into()),
					Ok(44) => message.price = value.parse().ok(),
					Ok(32) => message.last_qty = value.parse().ok(),
					Ok(31) => message.last_px = value.parse().ok(),
					Ok(151) => message.leaves_qty = value.parse().ok(),
					Ok(14) => message.cum_qty = value.parse().ok(),
					Ok(6) => message.avg_px = value.parse().ok(),
					Ok(58) => message.text = Some(value.into()),
					Ok(59) => message.time_in_force = Some(value.into()),
					Ok(18) => message.exec_inst = Some(value.into()),
					Ok(21) => message.handl_inst = Some(value.into()),
					Ok(19) => message.exec_ref_id = Some(value.into()),
					Ok(20) => message.exec_trans_type = Some(value.into()),
					Ok(10) => message.checksum = Cow::Owned(value.into()),
					Ok(tag) => {
						message.additional_fields.insert(tag, value.into());
					},
					Err(_) => return Err(format!("Invalid tag: {}", tag_str)),
				}
			}
		}

		Ok(message)
	}
}

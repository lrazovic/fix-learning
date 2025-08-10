//! New Order Single message implementation (MsgType=D)
//!
//! This module implements the FIX 4.2 New Order Single message, which is used for
//! session-level communication to maintain connection liveness and respond
//! to test requests.

use time::OffsetDateTime;

use crate::{
	FORMAT_TIME, Side,
	common::{Validate, ValidationError},
};

#[derive(Debug, Clone, PartialEq)]
pub struct NewOrderSingleBody {
	/// Unique identifier of the order as assigned by institution (Tag 11) - Required
	pub cl_ord_id: String,
	// (Tag 21) - Required
	pub handl_inst: String,
	// (Tag 55) - Required
	pub symbol: String,
	// (Tag 54) - Required
	pub side: Side,
	// (Tag 38) - Either CashOrderQty or OrderQty is required. Note that either, but not both, CashOrderQty or OrderQty should be specified.
	pub order_qty: Option<f64>,
	// (Tag 152)
	pub cash_order_qty: Option<f64>,
	// (Tag 60) - Required
	pub transact_time: OffsetDateTime,
	// (Tag 40) - Required
	pub ord_type: String,
	// (Tag 44) Price - Optional
	pub price: Option<f64>,
	// (Tag 207) - Optional
	pub security_exchange: Option<String>,
}

impl Validate for NewOrderSingleBody {
	fn validate(&self) -> Result<(), ValidationError> {
		// Rust is enforcing the presence of all the required values, so we should do a sanity check of the values.
		if self.order_qty.is_none() && self.cash_order_qty.is_none() {
			return Err(ValidationError::MissingRequiredField("OrderQty or CashOrderQty".to_string()));
		}
		Ok(())
	}
}

impl NewOrderSingleBody {
	/// Create a new empty heartbeat body
	pub fn new() -> Self {
		Self {
			cl_ord_id: String::new(),
			handl_inst: String::new(),
			symbol: String::new(),
			side: Side::Buy,
			order_qty: None,
			price: None,
			cash_order_qty: None,
			transact_time: OffsetDateTime::now_utc(),
			ord_type: String::new(),
			security_exchange: None,
		}
	}

	/// Serialize new order single-specific fields to FIX format
	pub(crate) fn serialize_fields(&self) -> String {
		let mut result = String::new();
		result.push_str(&format!("11={}\x01", self.cl_ord_id));
		result.push_str(&format!("21={}\x01", self.handl_inst));
		result.push_str(&format!("55={}\x01", self.symbol));
		result.push_str(&format!("54={}\x01", self.side));
		result.push_str(&format!("60={}\x01", self.transact_time.format(FORMAT_TIME).unwrap()));
		result.push_str(&format!("40={}\x01", self.ord_type));
		if let Some(security_exchange) = &self.security_exchange {
			result.push_str(&format!("207={}\x01", security_exchange));
		}
		if let Some(price) = self.price {
			result.push_str(&format!("44={}\x01", price));
		}
		result
	}

	/// Parse a heartbeat-specific field
	pub(crate) fn parse_field(&mut self, tag: u32, value: &str) -> Result<(), String> {
		match tag {
			11 => self.cl_ord_id = value.to_string(),
			21 => self.handl_inst = value.to_string(),
			55 => self.symbol = value.to_string(),
			54 => self.side = value.parse().map_err(|_| "Invalid side")?,
			60 => self.transact_time = OffsetDateTime::parse(value, FORMAT_TIME).map_err(|_| "Invalid time format")?,
			40 => self.ord_type = value.to_string(),
			207 => self.security_exchange = Some(value.to_string()),
			44 => self.price = Some(value.parse().map_err(|_| "Invalid price")?),
			_ => return Err(format!("Unknown heartbeat field: {}", tag)),
		}
		Ok(())
	}
}
